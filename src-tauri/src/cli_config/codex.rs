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
