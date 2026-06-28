//! 代理服务 - 数据类型定义

use serde::{Deserialize, Serialize};

/// 代理服务器运行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatus {
    /// 是否在运行
    pub running: bool,
    /// 监听地址
    pub address: String,
    /// 监听端口 (0 表示尚未绑定)
    pub port: u16,
    /// 启动时间 (Unix 秒, 0 表示未启动)
    pub started_at: i64,
    /// 当前生效的 provider id
    pub active_provider_id: Option<String>,
    pub active_provider_name: Option<String>,
    /// 各 app 是否处于接管模式
    pub claude_taken_over: bool,
    pub codex_taken_over: bool,
}

impl Default for ProxyStatus {
    fn default() -> Self {
        Self {
            running: false,
            address: "127.0.0.1".to_string(),
            port: 0,
            started_at: 0,
            active_provider_id: None,
            active_provider_name: None,
            claude_taken_over: false,
            codex_taken_over: false,
        }
    }
}

/// 启动代理服务器的返回信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyServerInfo {
    pub address: String,
    pub port: u16,
    pub started_at: i64,
}

/// 切换接管结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeoverResult {
    pub success: bool,
    pub message: String,
}
