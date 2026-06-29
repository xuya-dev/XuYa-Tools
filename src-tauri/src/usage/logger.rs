//! 请求日志写入器

use crate::db::Database;

use super::types::RequestLog;

/// 写入一条请求日志
pub fn log_request(db: &Database, log: &RequestLog) -> Result<i64, String> {
    let conn = crate::lock_conn!(db.conn);
    conn.execute(
        "INSERT INTO proxy_request_logs (
            request_id, app_type, provider_id, provider_name, model, request_model,
            input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens,
            total_cost_usd, latency_ms, first_token_ms,
            status_code, error_message, is_streaming, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        rusqlite::params![
            log.request_id,
            log.app_type,
            log.provider_id,
            log.provider_name,
            log.model,
            log.request_model,
            log.input_tokens as i64,
            log.output_tokens as i64,
            log.cache_read_tokens as i64,
            log.cache_creation_tokens as i64,
            log.total_cost_usd,
            log.latency_ms as i64,
            log.first_token_ms.map(|v| v as i64),
            log.status_code as i64,
            log.error_message,
            log.is_streaming as i64,
            log.created_at,
        ],
    )
    .map_err(|e| format!("写入请求日志失败: {e}"))?;
    Ok(conn.last_insert_rowid())
}
