//! Codex CLI live config 适配器
//!
//! 路径约定 (与 cc-switch 对齐):
//!   ~/.codex/auth.json   - 凭据 (OPENAI_API_KEY / OPENAI_BASE_URL)
//!   ~/.codex/config.toml - 行为配置 (model 等)
//!
//! Phase 1 只做「最小可识别 + 可写入」，不做 modelCatalog / backfill。

use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};
use toml::Table;

use super::types::{CliLiveStatus, CliProvider};

/// ~/.codex 目录
pub fn codex_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
}

pub fn codex_auth_path() -> PathBuf {
    codex_config_dir().join("auth.json")
}

pub fn codex_config_path() -> PathBuf {
    codex_config_dir().join("config.toml")
}

/// 只读探测当前 Codex live config 状态
pub fn detect() -> CliLiveStatus {
    let auth_path = codex_auth_path();
    let mut status = CliLiveStatus {
        config_path: auth_path.to_string_lossy().to_string(),
        ..Default::default()
    };

    if auth_path.exists() {
        status.installed = true;
        if let Ok(content) = fs::read_to_string(&auth_path) {
            if let Ok(root) = serde_json::from_str::<Value>(&content) {
                status.base_url = root
                    .get("OPENAI_BASE_URL")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
            }
        }
    }

    // model 一般在 config.toml
    let cfg_path = codex_config_path();
    if cfg_path.exists() {
        if let Ok(content) = fs::read_to_string(&cfg_path) {
            if let Ok(table) = content.parse::<Table>() {
                status.model = table
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
            }
        }
    }

    status
}

/// 写入 Codex live config: auth.json + config.toml
///
/// 优先级: 若 provider 携带了 codex_auth_json / codex_config_toml (来自双编辑器),
/// 则直接写入; 否则用表单字段 (api_key/base_url/model) 生成默认配置。
pub fn write_live(provider: &CliProvider) -> std::io::Result<()> {
    let dir = codex_config_dir();
    fs::create_dir_all(&dir)?;

    // ---------- auth.json ----------
    let auth_path = codex_auth_path();

    if !provider.codex_auth_json.trim().is_empty() {
        // 双编辑器模式: 直接写入用户编辑的完整内容 (先校验 JSON 合法性)
        if let Ok(parsed) = serde_json::from_str::<Value>(&provider.codex_auth_json) {
            write_atomic_json(&auth_path, &parsed)?;
        } else {
            // JSON 非法则回退到字段生成
            write_auth_from_fields(provider, &auth_path)?;
        }
    } else {
        write_auth_from_fields(provider, &auth_path)?;
    }

    // ---------- config.toml ----------
    let cfg_path = codex_config_path();

    if !provider.codex_config_toml.trim().is_empty() {
        // 双编辑器模式: 直接写入用户编辑的完整 TOML 内容
        let text = provider.codex_config_toml.clone();
        let tmp = cfg_path.with_extension("toml.tmp");
        fs::write(&tmp, &text)?;
        fs::rename(&tmp, &cfg_path)?;
    } else {
        write_config_toml_from_fields(provider, &cfg_path)?;
    }

    Ok(())
}

/// 用表单字段 (api_key/base_url) 生成 auth.json
/// 对齐 cc-switch generateThirdPartyAuth: OPENAI_API_KEY + OPENAI_BASE_URL
fn write_auth_from_fields(provider: &CliProvider, auth_path: &PathBuf) -> std::io::Result<()> {
    let mut auth: Value = fs::read_to_string(auth_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));
    if !auth.is_object() {
        auth = json!({});
    }
    let auth_obj = auth
        .as_object_mut()
        .expect("已重置 auth 为对象,此处必为对象");

    if !provider.api_key.is_empty() {
        auth_obj.insert("OPENAI_API_KEY".into(), json!(provider.api_key));
    }
    if !provider.base_url.is_empty() {
        auth_obj.insert("OPENAI_BASE_URL".into(), json!(provider.base_url));
    }
    write_atomic_json(auth_path, &auth)
}

/// 用表单字段生成 config.toml
/// 对齐 cc-switch generateThirdPartyConfig: 完整写出 model_provider + [model_providers.custom]
/// 这样字段模式 (未填双编辑器) 切过去 Codex 也能正常工作。
fn write_config_toml_from_fields(
    provider: &CliProvider,
    cfg_path: &PathBuf,
) -> std::io::Result<()> {
    // 字段模式: 完整生成, 不保留旧 table (避免半残配置混入)
    let model = primary_codex_model(provider);
    let provider_name = if provider.name.is_empty() {
        "custom".to_string()
    } else {
        provider.name.clone()
    };

    let mut text = String::new();
    text.push_str("model_provider = \"custom\"\n");
    text.push_str(&format!("model = {}\n", toml_quote(&model)));
    text.push_str("model_reasoning_effort = \"high\"\n");
    text.push_str("disable_response_storage = true\n\n");

    text.push_str("[model_providers.custom]\n");
    text.push_str(&format!("name = {}\n", toml_quote(&provider_name)));
    text.push_str(&format!("base_url = {}\n", toml_quote(&provider.base_url)));
    text.push_str("wire_api = \"responses\"\n");
    text.push_str("requires_openai_auth = true\n");

    let tmp = cfg_path.with_extension("toml.tmp");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, cfg_path)
}

/// 取 Codex 主模型: model > model_sonnet > model_opus > model_haiku > 默认
fn primary_codex_model(provider: &CliProvider) -> String {
    if !provider.model.is_empty() {
        return provider.model.clone();
    }
    if !provider.model_sonnet.is_empty() {
        return provider.model_sonnet.clone();
    }
    if !provider.model_opus.is_empty() {
        return provider.model_opus.clone();
    }
    if !provider.model_haiku.is_empty() {
        return provider.model_haiku.clone();
    }
    "gpt-5.5".to_string()
}

/// TOML 字符串字面量 (带引号)
fn toml_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn write_atomic_json(path: &PathBuf, value: &Value) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)
}

/// 在 TOML 文本中替换某 key 的值 (支持 top-level 和 [section] 内)。
fn toml_replace_value(content: &str, section: Option<&str>, key: &str, value: &str) -> String {
    let quoted = format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""));
    let lines: Vec<&str> = content.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut in_target_section = section.is_none();
    let mut found = false;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // 归一化: 去掉括号内首尾空格, 支持 [ x ] 和 [x.y] 两种写法
            let section_name = trimmed[1..trimmed.len() - 1].trim();
            in_target_section = section.map_or(true, |s| section_name == s);
            out.push(line.to_string());
            continue;
        }

        if in_target_section && !found {
            let prefix_sp = format!("{key} ");
            let prefix_eq = format!("{key}=");
            if trimmed.starts_with(&prefix_sp) || trimmed.starts_with(&prefix_eq) {
                let indent_len = line.len() - line.trim_start().len();
                out.push(format!("{}{key} = {}", " ".repeat(indent_len), quoted));
                found = true;
                continue;
            }
        }
        out.push(line.to_string());
    }

    if !found {
        if let Some(s) = section {
            let header = format!("[{s}]");
            if !content.contains(&header) {
                out.push(String::new());
                out.push(header);
            }
            out.push(format!("{key} = {quoted}"));
        } else {
            out.insert(0, format!("{key} = {quoted}"));
        }
    }
    out.join("\n")
}

/// 接管模式: 把 config.toml 的 [model_providers.custom].base_url 改为代理地址。
/// 不动 model / 其他字段。
pub fn update_config_base_url(base_url: &str) -> std::io::Result<()> {
    let path = codex_config_path();
    let content = fs::read_to_string(&path).unwrap_or_default();
    let updated = toml_replace_value(
        &content,
        Some("model_providers.custom"),
        "base_url",
        base_url,
    );
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, updated)?;
    fs::rename(&tmp, &path)
}

/// 接管模式下切 provider: 只更新 config.toml 的 model 字段, 不动 base_url。
pub fn update_live_model(provider: &CliProvider) -> std::io::Result<()> {
    let path = codex_config_path();
    let content = fs::read_to_string(&path).unwrap_or_default();
    let model = primary_codex_model(provider);
    let updated = toml_replace_value(&content, None, "model", &model);
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, updated)?;
    fs::rename(&tmp, &path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_config::types::{
        ApiFormat, AuthField, CliProvider, ProviderCategory, ProviderKind, ProviderScope,
    };

    fn sample_provider(model: &str) -> CliProvider {
        CliProvider {
            id: String::new(),
            name: "test".into(),
            scope: ProviderScope::Codex,
            kind: ProviderKind::Relay,
            category: ProviderCategory::Custom,
            base_url: "https://api.example.com".into(),
            api_key: "sk-test".into(),
            model: model.into(),
            model_sonnet: String::new(),
            model_haiku: String::new(),
            model_opus: String::new(),
            sonnet_name: String::new(),
            opus_name: String::new(),
            haiku_name: String::new(),
            sonnet_1m: false,
            opus_1m: false,
            haiku_1m: false,
            note: String::new(),
            website_url: String::new(),
            auth_field: AuthField::AnthropicAuthToken,
            api_format: ApiFormat::OpenaiChat,
            custom_user_agent: String::new(),
            models_url: String::new(),
            preset_id: String::new(),
            icon: String::new(),
            icon_color: String::new(),
            codex_auth_json: String::new(),
            codex_config_toml: String::new(),
            updated_at: 0,
        }
    }

    #[test]
    fn test_toml_replace_top_level() {
        let content = "model = \"gpt-4o\"\nfoo = \"bar\"\n";
        let result = toml_replace_value(content, None, "model", "o3");
        assert!(result.contains("model = \"o3\""));
        assert!(result.contains("foo = \"bar\""));
    }

    #[test]
    fn test_toml_replace_section_value() {
        let content = "[model_providers.custom]\nname = \"test\"\nbase_url = \"https://old.com\"\n";
        let result = toml_replace_value(
            content,
            Some("model_providers.custom"),
            "base_url",
            "https://new.com",
        );
        assert!(result.contains("https://new.com"));
        assert!(!result.contains("https://old.com"));
        assert!(result.contains("name = \"test\""));
    }

    #[test]
    fn test_toml_replace_missing_key_adds() {
        let content = "model = \"gpt-4o\"\n";
        let result = toml_replace_value(content, None, "max_tokens", "4096");
        assert!(result.contains("max_tokens = \"4096\""));
    }

    #[test]
    fn test_toml_replace_missing_section_creates() {
        let content = "model = \"gpt-4o\"\n";
        let result = toml_replace_value(
            content,
            Some("model_providers.custom"),
            "base_url",
            "https://proxy.local",
        );
        assert!(result.contains("[model_providers.custom]"));
        assert!(result.contains("https://proxy.local"));
    }

    #[test]
    fn test_primary_codex_model() {
        assert_eq!(primary_codex_model(&sample_provider("gpt-5")), "gpt-5");
        assert_eq!(primary_codex_model(&sample_provider("")), "gpt-5.5");
    }

    #[test]
    fn test_toml_quote_escapes() {
        assert_eq!(toml_quote("hello"), "\"hello\"");
        assert_eq!(toml_quote("say \"hi\""), "\"say \\\"hi\\\"\"");
        assert_eq!(toml_quote(r"C:\path"), "\"C:\\path\"");
    }
}
