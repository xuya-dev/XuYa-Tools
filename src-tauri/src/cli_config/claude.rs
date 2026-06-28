//! Claude Code live config 适配器
//!
//! 路径约定 (与 cc-switch 对齐):
//!   主配置: ~/.claude/settings.json (旧版可能是 claude.json)
//!   base URL 放在 env.ANTHROPIC_BASE_URL
//!   token 放在 env.ANTHROPIC_AUTH_TOKEN 或 env.ANTHROPIC_API_KEY
//!   模型放在 env.ANTHROPIC_MODEL

use std::fs;
use std::path::PathBuf;

use serde_json::{json, Map, Value};

use super::types::{CliLiveStatus, CliProvider};

/// ~/.claude 目录
pub fn claude_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
}

/// 主配置文件路径 (settings.json 优先，兼容 claude.json)
pub fn claude_settings_path() -> PathBuf {
    let dir = claude_config_dir();
    let settings = dir.join("settings.json");
    if settings.exists() {
        return settings;
    }
    let legacy = dir.join("claude.json");
    if legacy.exists() {
        return legacy;
    }
    settings
}

/// 只读探测当前 Claude live config 状态
pub fn detect() -> CliLiveStatus {
    let path = claude_settings_path();
    let mut status = CliLiveStatus {
        config_path: path.to_string_lossy().to_string(),
        ..Default::default()
    };
    if !path.exists() {
        return status;
    }
    status.installed = true;
    let Ok(content) = fs::read_to_string(&path) else {
        return status;
    };
    let Ok(root) = serde_json::from_str::<Value>(&content) else {
        return status;
    };

    if let Some(env) = root.get("env").and_then(Value::as_object) {
        status.base_url = env
            .get("ANTHROPIC_BASE_URL")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        status.model = env
            .get("ANTHROPIC_MODEL")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        status.model_sonnet = env
            .get("ANTHROPIC_DEFAULT_SONNET_MODEL")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        status.model_opus = env
            .get("ANTHROPIC_DEFAULT_OPUS_MODEL")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        status.model_haiku = env
            .get("ANTHROPIC_DEFAULT_HAIKU_MODEL")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
    }
    status
}

/// 把指定 provider 写成 Claude live config。
/// 采用「合并写」: 保留原有 JSON 顶层字段，只覆盖 env 区块相关键。
pub fn write_live(provider: &CliProvider) -> std::io::Result<()> {
    let path = claude_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 读出已有内容，不存在则用空对象
    let mut root: Value = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));

    if !root.is_object() {
        root = json!({});
    }
    let root_obj = root.as_object_mut().unwrap();
    let env_entry = root_obj
        .entry("env".to_string())
        .or_insert_with(|| json!({}));
    if !env_entry.is_object() {
        *env_entry = json!({});
    }
    let env = env_entry.as_object_mut().unwrap();

    set_env(env, provider);
    write_atomic_json(&path, &root)
}

fn set_env(env: &mut Map<String, Value>, provider: &CliProvider) {
    // base URL
    if !provider.base_url.is_empty() {
        env.insert(
            "ANTHROPIC_BASE_URL".into(),
            json!(provider.base_url),
        );
    } else {
        env.remove("ANTHROPIC_BASE_URL");
    }

    // token: 官方登录场景没有 key，保留原有 key 不动
    if !provider.api_key.is_empty() {
        let key_name = match provider.auth_field {
            crate::cli_config::types::AuthField::AnthropicApiKey => "ANTHROPIC_API_KEY",
            crate::cli_config::types::AuthField::AnthropicAuthToken => "ANTHROPIC_AUTH_TOKEN",
        };
        env.insert(key_name.into(), json!(provider.api_key));
        // 写入主认证字段后, 清掉另一个避免冲突
        let other = if key_name == "ANTHROPIC_AUTH_TOKEN" {
            "ANTHROPIC_API_KEY"
        } else {
            "ANTHROPIC_AUTH_TOKEN"
        };
        env.remove(other);
    }

    // 自定义 User-Agent
    if !provider.custom_user_agent.is_empty() {
        env.insert("CLAUDE_CODE_CUSTOM_HEADERS".into(), json!(format!("User-Agent: {}", provider.custom_user_agent)));
    }

    // 模型: 对齐 cc-switch, 按角色分别写入 ANTHROPIC_DEFAULT_{SONNET,OPUS,HAIKU}_MODEL。
    // Claude Code 会按角色用对应模型; 1M 上下文通过模型名 [1M] 后缀声明 (Claude Code 原生识别)。
    let sonnet = with_one_m_marker(&provider.model_sonnet, provider.sonnet_1m);
    let opus = with_one_m_marker(&provider.model_opus, provider.opus_1m);
    let haiku = with_one_m_marker(&provider.model_haiku, provider.haiku_1m);

    if !sonnet.is_empty() {
        env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), json!(sonnet));
    } else {
        env.remove("ANTHROPIC_DEFAULT_SONNET_MODEL");
    }
    if !opus.is_empty() {
        env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), json!(opus));
    } else {
        env.remove("ANTHROPIC_DEFAULT_OPUS_MODEL");
    }
    if !haiku.is_empty() {
        env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), json!(haiku));
    } else {
        env.remove("ANTHROPIC_DEFAULT_HAIKU_MODEL");
    }

    // 各角色显示名 (写入 *_MODEL_NAME, 影响 Claude Code /model 菜单)
    // 仅当对应角色模型非空时写入, 否则移除
    if !sonnet.is_empty() && !provider.sonnet_name.is_empty() {
        env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL_NAME".into(), json!(provider.sonnet_name));
    } else {
        env.remove("ANTHROPIC_DEFAULT_SONNET_MODEL_NAME");
    }
    if !opus.is_empty() && !provider.opus_name.is_empty() {
        env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL_NAME".into(), json!(provider.opus_name));
    } else {
        env.remove("ANTHROPIC_DEFAULT_OPUS_MODEL_NAME");
    }
    if !haiku.is_empty() && !provider.haiku_name.is_empty() {
        env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME".into(), json!(provider.haiku_name));
    } else {
        env.remove("ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME");
    }

    // 兜底模型 ANTHROPIC_MODEL (优先级: 显式 model > sonnet(剥后缀) > opus > haiku)
    let primary = if !provider.model.is_empty() {
        provider.model.clone()
    } else if !provider.model_sonnet.is_empty() {
        strip_one_m_marker(&provider.model_sonnet).to_string()
    } else if !provider.model_opus.is_empty() {
        strip_one_m_marker(&provider.model_opus).to_string()
    } else {
        strip_one_m_marker(&provider.model_haiku).to_string()
    };
    if !primary.is_empty() {
        env.insert("ANTHROPIC_MODEL".into(), json!(primary));
    } else {
        env.remove("ANTHROPIC_MODEL");
    }
}

/// 1M 上下文后缀常量 (Claude Code 原生识别, 对齐 cc-switch ONE_M_CONTEXT_MARKER)
const ONE_M_MARKER: &str = "[1M]";

/// 若启用 1M, 给模型名追加 [1M] 后缀 (已含则不重复追加)
fn with_one_m_marker(model: &str, enable_1m: bool) -> String {
    if !enable_1m || model.is_empty() {
        return model.to_string();
    }
    if has_one_m_marker(model) {
        return model.to_string();
    }
    format!("{model}{ONE_M_MARKER}")
}

/// 剥离模型名末尾的 [1M] 后缀 (大小写不敏感)
fn strip_one_m_marker(model: &str) -> &str {
    let trimmed = model.trim_end();
    if trimmed.len() >= ONE_M_MARKER.len()
        && trimmed[trimmed.len() - ONE_M_MARKER.len()..]
            .eq_ignore_ascii_case(ONE_M_MARKER)
    {
        trimmed[..trimmed.len() - ONE_M_MARKER.len()].trim_end()
    } else {
        model
    }
}

/// 判断模型名是否带 [1M] 后缀 (大小写不敏感)
fn has_one_m_marker(model: &str) -> bool {
    let trimmed = model.trim_end();
    trimmed.len() >= ONE_M_MARKER.len()
        && trimmed[trimmed.len() - ONE_M_MARKER.len()..]
            .eq_ignore_ascii_case(ONE_M_MARKER)
}

fn write_atomic_json(path: &PathBuf, value: &Value) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_config::types::*;

    #[test]
    fn test_one_m_marker_helpers() {
        // with_one_m_marker
        assert_eq!(with_one_m_marker("claude-sonnet-4", true), "claude-sonnet-4[1M]");
        assert_eq!(with_one_m_marker("claude-sonnet-4", false), "claude-sonnet-4");
        assert_eq!(with_one_m_marker("", true), "");
        // 不重复追加
        assert_eq!(with_one_m_marker("claude-sonnet-4[1M]", true), "claude-sonnet-4[1M]");
        // 大小写不敏感
        assert_eq!(with_one_m_marker("claude-sonnet-4[1m]", true), "claude-sonnet-4[1m]");
    }

    #[test]
    fn test_strip_one_m_marker() {
        assert_eq!(strip_one_m_marker("claude-sonnet-4[1M]"), "claude-sonnet-4");
        assert_eq!(strip_one_m_marker("claude-sonnet-4[1m]"), "claude-sonnet-4");
        assert_eq!(strip_one_m_marker("claude-sonnet-4"), "claude-sonnet-4");
        assert_eq!(strip_one_m_marker("claude-sonnet-4 [1M] "), "claude-sonnet-4");
    }

    #[test]
    fn test_has_one_m_marker() {
        assert!(has_one_m_marker("claude-sonnet-4[1M]"));
        assert!(has_one_m_marker("claude-sonnet-4[1m]"));
        assert!(!has_one_m_marker("claude-sonnet-4"));
    }

    #[test]
    fn test_set_env_writes_per_role_models() {
        let mut env = Map::new();
        let provider = CliProvider {
            id: "t".into(), name: "t".into(), scope: ProviderScope::Claude,
            kind: ProviderKind::Relay, category: ProviderCategory::Custom,
            base_url: "https://x.com".into(), api_key: "sk".into(),
            model: String::new(),
            model_sonnet: "claude-sonnet-4".into(),
            model_haiku: "claude-haiku-4".into(),
            model_opus: "claude-opus-4".into(),
            sonnet_name: "我的 Sonnet".into(),
            opus_name: String::new(),
            haiku_name: String::new(),
            sonnet_1m: true,
            opus_1m: false,
            haiku_1m: false,
            note: String::new(), website_url: String::new(),
            auth_field: AuthField::AnthropicAuthToken, api_format: ApiFormat::Anthropic,
            custom_user_agent: String::new(), models_url: String::new(), preset_id: String::new(),
            icon: String::new(), icon_color: String::new(),
            codex_auth_json: String::new(), codex_config_toml: String::new(),
            updated_at: 0,
        };
        set_env(&mut env, &provider);
        // sonnet 带 1M 后缀
        assert_eq!(env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").unwrap().as_str().unwrap(), "claude-sonnet-4[1M]");
        // opus/haiku 不带
        assert_eq!(env.get("ANTHROPIC_DEFAULT_OPUS_MODEL").unwrap().as_str().unwrap(), "claude-opus-4");
        assert_eq!(env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").unwrap().as_str().unwrap(), "claude-haiku-4");
        // 兜底 ANTHROPIC_MODEL 取 sonnet (剥后缀)
        assert_eq!(env.get("ANTHROPIC_MODEL").unwrap().as_str().unwrap(), "claude-sonnet-4");
        // 显示名: sonnet 有 (其他为空不写)
        assert_eq!(env.get("ANTHROPIC_DEFAULT_SONNET_MODEL_NAME").unwrap().as_str().unwrap(), "我的 Sonnet");
        assert!(env.get("ANTHROPIC_DEFAULT_OPUS_MODEL_NAME").is_none());
    }

    #[test]
    fn test_set_env_removes_empty_role_models() {
        let mut env = Map::new();
        // 预置一个旧的 sonnet model
        env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), json!("old-value"));
        let provider = CliProvider {
            id: "t".into(), name: "t".into(), scope: ProviderScope::Claude,
            kind: ProviderKind::Relay, category: ProviderCategory::Custom,
            base_url: "https://x.com".into(), api_key: "sk".into(),
            model: "gpt-4o".into(),
            model_sonnet: String::new(), // 空应移除旧值
            model_haiku: String::new(),
            model_opus: String::new(),
            sonnet_name: String::new(),
            opus_name: String::new(),
            haiku_name: String::new(),
            sonnet_1m: false, opus_1m: false, haiku_1m: false,
            note: String::new(), website_url: String::new(),
            auth_field: AuthField::AnthropicAuthToken, api_format: ApiFormat::Anthropic,
            custom_user_agent: String::new(), models_url: String::new(), preset_id: String::new(),
            icon: String::new(), icon_color: String::new(),
            codex_auth_json: String::new(), codex_config_toml: String::new(),
            updated_at: 0,
        };
        set_env(&mut env, &provider);
        assert!(env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").is_none());
        assert_eq!(env.get("ANTHROPIC_MODEL").unwrap().as_str().unwrap(), "gpt-4o");
    }
}
