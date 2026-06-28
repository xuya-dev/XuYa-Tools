//! 接管逻辑
//!
//! 启用接管: 把某 CLI 的 live config base_url 改指向本地代理。
//! 停止接管: 从备份恢复原始配置。
//!
//! Phase 2 MVP: 不做 PROXY_MANAGED 占位符 token 替换,
//! 保留原 token 让请求带真实凭据转发 (足够验证链路)。

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::cli_config::{claude, codex};
use super::types::TakeoverResult;

/// 接管 Claude Code: 把 base_url 指向 proxy_url
pub fn takeover_claude(proxy_url: &str) -> TakeoverResult {
    let path = claude::claude_settings_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut root: Value = read_json_or_empty(&path);
    let env_entry = root
        .as_object_mut()
        .unwrap()
        .entry("env".to_string())
        .or_insert_with(|| json!({}));
    if !env_entry.is_object() {
        *env_entry = json!({});
    }
    env_entry
        .as_object_mut()
        .unwrap()
        .insert("ANTHROPIC_BASE_URL".into(), json!(proxy_url));

    write_or_fail(&path, &root, "Claude Code 已接管")
}

/// 接管 Codex: auth.json 的 OPENAI_BASE_URL 指向 proxy_url
pub fn takeover_codex(proxy_url: &str) -> TakeoverResult {
    let dir = codex::codex_config_dir();
    let _ = fs::create_dir_all(&dir);
    let path = codex::codex_auth_path();

    let mut root = read_json_or_empty(&path);
    root.as_object_mut()
        .unwrap()
        .insert("OPENAI_BASE_URL".into(), json!(proxy_url));

    write_or_fail(&path, &root, "Codex CLI 已接管")
}

/// 恢复 Claude: 用备份覆盖 live settings
pub fn restore_claude(backup: &Path) -> TakeoverResult {
    restore_from_backup(backup, &claude::claude_settings_path(), "Claude Code")
}

/// 恢复 Codex: 用备份覆盖 auth.json
pub fn restore_codex(backup: &Path) -> TakeoverResult {
    restore_from_backup(backup, &codex::codex_auth_path(), "Codex CLI")
}

fn read_json_or_empty(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .filter(|v: &Value| v.is_object())
        .unwrap_or_else(|| json!({}))
}

fn write_or_fail(path: &Path, value: &Value, ok_msg: &str) -> TakeoverResult {
    match write_atomic_json(path, value) {
        Ok(_) => TakeoverResult {
            success: true,
            message: ok_msg.to_string(),
        },
        Err(e) => TakeoverResult {
            success: false,
            message: format!("写入失败: {e}"),
        },
    }
}

fn restore_from_backup(backup: &Path, live: &Path, name: &str) -> TakeoverResult {
    if !backup.exists() {
        return TakeoverResult {
            success: false,
            message: format!("{name} 备份不存在, 无法恢复"),
        };
    }
    match fs::copy(backup, live) {
        Ok(_) => TakeoverResult {
            success: true,
            message: format!("{name} 已恢复"),
        },
        Err(e) => TakeoverResult {
            success: false,
            message: format!("恢复失败: {e}"),
        },
    }
}

fn write_atomic_json(path: &Path, value: &Value) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)
}
