//! Usage 数据类型

use serde::{Deserialize, Serialize};

/// 请求日志 (写入用)
#[derive(Debug, Clone)]
pub struct RequestLog {
    pub request_id: String,
    pub app_type: String,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub model: Option<String>,
    pub request_model: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub total_cost_usd: f64,
    pub latency_ms: u64,
    pub first_token_ms: Option<u64>,
    pub status_code: u16,
    pub error_message: Option<String>,
    pub is_streaming: bool,
    pub created_at: i64,
}

/// 汇总统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummary {
    pub total_requests: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub success_rate: f64,
    pub avg_latency_ms: u64,
    pub avg_first_token_ms: u64,
    pub streaming_count: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cost_usd: f64,
}

impl Default for UsageSummary {
    fn default() -> Self {
        Self {
            total_requests: 0,
            success_count: 0,
            error_count: 0,
            success_rate: 0.0,
            avg_latency_ms: 0,
            avg_first_token_ms: 0,
            streaming_count: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cache_read_tokens: 0,
            total_cache_creation_tokens: 0,
            total_cost_usd: 0.0,
        }
    }
}

/// 单日统计 (用于图表)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyStat {
    pub date: String,   // YYYY-MM-DD
    pub timestamp: i64, // 当天 0 点 Unix 秒
    pub request_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cost_usd: f64,
    pub avg_latency_ms: u64,
}

/// 日志过滤器
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogFilters {
    pub app_type: Option<String>,
    pub provider_id: Option<String>,
    pub status_code: Option<u16>,
}

/// 分页结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedLogs {
    pub data: Vec<RequestLogDetail>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

/// 日志详情 (读取用)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestLogDetail {
    pub id: i64,
    pub request_id: String,
    pub app_type: String,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub model: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub total_cost_usd: f64,
    pub latency_ms: u64,
    pub first_token_ms: Option<u64>,
    pub status_code: u16,
    pub error_message: Option<String>,
    pub is_streaming: bool,
    pub created_at: i64,
}
