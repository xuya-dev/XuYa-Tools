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
                .filter_map(|m| {
                    m.get("text")
                        .and_then(|t| t.as_str())
                        .filter(|t| !t.is_empty())
                        .map(String::from)
                })
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
    if result
        .get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
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
            let args_str = func
                .get("arguments")
                .and_then(|a| a.as_str())
                .unwrap_or("{}");
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
    let prompt_tokens = usage
        .get("prompt_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let input_tokens = prompt_tokens
        .saturating_sub(cached)
        .saturating_sub(cache_creation);
    let output_tokens = usage
        .get("completion_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

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
                        let media_type = source
                            .get("media_type")
                            .and_then(|m| m.as_str())
                            .unwrap_or("image/png");
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
                    let tool_use_id = block
                        .get("tool_use_id")
                        .and_then(|i| i.as_str())
                        .unwrap_or("");
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
                .map(|(k, v)| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(k).unwrap_or_default(),
                        canonical_json_string(v)
                    )
                })
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

// ==================== 反向转换: OpenAI → Anthropic ====================

/// SSE 转换器 trait (正向和反向共用)
pub trait SseConvert: Send {
    fn feed(&mut self, data: &[u8]) -> Vec<u8>;
    fn flush(&mut self) -> Vec<u8>;
}

impl SseConvert for SseConverter {
    fn feed(&mut self, data: &[u8]) -> Vec<u8> {
        SseConverter::feed(self, data)
    }
    fn flush(&mut self) -> Vec<u8> {
        SseConverter::flush(self)
    }
}

impl SseConvert for AnthropicSseConverter {
    fn feed(&mut self, data: &[u8]) -> Vec<u8> {
        AnthropicSseConverter::feed(self, data)
    }
    fn flush(&mut self) -> Vec<u8> {
        AnthropicSseConverter::flush(self)
    }
}

/// OpenAI Chat Completions 请求 → Anthropic Messages 请求
///
/// 用于 Codex CLI (发 /v1/chat/completions) 指向 Anthropic 原生上游的场景。
pub fn openai_to_anthropic_request(body: &Value) -> Result<Value, String> {
    let messages = body
        .get("messages")
        .and_then(|v| v.as_array())
        .ok_or("缺少 messages")?;

    // 提取 system message → Anthropic 顶层 system 字段
    let mut system_text = String::new();
    let mut anthropic_messages: Vec<Value> = Vec::new();
    for msg in messages {
        let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
        if role == "system" {
            if let Some(content) = msg.get("content") {
                if let Some(s) = content.as_str() {
                    if !system_text.is_empty() {
                        system_text.push('\n');
                    }
                    system_text.push_str(s);
                }
            }
            continue;
        }
        let content = msg.get("content");
        let anthropic_content = match content {
            Some(Value::String(s)) => json!(s),
            Some(Value::Array(parts)) => {
                let blocks: Vec<Value> = parts.iter().filter_map(|part| {
                    let ptype = part.get("type").and_then(|t| t.as_str())?;
                    match ptype {
                        "text" => Some(json!({"type": "text", "text": part.get("text").and_then(|t| t.as_str()).unwrap_or("")})),
                        "image_url" => {
                            let url = part.get("image_url")
                                .and_then(|iu| iu.get("url"))
                                .and_then(|u| u.as_str())
                                .unwrap_or("");
                            if let Some(b64) = url.strip_prefix("data:") {
                                let (mime, data) = b64.split_once(',').unwrap_or(("image/png", b64));
                                let clean_mime = mime.split(';').next().unwrap_or("image/png");
                                Some(json!({"type": "image", "source": {"type": "base64", "media_type": clean_mime, "data": data}}))
                            } else { None }
                        }
                        _ => None,
                    }
                }).collect();
                Value::Array(blocks)
            }
            _ => json!(""),
        };

        let mapped_role = if role == "assistant" {
            "assistant"
        } else {
            "user"
        };
        anthropic_messages.push(json!({"role": mapped_role, "content": anthropic_content}));
    }

    let mut result = json!({"messages": anthropic_messages});
    if !system_text.is_empty() {
        result["system"] = json!(system_text);
    }
    if let Some(m) = body.get("model") {
        result["model"] = m.clone();
    }
    if let Some(mt) = body.get("max_tokens").or(body.get("max_completion_tokens")) {
        result["max_tokens"] = mt.clone();
    } else {
        result["max_tokens"] = json!(4096);
    }
    for key in &["temperature", "top_p", "stop"] {
        if let Some(v) = body.get(key) {
            result[key] = v.clone();
        }
    }
    if let Some(s) = body.get("stream") {
        result["stream"] = s.clone();
    }

    // tools: OpenAI functions → Anthropic tools
    if let Some(tools) = body.get("tools").and_then(|t| t.as_array()) {
        let anthropic_tools: Vec<Value> = tools.iter().filter_map(|tool| {
            let func = tool.get("function")?;
            Some(json!({
                "name": func.get("name").and_then(|n| n.as_str()).unwrap_or(""),
                "description": func.get("description").and_then(|d| d.as_str()).unwrap_or(""),
                "input_schema": func.get("parameters").cloned().unwrap_or(json!({"type": "object", "properties": {}}))
            }))
        }).collect();
        if !anthropic_tools.is_empty() {
            result["tools"] = json!(anthropic_tools);
        }
    }

    Ok(result)
}

/// Anthropic 响应 → OpenAI Chat Completions 响应 (非流式)
pub fn anthropic_to_openai_response(resp: &Value) -> Result<Value, String> {
    let content_blocks = resp.get("content").and_then(|c| c.as_array());
    let mut text_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<Value> = Vec::new();

    if let Some(blocks) = content_blocks {
        for block in blocks {
            match block.get("type").and_then(|t| t.as_str()) {
                Some("text") => {
                    if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                        text_parts.push(t.to_string());
                    }
                }
                Some("tool_use") => {
                    let id = block.get("id").and_then(|i| i.as_str()).unwrap_or("call_0");
                    let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let args = block.get("input").cloned().unwrap_or(json!({}));
                    let args_str = serde_json::to_string(&args).unwrap_or_default();
                    tool_calls.push(json!({
                        "id": id,
                        "type": "function",
                        "function": {"name": name, "arguments": args_str}
                    }));
                }
                _ => {}
            }
        }
    }

    let stop_reason = resp
        .get("stop_reason")
        .and_then(|s| s.as_str())
        .unwrap_or("end_turn");
    let finish_reason = match stop_reason {
        "end_turn" | "stop_sequence" => "stop",
        "max_tokens" => "length",
        "tool_use" => "tool_calls",
        _ => "stop",
    };

    let mut message = json!({"role": "assistant", "content": text_parts.join("")});
    if !tool_calls.is_empty() {
        message["tool_calls"] = json!(tool_calls);
    }

    let usage = resp.get("usage").cloned().unwrap_or(json!({}));
    let input_tokens = usage
        .get("input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let output_tokens = usage
        .get("output_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    Ok(json!({
        "id": resp.get("id").and_then(|i| i.as_str()).unwrap_or("chatcmpl-proxy"),
        "object": "chat.completion",
        "model": resp.get("model").and_then(|m| m.as_str()).unwrap_or(""),
        "choices": [{
            "index": 0,
            "message": message,
            "finish_reason": finish_reason
        }],
        "usage": {
            "prompt_tokens": input_tokens,
            "completion_tokens": output_tokens,
            "total_tokens": input_tokens + output_tokens
        }
    }))
}

/// Anthropic SSE → OpenAI SSE 流式转换器。
///
/// 用于 Codex CLI 指向 Anthropic 原生上游时, 把 Anthropic 流式响应转回 OpenAI 格式。
pub struct AnthropicSseConverter {
    buf: String,
    started: bool,
    finished: bool,
    tool_call_index: u32,
    current_block_type: Option<&'static str>,
}

impl AnthropicSseConverter {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            started: false,
            finished: false,
            tool_call_index: 0,
            current_block_type: None,
        }
    }

    pub fn feed(&mut self, data: &[u8]) -> Vec<u8> {
        let text = String::from_utf8_lossy(data);
        self.buf.push_str(&text.replace("\r\n", "\n"));
        let mut out = String::new();

        while let Some(pos) = self.buf.find("\n\n") {
            let raw = self.buf[..pos].to_string();
            self.buf = self.buf[pos + 2..].to_string();
            self.process_event(&raw, &mut out);
        }
        out.into_bytes()
    }

    pub fn flush(&mut self) -> Vec<u8> {
        let mut out = String::new();
        if !self.buf.is_empty() {
            self.process_event(&self.buf.clone(), &mut out);
            self.buf.clear();
        }
        if !self.finished {
            out.push_str(
                "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
            );
            out.push_str("data: [DONE]\n\n");
            self.finished = true;
        }
        out.into_bytes()
    }

    fn process_event(&mut self, raw: &str, out: &mut String) {
        let mut event_type = String::new();
        let mut data_str = String::new();
        for line in raw.lines() {
            let line = line.trim();
            if let Some(et) = line.strip_prefix("event:") {
                event_type = et.trim().to_string();
            } else if let Some(dt) = line.strip_prefix("data:") {
                data_str = dt.trim().to_string();
            }
        }
        if data_str.is_empty() || data_str == "[DONE]" {
            return;
        }
        let Ok(data) = serde_json::from_str::<Value>(&data_str) else {
            return;
        };

        match event_type.as_str() {
            "message_start" => {
                self.started = true;
                let model = data
                    .pointer("/message/model")
                    .and_then(|m| m.as_str())
                    .unwrap_or("");
                let id = data
                    .pointer("/message/id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("chatcmpl-proxy");
                out.push_str(&format!(
                    "data: {}\n\n",
                    json!({"id": id, "object": "chat.completion.chunk", "model": model,
                        "choices": [{"index": 0, "delta": {"role": "assistant", "content": ""}, "finish_reason": null}]
                    }),
                ));
            }
            "content_block_delta" => {
                let delta_type = data
                    .pointer("/delta/type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                match delta_type {
                    "text_delta" => {
                        let text = data
                            .pointer("/delta/text")
                            .and_then(|t| t.as_str())
                            .unwrap_or("");
                        if !text.is_empty() {
                            out.push_str(&format!("data: {}\n\n", json!({
                                "choices": [{"index": 0, "delta": {"content": text}, "finish_reason": null}]
                            })));
                        }
                    }
                    "thinking_delta" => {
                        let thinking = data
                            .pointer("/delta/thinking")
                            .and_then(|t| t.as_str())
                            .unwrap_or("");
                        if !thinking.is_empty() {
                            out.push_str(&format!("data: {}\n\n", json!({
                                "choices": [{"index": 0, "delta": {"reasoning_content": thinking}, "finish_reason": null}]
                            })));
                        }
                    }
                    "input_json_delta" => {
                        let partial = data
                            .pointer("/delta/partial_json")
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        if !partial.is_empty() {
                            let idx = self.tool_call_index;
                            out.push_str(&format!("data: {}\n\n", json!({
                                "choices": [{"index": 0, "delta": {"tool_calls": [{"index": idx, "function": {"arguments": partial}}]}, "finish_reason": null}]
                            })));
                        }
                    }
                    _ => {}
                }
            }
            "content_block_start" => {
                let btype = data
                    .pointer("/content_block/type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                self.current_block_type = Some(if btype == "tool_use" {
                    "tool_use"
                } else if btype == "thinking" {
                    "thinking"
                } else {
                    "text"
                });
                if btype == "tool_use" {
                    let id = data
                        .pointer("/content_block/id")
                        .and_then(|i| i.as_str())
                        .unwrap_or("call_0");
                    let name = data
                        .pointer("/content_block/name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    let idx = self.tool_call_index;
                    out.push_str(&format!("data: {}\n\n", json!({
                        "choices": [{"index": 0, "delta": {"tool_calls": [{"index": idx, "id": id, "type": "function", "function": {"name": name, "arguments": ""}}]}, "finish_reason": null}]
                    })));
                }
            }
            "content_block_stop" => {
                // P1-3: tool_use block 结束 → 递增 tool_call_index 供下一个 tool call 使用
                if self.current_block_type == Some("tool_use") {
                    self.tool_call_index += 1;
                }
                self.current_block_type = None;
            }
            "message_delta" => {
                let stop_reason = data.pointer("/delta/stop_reason").and_then(|s| s.as_str());
                let finish_reason = match stop_reason {
                    Some("end_turn") | Some("stop_sequence") => "stop",
                    Some("max_tokens") => "length",
                    Some("tool_use") => "tool_calls",
                    _ => "stop",
                };
                let usage = data.get("usage").cloned().unwrap_or(json!({}));
                let output_tokens = usage
                    .get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                out.push_str(&format!("data: {}\n\n", json!({
                    "choices": [{"index": 0, "delta": {}, "finish_reason": finish_reason}],
                    "usage": {"prompt_tokens": 0, "completion_tokens": output_tokens, "total_tokens": output_tokens}
                })));
            }
            "message_stop" if !self.finished => {
                out.push_str("data: [DONE]\n\n");
                self.finished = true;
            }
            _ => {}
        }
    }
}

/// OpenAI SSE → Anthropic SSE 流式转换器。
///
/// 逐 chunk feed 上游 SSE 字节,输出 Anthropic 格式 SSE 字节。
/// 维护内部缓冲区处理跨 chunk 的不完整 SSE 事件,并管理多个 content block
/// (thinking → text → tool_use) 的生命周期。
pub struct SseConverter {
    buf: String,
    started: bool,
    finished: bool,
    block_index: usize,
    current_block_type: Option<&'static str>, // "thinking" | "text" | "tool_use"
    tool_call_indices: Vec<usize>,            // OpenAI tool_calls[].index → 已分配的 block_index
}

impl SseConverter {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            started: false,
            finished: false,
            block_index: 0,
            current_block_type: None,
            tool_call_indices: Vec::new(),
        }
    }

    /// 输入上游 SSE 字节,返回转换后的 Anthropic SSE 字节。
    pub fn feed(&mut self, data: &[u8]) -> Vec<u8> {
        let text = String::from_utf8_lossy(data);
        // CRLF 规范化 (P0-2): 某些反代/CDN 用 \r\n 行尾
        self.buf.push_str(&text.replace("\r\n", "\n"));
        let mut out = String::new();

        while let Some(pos) = self.buf.find("\n\n") {
            let raw_event = self.buf[..pos].to_string();
            self.buf = self.buf[pos + 2..].to_string();
            self.process_event(&raw_event, &mut out);
        }

        out.into_bytes()
    }

    /// 流结束时 flush 剩余缓冲 + 兜底 emit_stop
    pub fn flush(&mut self) -> Vec<u8> {
        let mut out = String::new();
        if !self.buf.is_empty() {
            self.process_event(&self.buf.clone(), &mut out);
            self.buf.clear();
        }
        if !self.finished {
            self.close_current_block(&mut out);
            self.emit_stop(&mut out, "end_turn", 0);
        }
        out.into_bytes()
    }

    fn process_event(&mut self, raw: &str, out: &mut String) {
        for line in raw.lines() {
            let line = line.trim();
            if !line.starts_with("data:") {
                continue;
            }
            let data = line[5..].trim();
            if data == "[DONE]" {
                if !self.finished {
                    self.close_current_block(out);
                    self.emit_stop(out, "end_turn", 0);
                }
                continue;
            }
            let Ok(json_val) = serde_json::from_str::<Value>(data) else {
                continue;
            };
            self.convert_openai_chunk(&json_val, out);
        }
    }

    fn convert_openai_chunk(&mut self, chunk: &Value, out: &mut String) {
        // 首次: 发 message_start (P1-3: msg_ 前缀)
        if !self.started {
            self.started = true;
            let raw_id = chunk.get("id").and_then(|v| v.as_str()).unwrap_or("proxy");
            let msg_id = if raw_id.starts_with("msg_") {
                raw_id.to_string()
            } else {
                format!("msg_{}", &uuid::Uuid::new_v4().to_string()[..24])
            };
            let model = chunk
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            out.push_str(&format!(
                "event: message_start\ndata: {}\n\n",
                json!({
                    "type": "message_start",
                    "message": {
                        "id": msg_id, "type": "message", "role": "assistant",
                        "content": [], "model": model,
                        "stop_reason": null,
                        "usage": {"input_tokens": 0, "output_tokens": 0}
                    }
                }),
            ));
        }

        let choices = match chunk.get("choices").and_then(|v| v.as_array()) {
            Some(c) => c,
            None => return,
        };
        let choice = match choices.first() {
            Some(c) => c,
            None => return,
        };
        let delta = choice.get("delta");

        // 1. thinking / reasoning_content (P1-1)
        if let Some(reasoning) = delta
            .and_then(|d| d.get("reasoning_content"))
            .and_then(|r| r.as_str())
        {
            if !reasoning.is_empty() {
                self.ensure_block(
                    "thinking",
                    out,
                    || json!({"type": "thinking", "thinking": ""}),
                );
                out.push_str(&format!(
                    "event: content_block_delta\ndata: {}\n\n",
                    json!({
                        "type": "content_block_delta",
                        "index": self.block_index,
                        "delta": {"type": "thinking_delta", "thinking": reasoning}
                    }),
                ));
            }
        }

        // 2. text content
        if let Some(content) = delta
            .and_then(|d| d.get("content"))
            .and_then(|c| c.as_str())
        {
            if !content.is_empty() {
                self.ensure_block("text", out, || json!({"type": "text", "text": ""}));
                out.push_str(&format!(
                    "event: content_block_delta\ndata: {}\n\n",
                    json!({
                        "type": "content_block_delta",
                        "index": self.block_index,
                        "delta": {"type": "text_delta", "text": content}
                    }),
                ));
            }
        }

        // 3. tool_calls (P0-1)
        if let Some(tool_calls) = delta
            .and_then(|d| d.get("tool_calls"))
            .and_then(|t| t.as_array())
        {
            for tc in tool_calls {
                self.handle_tool_call_delta(tc, out);
            }
        }

        // 4. finish_reason
        let finish_reason = choice.get("finish_reason").and_then(|v| v.as_str());
        if let Some(reason) = finish_reason {
            if !self.finished {
                self.close_current_block(out);
                let stop_reason = match reason {
                    "stop" => "end_turn",
                    "length" => "max_tokens",
                    "tool_calls" | "function_call" => "tool_use",
                    _ => "end_turn",
                };
                let output_tokens = chunk
                    .get("usage")
                    .and_then(|u| u.get("completion_tokens"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                self.emit_stop(out, stop_reason, output_tokens);
            }
        }
    }

    /// 确保当前 block 类型匹配 desired; 不匹配则关闭旧 block + 开新 block
    fn ensure_block(
        &mut self,
        desired: &'static str,
        out: &mut String,
        make_block: impl Fn() -> Value,
    ) {
        if self.current_block_type == Some(desired) {
            return;
        }
        // 关闭旧 block
        self.close_current_block(out);
        // 开新 block
        self.current_block_type = Some(desired);
        out.push_str(&format!(
            "event: content_block_start\ndata: {}\n\n",
            json!({
                "type": "content_block_start",
                "index": self.block_index,
                "content_block": make_block(),
            }),
        ));
    }

    /// 处理单个 tool_call delta
    fn handle_tool_call_delta(&mut self, tc: &Value, out: &mut String) {
        let oai_index = tc.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

        // 首次见到这个 tool_call index: 开新 tool_use block
        if oai_index >= self.tool_call_indices.len() {
            // 补齐
            while self.tool_call_indices.len() <= oai_index {
                self.close_current_block(out);
                let block_idx = self.block_index;
                self.tool_call_indices.push(block_idx);
                self.current_block_type = Some("tool_use");

                let call_id = tc
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("toolu_proxy")
                    .to_string();
                let fn_name = tc
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();

                out.push_str(&format!(
                    "event: content_block_start\ndata: {}\n\n",
                    json!({
                        "type": "content_block_start",
                        "index": block_idx,
                        "content_block": {
                            "type": "tool_use",
                            "id": call_id,
                            "name": fn_name,
                            "input": {}
                        }
                    }),
                ));
            }
        }

        // arguments 增量
        if let Some(args) = tc
            .get("function")
            .and_then(|f| f.get("arguments"))
            .and_then(|a| a.as_str())
        {
            if !args.is_empty() {
                let block_idx = self.tool_call_indices.get(oai_index).copied().unwrap_or(0);
                out.push_str(&format!(
                    "event: content_block_delta\ndata: {}\n\n",
                    json!({
                        "type": "content_block_delta",
                        "index": block_idx,
                        "delta": {"type": "input_json_delta", "partial_json": args}
                    }),
                ));
            }
        }
    }

    /// 关闭当前 content block (如果有)
    fn close_current_block(&mut self, out: &mut String) {
        if self.current_block_type.is_some() {
            out.push_str(&format!(
                "event: content_block_stop\ndata: {}\n\n",
                json!({"type": "content_block_stop", "index": self.block_index}),
            ));
            self.block_index += 1;
            self.current_block_type = None;
        }
    }

    fn emit_stop(&mut self, out: &mut String, stop_reason: &str, output_tokens: u64) {
        self.finished = true;
        out.push_str(&format!(
            "event: message_delta\ndata: {}\n\nevent: message_stop\ndata: {}\n\n",
            json!({
                "type": "message_delta",
                "delta": {"stop_reason": stop_reason, "stop_sequence": null},
                "usage": {"output_tokens": output_tokens}
            }),
            json!({"type": "message_stop"}),
        ));
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
        assert_eq!(
            msgs[0]["tool_calls"][0]["function"]["arguments"],
            "{\"city\":\"NYC\"}"
        );
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
        assert_eq!(
            canonical_json_string(&json!({"b": 1, "a": 2})),
            "{\"a\":2,\"b\":1}"
        );
        assert_eq!(canonical_json_string(&json!("hi")), "\"hi\"");
        assert_eq!(canonical_json_string(&json!(null)), "null");
    }

    // ===== SseConverter 测试 =====

    fn make_openai_chunk(content: &str, finish: Option<&str>) -> String {
        let fr = match finish {
            Some(r) => format!("\"finish_reason\":\"{r}\""),
            None => "\"finish_reason\":null".to_string(),
        };
        if content.is_empty() && finish.is_some() {
            format!(
                "{{\"id\":\"chatcmpl-1\",\"model\":\"gpt-4o\",\"choices\":[{{\"index\":0,\"delta\":{{}},{fr}}}]}}"
            )
        } else {
            format!(
                "{{\"id\":\"chatcmpl-1\",\"model\":\"gpt-4o\",\"choices\":[{{\"index\":0,\"delta\":{{\"content\":\"{content}\"}},{fr}}}]}}"
            )
        }
    }

    #[test]
    fn test_sse_basic_text() {
        let mut conv = SseConverter::new();
        let input = format!(
            "data: {}\n\ndata: {}\n\n",
            make_openai_chunk("Hello", None),
            make_openai_chunk("", Some("stop")),
        );
        let out = String::from_utf8(conv.feed(input.as_bytes())).unwrap();
        // 应包含 message_start, text_delta, content_block_stop, message_stop
        assert!(out.contains("message_start"));
        assert!(out.contains("text_delta"));
        assert!(out.contains("\"text\":\"Hello\""));
        assert!(out.contains("message_stop"));
    }

    #[test]
    fn test_sse_crlf() {
        let mut conv = SseConverter::new();
        let input = format!("data: {}\r\n\r\n", make_openai_chunk("Hi", None),);
        let out = String::from_utf8(conv.feed(input.as_bytes())).unwrap();
        assert!(out.contains("text_delta"), "CRLF 分割应正常工作");
    }

    #[test]
    fn test_sse_done_signal() {
        let mut conv = SseConverter::new();
        let input = format!(
            "data: {}\n\ndata: [DONE]\n\n",
            make_openai_chunk("Bye", None)
        );
        let out = String::from_utf8(conv.feed(input.as_bytes())).unwrap();
        assert!(out.contains("message_stop"), "[DONE] 应触发 emit_stop");
    }

    #[test]
    fn test_sse_tool_calls() {
        let mut conv = SseConverter::new();
        // 首帧: tool_call 定义
        let chunk1 = r#"{"id":"chatcmpl-1","model":"gpt-4o","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_abc","type":"function","function":{"name":"get_weather","arguments":""}}]},"finish_reason":null}]}"#;
        // 次帧: tool_call 参数增量
        let chunk2 = r#"{"id":"chatcmpl-1","model":"gpt-4o","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"city\":\"NYC\"}"}}]},"finish_reason":null}]}"#;
        // 终帧
        let chunk3 = r#"{"id":"chatcmpl-1","model":"gpt-4o","choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]}"#;

        let input = format!("data: {chunk1}\n\ndata: {chunk2}\n\ndata: {chunk3}\n\n");
        let out = String::from_utf8(conv.feed(input.as_bytes())).unwrap();
        assert!(out.contains("tool_use"), "应包含 tool_use block");
        assert!(out.contains("get_weather"), "应包含函数名");
        assert!(out.contains("input_json_delta"), "应包含参数增量");
        assert!(
            out.contains("\"stop_reason\":\"tool_use\""),
            "finish_reason=tool_calls → stop_reason=tool_use"
        );
    }

    #[test]
    fn test_sse_cross_chunk_boundary() {
        let mut conv = SseConverter::new();
        // 跨 chunk: 事件被切断
        let part1 = format!("data: {}", make_openai_chunk("Hel", None));
        let part2 = format!("\n\ndata: {}\n\n", make_openai_chunk("", Some("stop")));

        let out1 = String::from_utf8(conv.feed(part1.as_bytes())).unwrap();
        assert!(out1.is_empty(), "不完整事件不应输出");

        let out2 = String::from_utf8(conv.feed(part2.as_bytes())).unwrap();
        assert!(out2.contains("text_delta"), "补全后应输出");
    }

    #[test]
    fn test_sse_msg_id_prefix() {
        let mut conv = SseConverter::new();
        let input = format!("data: {}\n\n", make_openai_chunk("test", None));
        let out = String::from_utf8(conv.feed(input.as_bytes())).unwrap();
        assert!(
            out.contains("\"id\":\"msg_"),
            "msg_id 应以 msg_ 开头, 不应用 chatcmpl-"
        );
    }

    // ===== 反向转换测试 =====

    #[test]
    fn test_openai_to_anthropic_request_basic() {
        let input = json!({
            "model": "claude-sonnet-4",
            "messages": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "Hello"}
            ],
            "max_tokens": 1024
        });
        let result = openai_to_anthropic_request(&input).unwrap();
        assert_eq!(result["system"], "You are helpful.");
        assert_eq!(result["messages"][0]["role"], "user");
        assert_eq!(result["messages"][0]["content"], "Hello");
        assert_eq!(result["max_tokens"], 1024);
    }

    #[test]
    fn test_anthropic_to_openai_response_basic() {
        let input = json!({
            "id": "msg_test",
            "model": "claude-sonnet-4",
            "content": [{"type": "text", "text": "Hi there!"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        });
        let result = anthropic_to_openai_response(&input).unwrap();
        assert_eq!(result["choices"][0]["message"]["content"], "Hi there!");
        assert_eq!(result["choices"][0]["finish_reason"], "stop");
        assert_eq!(result["usage"]["prompt_tokens"], 10);
        assert_eq!(result["usage"]["total_tokens"], 15);
    }

    #[test]
    fn test_anthropic_to_openai_response_tool_use() {
        let input = json!({
            "id": "msg_test",
            "content": [{"type": "tool_use", "id": "toolu_1", "name": "get_weather", "input": {"city": "NYC"}}],
            "stop_reason": "tool_use",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        });
        let result = anthropic_to_openai_response(&input).unwrap();
        assert_eq!(result["choices"][0]["finish_reason"], "tool_calls");
        assert_eq!(
            result["choices"][0]["message"]["tool_calls"][0]["function"]["name"],
            "get_weather"
        );
    }
}
