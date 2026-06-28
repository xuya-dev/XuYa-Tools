//! Anthropic ↔ OpenAI Chat Completions 协议转换层
//!
//! 当 Claude Code (发 anthropic /v1/messages) 指向 openai_chat 格式的供应商时,
//! 请求体需 anthropic→openai 转换, 端点路径 /v1/messages → /v1/chat/completions;
//! 响应体需 openai→anthropic 回转, 这样 Claude Code 才能正确解析。
//!
//! 逻辑搬运自 cc-switch (src-tauri/src/proxy/providers/transform.rs), 精简掉了
//! reasoning_content (Moonshot/Kimi 专用) 和 billing header 剥离等边角逻辑。

use serde_json::{json, Value};

/// Anthropic 请求 → OpenAI Chat Completions 请求
pub fn anthropic_to_openai(body: &Value) -> Result<Value, String> {
    let mut result = json!({});
    let model = body.get("model").and_then(|m| m.as_str()).unwrap_or("");
    if !model.is_empty() {
        result["model"] = json!(model);
    }

    let mut messages = Vec::new();

    // system prompt (string 或 array)
    if let Some(system) = body.get("system") {
        if let Some(text) = system.as_str() {
            if !text.is_empty() {
                messages.push(json!({"role": "system", "content": text}));
            }
        } else if let Some(arr) = system.as_array() {
            let parts: Vec<String> = arr
                .iter()
                .filter_map(|m| m.get("text").and_then(|t| t.as_str()).filter(|t| !t.is_empty()).map(String::from))
                .collect();
            if !parts.is_empty() {
                messages.push(json!({"role": "system", "content": parts.join("\n")}));
            }
        }
    }

    // messages
    if let Some(msgs) = body.get("messages").and_then(|m| m.as_array()) {
        for msg in msgs {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content = msg.get("content");
            let converted = convert_message_to_openai(role, content)?;
            messages.extend(converted);
        }
    }

    normalize_system_messages(&mut messages);
    result["messages"] = json!(messages);

    // 参数: max_tokens (o-series 用 max_completion_tokens)
    if let Some(v) = body.get("max_tokens") {
        if is_openai_o_series(model) {
            result["max_completion_tokens"] = v.clone();
        } else {
            result["max_tokens"] = v.clone();
        }
    }
    for (anthr_key, openai_key) in [
        ("temperature", "temperature"),
        ("top_p", "top_p"),
        ("stream", "stream"),
    ] {
        if let Some(v) = body.get(anthr_key) {
            result[openai_key] = v.clone();
        }
    }
    if let Some(v) = body.get("stop_sequences") {
        result["stop"] = v.clone();
    }

    // reasoning_effort (GPT-5+/o-series)
    if supports_reasoning_effort(model) {
        if let Some(effort) = resolve_reasoning_effort(body) {
            result["reasoning_effort"] = json!(effort);
        }
    }

    // tools
    if let Some(tools) = body.get("tools").and_then(|t| t.as_array()) {
        let openai_tools: Vec<Value> = tools
            .iter()
            .filter(|t| t.get("type").and_then(|v| v.as_str()) != Some("BatchTool"))
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.get("name").and_then(|n| n.as_str()).unwrap_or(""),
                        "description": t.get("description").cloned().unwrap_or(json!("")),
                        "parameters": clean_schema(t.get("input_schema").cloned().unwrap_or(json!({})))
                    }
                })
            })
            .collect();
        if !openai_tools.is_empty() {
            result["tools"] = json!(openai_tools);
        }
    }

    if let Some(v) = body.get("tool_choice") {
        result["tool_choice"] = map_tool_choice(v);
    }

    // 流式请求注入 include_usage, 确保 usage 在末尾 chunk 返回 (便于计费)
    if result.get("stream").and_then(|v| v.as_bool()).unwrap_or(false) {
        match result.get_mut("stream_options") {
            Some(Value::Object(opts)) => {
                opts.insert("include_usage".to_string(), json!(true));
            }
            _ => {
                result["stream_options"] = json!({ "include_usage": true });
            }
        }
    }

    Ok(result)
}

/// OpenAI Chat Completions 响应 → Anthropic Messages 响应
pub fn openai_to_anthropic(body: &Value) -> Result<Value, String> {
    let choices = body
        .get("choices")
        .and_then(|c| c.as_array())
        .ok_or_else(|| "No choices in response".to_string())?;
    let choice = choices
        .first()
        .ok_or_else(|| "Empty choices array".to_string())?;
    let message = choice
        .get("message")
        .ok_or_else(|| "No message in choice".to_string())?;

    let mut content = Vec::new();
    let mut has_tool_use = false;

    // reasoning_content (DeepSeek thinking)
    if let Some(rc) = message.get("reasoning_content").and_then(|r| r.as_str()) {
        if !rc.is_empty() {
            content.push(json!({"type": "thinking", "thinking": rc}));
        }
    }

    // 文本内容
    if let Some(msg_content) = message.get("content") {
        if let Some(text) = msg_content.as_str() {
            if !text.is_empty() {
                content.push(json!({"type": "text", "text": text}));
            }
        } else if let Some(parts) = msg_content.as_array() {
            for part in parts {
                let pt = part.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if pt == "text" || pt == "output_text" {
                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                        if !text.is_empty() {
                            content.push(json!({"type": "text", "text": text}));
                        }
                    }
                }
            }
        }
    }
    // message 级 refusal
    if let Some(refusal) = message.get("refusal").and_then(|r| r.as_str()) {
        if !refusal.is_empty() {
            content.push(json!({"type": "text", "text": refusal}));
        }
    }

    // tool_calls
    if let Some(tool_calls) = message.get("tool_calls").and_then(|t| t.as_array()) {
        if !tool_calls.is_empty() {
            has_tool_use = true;
        }
        for tc in tool_calls {
            let id = tc.get("id").and_then(|i| i.as_str()).unwrap_or("");
            let empty_obj = json!({});
            let func = tc.get("function").unwrap_or(&empty_obj);
            let name = func.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args_str = func.get("arguments").and_then(|a| a.as_str()).unwrap_or("{}");
            let input: Value = serde_json::from_str(args_str).unwrap_or(json!({}));
            content.push(json!({"type": "tool_use", "id": id, "name": name, "input": input}));
        }
    }

    // finish_reason → stop_reason
    let stop_reason = choice
        .get("finish_reason")
        .and_then(|r| r.as_str())
        .map(|r| match r {
            "stop" => "end_turn",
            "length" => "max_tokens",
            "tool_calls" | "function_call" => "tool_use",
            "content_filter" => "end_turn",
            _ => "end_turn",
        })
        .or(if has_tool_use { Some("tool_use") } else { None });

    // usage (含缓存 token 拆分)
    let usage = body.get("usage").cloned().unwrap_or(json!({}));
    let cached = usage
        .get("cache_read_input_tokens")
        .and_then(|v| v.as_u64())
        .or_else(|| {
            usage
                .pointer("/prompt_tokens_details/cached_tokens")
                .and_then(|v| v.as_u64())
        })
        .unwrap_or(0);
    let cache_creation = usage
        .get("cache_creation_input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let prompt_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    let input_tokens = prompt_tokens.saturating_sub(cached).saturating_sub(cache_creation);
    let output_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0);

    let mut usage_json = json!({"input_tokens": input_tokens, "output_tokens": output_tokens});
    if cached > 0 {
        usage_json["cache_read_input_tokens"] = json!(cached);
    }
    if cache_creation > 0 {
        usage_json["cache_creation_input_tokens"] = json!(cache_creation);
    }

    Ok(json!({
        "id": body.get("id").and_then(|i| i.as_str()).unwrap_or(""),
        "type": "message",
        "role": "assistant",
        "content": content,
        "model": body.get("model").and_then(|m| m.as_str()).unwrap_or(""),
        "stop_reason": stop_reason,
        "stop_sequence": null,
        "usage": usage_json
    }))
}

// ==================== 内部 helpers ====================

fn convert_message_to_openai(role: &str, content: Option<&Value>) -> Result<Vec<Value>, String> {
    let mut result = Vec::new();
    let content = match content {
        Some(c) => c,
        None => {
            result.push(json!({"role": role, "content": null}));
            return Ok(result);
        }
    };

    // 字符串内容
    if let Some(text) = content.as_str() {
        result.push(json!({"role": role, "content": text}));
        return Ok(result);
    }

    // 数组内容 (多模态/工具)
    if let Some(blocks) = content.as_array() {
        let mut content_parts = Vec::new();
        let mut tool_calls = Vec::new();

        for block in blocks {
            let bt = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match bt {
                "text" => {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        content_parts.push(json!({"type": "text", "text": text}));
                    }
                }
                "image" => {
                    if let Some(source) = block.get("source") {
                        let media_type = source.get("media_type").and_then(|m| m.as_str()).unwrap_or("image/png");
                        let data = source.get("data").and_then(|d| d.as_str()).unwrap_or("");
                        content_parts.push(json!({
                            "type": "image_url",
                            "image_url": {"url": format!("data:{};base64,{}", media_type, data)}
                        }));
                    }
                }
                "tool_use" => {
                    let id = block.get("id").and_then(|i| i.as_str()).unwrap_or("");
                    let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let input = block.get("input").cloned().unwrap_or(json!({}));
                    tool_calls.push(json!({
                        "id": id, "type": "function",
                        "function": {"name": name, "arguments": canonical_json_string(&input)}
                    }));
                }
                "tool_result" => {
                    let tool_use_id = block.get("tool_use_id").and_then(|i| i.as_str()).unwrap_or("");
                    let content_str = match block.get("content") {
                        Some(Value::String(s)) => s.clone(),
                        Some(v) => canonical_json_string(v),
                        None => String::new(),
                    };
                    result.push(json!({"role": "tool", "tool_call_id": tool_use_id, "content": content_str}));
                }
                _ => {}
            }
        }

        if !content_parts.is_empty() || !tool_calls.is_empty() {
            let mut msg = json!({"role": role});
            if content_parts.is_empty() {
                msg["content"] = Value::Null;
            } else if content_parts.len() == 1 {
                if let Some(text) = content_parts[0].get("text") {
                    msg["content"] = text.clone();
                } else {
                    msg["content"] = json!(content_parts);
                }
            } else {
                msg["content"] = json!(content_parts);
            }
            if !tool_calls.is_empty() {
                msg["tool_calls"] = json!(tool_calls);
            }
            result.push(msg);
        }
        return Ok(result);
    }

    // 其它透传
    result.push(json!({"role": role, "content": content}));
    Ok(result)
}

/// 把 system 消息合并到首条 (OpenAI 要求 system 在最前)
fn normalize_system_messages(messages: &mut Vec<Value>) {
    let system_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m.get("role").and_then(|v| v.as_str()) == Some("system"))
        .map(|(i, _)| i)
        .collect();
    if system_indices.is_empty() {
        return;
    }
    if system_indices.len() == 1 {
        let idx = system_indices[0];
        if idx > 0 {
            let m = messages.remove(idx);
            messages.insert(0, m);
        }
        return;
    }
    // 多条 system 合并
    let mut parts = Vec::new();
    let mut keep = Vec::new();
    for (i, m) in messages.iter().enumerate() {
        if system_indices.contains(&i) {
            if let Some(text) = m.get("content").and_then(|v| v.as_str()) {
                if !text.is_empty() {
                    parts.push(text.to_string());
                }
            }
        } else {
            keep.push(m.clone());
        }
    }
    let mut new_msgs = Vec::new();
    if !parts.is_empty() {
        new_msgs.push(json!({"role": "system", "content": parts.join("\n")}));
    }
    new_msgs.extend(keep);
    *messages = new_msgs;
}

fn map_tool_choice(tool_choice: &Value) -> Value {
    match tool_choice {
        Value::String(s) => match s.as_str() {
            "any" => json!("required"),
            _ => json!(s),
        },
        Value::Object(obj) => match obj.get("type").and_then(|t| t.as_str()) {
            Some("any") => json!("required"),
            Some("auto") => json!("auto"),
            Some("none") => json!("none"),
            Some("tool") => {
                let name = obj.get("name").and_then(|n| n.as_str()).unwrap_or("");
                json!({"type": "function", "function": {"name": name}})
            }
            _ => tool_choice.clone(),
        },
        _ => tool_choice.clone(),
    }
}

fn clean_schema(mut schema: Value) -> Value {
    if let Some(obj) = schema.as_object_mut() {
        if obj.get("format").and_then(|v| v.as_str()) == Some("uri") {
            obj.remove("format");
        }
        if let Some(properties) = obj.get_mut("properties").and_then(|v| v.as_object_mut()) {
            for (_, v) in properties.iter_mut() {
                *v = clean_schema(v.clone());
            }
        }
        if let Some(items) = obj.get_mut("items") {
            *items = clean_schema(items.clone());
        }
    }
    schema
}

/// 稳定的 JSON 字符串 (key 排序), 用于 tool arguments
fn canonical_json_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).unwrap_or_default(),
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(canonical_json_string).collect();
            format!("[{}]", parts.join(","))
        }
        Value::Object(map) => {
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by_key(|(k, _)| *k);
            let parts: Vec<String> = entries
                .into_iter()
                .map(|(k, v)| format!("{}:{}", serde_json::to_string(k).unwrap_or_default(), canonical_json_string(v)))
                .collect();
            format!("{{{}}}", parts.join(","))
        }
    }
}

fn is_openai_o_series(model: &str) -> bool {
    model.len() > 1
        && model.starts_with('o')
        && model.as_bytes().get(1).is_some_and(|b| b.is_ascii_digit())
}

fn supports_reasoning_effort(model: &str) -> bool {
    if is_openai_o_series(model) {
        return true;
    }
    model
        .to_lowercase()
        .strip_prefix("gpt-")
        .and_then(|rest| rest.chars().next())
        .is_some_and(|c| c.is_ascii_digit() && c >= '5')
}

fn resolve_reasoning_effort(body: &Value) -> Option<&'static str> {
    let thinking = body.get("thinking")?;
    match thinking.get("type").and_then(|t| t.as_str()) {
        Some("adaptive") => Some("xhigh"),
        Some("enabled") => {
            let budget = thinking.get("budget_tokens").and_then(|b| b.as_u64());
            match budget {
                Some(b) if b < 4_000 => Some("low"),
                Some(b) if b < 16_000 => Some("medium"),
                Some(_) => Some("high"),
                None => Some("high"),
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_to_openai_basic() {
        let input = json!({
            "model": "gpt-4o",
            "max_tokens": 1024,
            "messages": [
                {"role": "user", "content": "Hello"}
            ]
        });
        let result = anthropic_to_openai(&input).unwrap();
        assert_eq!(result["model"], "gpt-4o");
        assert_eq!(result["max_tokens"], 1024);
        let msgs = result["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0]["role"], "user");
        assert_eq!(msgs[0]["content"], "Hello");
    }

    #[test]
    fn test_anthropic_to_openai_with_system() {
        let input = json!({
            "model": "gpt-4o",
            "max_tokens": 100,
            "system": "You are helpful",
            "messages": [{"role": "user", "content": "Hi"}]
        });
        let result = anthropic_to_openai(&input).unwrap();
        let msgs = result["messages"].as_array().unwrap();
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs[0]["content"], "You are helpful");
        assert_eq!(msgs[1]["role"], "user");
    }

    #[test]
    fn test_anthropic_to_openai_tool_use() {
        let input = json!({
            "model": "gpt-4o",
            "max_tokens": 100,
            "messages": [{
                "role": "assistant",
                "content": [{"type": "tool_use", "id": "t1", "name": "get_weather", "input": {"city": "NYC"}}]
            }]
        });
        let result = anthropic_to_openai(&input).unwrap();
        let msgs = result["messages"].as_array().unwrap();
        assert_eq!(msgs[0]["tool_calls"][0]["function"]["name"], "get_weather");
        assert_eq!(msgs[0]["tool_calls"][0]["function"]["arguments"], "{\"city\":\"NYC\"}");
    }

    #[test]
    fn test_anthropic_to_openai_tool_result() {
        let input = json!({
            "model": "gpt-4o", "max_tokens": 100,
            "messages": [{
                "role": "user",
                "content": [{"type": "tool_result", "tool_use_id": "t1", "content": "sunny"}]
            }]
        });
        let result = anthropic_to_openai(&input).unwrap();
        let msgs = result["messages"].as_array().unwrap();
        assert_eq!(msgs[0]["role"], "tool");
        assert_eq!(msgs[0]["tool_call_id"], "t1");
        assert_eq!(msgs[0]["content"], "sunny");
    }

    #[test]
    fn test_anthropic_to_openai_tools_definition() {
        let input = json!({
            "model": "gpt-4o", "max_tokens": 100,
            "messages": [{"role": "user", "content": "hi"}],
            "tools": [{
                "name": "search", "description": "search web",
                "input_schema": {"type": "object", "properties": {"q": {"type": "string"}}}
            }]
        });
        let result = anthropic_to_openai(&input).unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools[0]["type"], "function");
        assert_eq!(tools[0]["function"]["name"], "search");
    }

    #[test]
    fn test_anthropic_to_openai_stream_injects_usage() {
        let input = json!({"model": "gpt-4o", "max_tokens": 100, "stream": true, "messages": [{"role": "user", "content": "hi"}]});
        let result = anthropic_to_openai(&input).unwrap();
        assert_eq!(result["stream_options"]["include_usage"], true);
    }

    #[test]
    fn test_anthropic_to_openai_o_series_max_completion_tokens() {
        let input = json!({"model": "o3", "max_tokens": 500, "messages": [{"role": "user", "content": "hi"}]});
        let result = anthropic_to_openai(&input).unwrap();
        assert_eq!(result["max_completion_tokens"], 500);
        assert!(result.get("max_tokens").is_none());
    }

    #[test]
    fn test_openai_to_anthropic_basic() {
        let input = json!({
            "id": "chatcmpl-1", "model": "gpt-4o",
            "choices": [{"message": {"role": "assistant", "content": "Hi there"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5}
        });
        let result = openai_to_anthropic(&input).unwrap();
        assert_eq!(result["type"], "message");
        assert_eq!(result["role"], "assistant");
        assert_eq!(result["stop_reason"], "end_turn");
        let content = result["content"].as_array().unwrap();
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[0]["text"], "Hi there");
        assert_eq!(result["usage"]["input_tokens"], 10);
        assert_eq!(result["usage"]["output_tokens"], 5);
    }

    #[test]
    fn test_openai_to_anthropic_tool_calls() {
        let input = json!({
            "id": "1", "model": "gpt-4o",
            "choices": [{
                "message": {
                    "role": "assistant", "content": null,
                    "tool_calls": [{"id": "call_1", "type": "function",
                        "function": {"name": "get_weather", "arguments": "{\"city\":\"SF\"}"}}]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {"prompt_tokens": 5, "completion_tokens": 3}
        });
        let result = openai_to_anthropic(&input).unwrap();
        assert_eq!(result["stop_reason"], "tool_use");
        let content = result["content"].as_array().unwrap();
        assert_eq!(content[0]["type"], "tool_use");
        assert_eq!(content[0]["name"], "get_weather");
        assert_eq!(content[0]["input"]["city"], "SF");
    }

    #[test]
    fn test_openai_to_anthropic_cache_tokens() {
        let input = json!({
            "id": "1", "model": "gpt-4o",
            "choices": [{"message": {"role": "assistant", "content": "ok"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 100, "completion_tokens": 5, "prompt_tokens_details": {"cached_tokens": 80}}
        });
        let result = openai_to_anthropic(&input).unwrap();
        // input = 100 - 80 = 20
        assert_eq!(result["usage"]["input_tokens"], 20);
        assert_eq!(result["usage"]["cache_read_input_tokens"], 80);
    }

    #[test]
    fn test_canonical_json_string() {
        assert_eq!(canonical_json_string(&json!({"b": 1, "a": 2})), "{\"a\":2,\"b\":1}");
        assert_eq!(canonical_json_string(&json!("hi")), "\"hi\"");
        assert_eq!(canonical_json_string(&json!(null)), "null");
    }
}
