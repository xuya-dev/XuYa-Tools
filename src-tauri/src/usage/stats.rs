//! 使用统计聚合查询

use rusqlite::params_from_iter;

use crate::db::Database;

use super::types::{LogFilters, PaginatedLogs, RequestLogDetail, UsageSummary};

/// 起止时间内的汇总 (None 表示全部)
pub fn get_summary(
    db: &Database,
    start: Option<i64>,
    end: Option<i64>,
) -> Result<UsageSummary, String> {
    let conn = crate::lock_conn!(db.conn);

    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(s) = start {
        conditions.push("created_at >= ?".to_string());
        binds.push(Box::new(s));
    }
    if let Some(e) = end {
        conditions.push("created_at < ?".to_string());
        binds.push(Box::new(e));
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT
            COUNT(*) as total,
            COALESCE(SUM(CASE WHEN status_code >= 200 AND status_code < 400 THEN 1 ELSE 0 END), 0) as ok,
            COALESCE(SUM(CASE WHEN status_code >= 400 OR status_code = 0 THEN 1 ELSE 0 END), 0) as err,
            COALESCE(AVG(latency_ms), 0) as avg_lat,
            COALESCE(SUM(input_tokens), 0) as in_tok,
            COALESCE(SUM(output_tokens), 0) as out_tok,
            COALESCE(SUM(total_cost_usd), 0) as cost
         FROM proxy_request_logs {where_clause}"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let row = stmt
        .query_row(params_from_iter(binds.iter().map(|b| b.as_ref())), |row| {
            let total: i64 = row.get(0)?;
            let ok: i64 = row.get(1)?;
            let err: i64 = row.get(2)?;
            let avg_lat: f64 = row.get(3)?;
            let in_tok: i64 = row.get(4)?;
            let out_tok: i64 = row.get(5)?;
            let cost: f64 = row.get(6)?;
            Ok((total, ok, err, avg_lat, in_tok, out_tok, cost))
        })
        .map_err(|e| e.to_string())?;

    let (total, ok, err, avg_lat, in_tok, out_tok, cost) = row;
    let success_rate = if total > 0 {
        ok as f64 / total as f64
    } else {
        0.0
    };

    Ok(UsageSummary {
        total_requests: total as u64,
        success_count: ok as u64,
        error_count: err as u64,
        success_rate,
        avg_latency_ms: avg_lat as u64,
        total_input_tokens: in_tok as u64,
        total_output_tokens: out_tok as u64,
        total_cost_usd: cost,
    })
}

/// 今日 0 点的 Unix 秒 (本地时区)
pub fn today_start_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // 本地时区偏移 (简化: 用 UTC 当天, 误差最多几小时)
    let secs_per_day: i64 = 86400;
    now - (now.rem_euclid(secs_per_day))
}

/// 今日摘要 (供悬浮框)
pub fn today_summary(db: &Database) -> Result<(u64, u64, u64), String> {
    let summary = get_summary(db, Some(today_start_secs()), None)?;
    Ok((
        summary.total_requests,
        summary.error_count,
        summary.avg_latency_ms,
    ))
}

/// 分页查询日志
pub fn get_logs(
    db: &Database,
    filters: LogFilters,
    page: u32,
    page_size: u32,
) -> Result<PaginatedLogs, String> {
    let conn = crate::lock_conn!(db.conn);
    let page = if page == 0 { 1 } else { page };
    let page_size = if page_size == 0 { 20 } else { page_size };
    let offset = (page - 1) * page_size;

    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(app) = &filters.app_type {
        if !app.is_empty() {
            conditions.push("app_type = ?".to_string());
            binds.push(Box::new(app.clone()));
        }
    }
    if let Some(pid) = &filters.provider_id {
        if !pid.is_empty() {
            conditions.push("provider_id = ?".to_string());
            binds.push(Box::new(pid.clone()));
        }
    }
    if let Some(code) = filters.status_code {
        conditions.push("status_code = ?".to_string());
        binds.push(Box::new(code as i64));
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // 总数
    let total_sql = format!("SELECT COUNT(*) FROM proxy_request_logs {where_clause}");
    let total: u32 = conn
        .query_row(
            &total_sql,
            params_from_iter(binds.iter().map(|b| b.as_ref())),
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // 分页数据
    let data_sql = format!(
        "SELECT id, request_id, app_type, provider_id, provider_name, model,
                input_tokens, output_tokens, total_cost_usd, latency_ms, first_token_ms,
                status_code, error_message, is_streaming, created_at
         FROM proxy_request_logs {where_clause}
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?"
    );

    let mut all_binds: Vec<Box<dyn rusqlite::ToSql>> = binds;
    all_binds.push(Box::new(page_size as i64));
    all_binds.push(Box::new(offset as i64));

    let mut stmt = conn.prepare(&data_sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(
            params_from_iter(all_binds.iter().map(|b| b.as_ref())),
            |row| {
                Ok(RequestLogDetail {
                    id: row.get(0)?,
                    request_id: row.get(1)?,
                    app_type: row.get(2)?,
                    provider_id: row.get(3)?,
                    provider_name: row.get(4)?,
                    model: row.get(5)?,
                    input_tokens: row.get::<_, i64>(6)? as u64,
                    output_tokens: row.get::<_, i64>(7)? as u64,
                    total_cost_usd: row.get(8)?,
                    latency_ms: row.get::<_, i64>(9)? as u64,
                    first_token_ms: row.get::<_, Option<i64>>(10)?.map(|v| v as u64),
                    status_code: row.get::<_, i64>(11)? as u16,
                    error_message: row.get(12)?,
                    is_streaming: row.get::<_, i64>(13)? != 0,
                    created_at: row.get(14)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let mut data = Vec::new();
    for r in rows {
        data.push(r.map_err(|e| e.to_string())?);
    }

    Ok(PaginatedLogs {
        data,
        total,
        page,
        page_size,
    })
}

/// 清空全部日志
pub fn clear_logs(db: &Database) -> Result<(), String> {
    let conn = crate::lock_conn!(db.conn);
    conn.execute("DELETE FROM proxy_request_logs", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::usage::logger;
    use crate::usage::types::RequestLog;

    fn sample_log(status: u16, latency: u64, in_tok: u64, out_tok: u64) -> RequestLog {
        RequestLog {
            request_id: format!("req-{}", status),
            app_type: "claude".to_string(),
            provider_id: Some("p1".to_string()),
            provider_name: Some("TestProvider".to_string()),
            model: Some("sonnet".to_string()),
            request_model: None,
            input_tokens: in_tok,
            output_tokens: out_tok,
            total_cost_usd: 0.0,
            latency_ms: latency,
            first_token_ms: None,
            status_code: status,
            error_message: None,
            is_streaming: false,
            created_at: 1000,
        }
    }

    #[test]
    fn test_log_and_summary() {
        let db = Database::in_memory().unwrap();

        // 写入 3 条: 2 成功 1 失败
        logger::log_request(&db, &sample_log(200, 100, 10, 20)).unwrap();
        logger::log_request(&db, &sample_log(200, 200, 30, 40)).unwrap();
        logger::log_request(&db, &sample_log(500, 300, 0, 0)).unwrap();

        let summary = get_summary(&db, None, None).unwrap();
        assert_eq!(summary.total_requests, 3);
        assert_eq!(summary.success_count, 2);
        assert_eq!(summary.error_count, 1);
        // 成功率 2/3
        assert!((summary.success_rate - 0.6667).abs() < 0.01);
        // 平均延迟 (100+200+300)/3 = 200
        assert_eq!(summary.avg_latency_ms, 200);
        assert_eq!(summary.total_input_tokens, 40);
        assert_eq!(summary.total_output_tokens, 60);
    }

    #[test]
    fn test_get_logs_pagination() {
        let db = Database::in_memory().unwrap();
        for i in 0..5 {
            let mut log = sample_log(200, 100, 10, 20);
            log.request_id = format!("req-{i}");
            log.created_at = 1000 + i;
            logger::log_request(&db, &log).unwrap();
        }

        let page1 = get_logs(&db, LogFilters::default(), 1, 2).unwrap();
        assert_eq!(page1.total, 5);
        assert_eq!(page1.data.len(), 2);
        // 应按 created_at DESC, 即最新 (1004) 在前
        assert_eq!(page1.data[0].request_id, "req-4");
        assert_eq!(page1.page, 1);
        assert_eq!(page1.page_size, 2);
    }

    #[test]
    fn test_clear_logs() {
        let db = Database::in_memory().unwrap();
        logger::log_request(&db, &sample_log(200, 100, 10, 20)).unwrap();
        assert_eq!(get_summary(&db, None, None).unwrap().total_requests, 1);
        clear_logs(&db).unwrap();
        assert_eq!(get_summary(&db, None, None).unwrap().total_requests, 0);
    }

    #[test]
    fn test_today_summary_empty() {
        let db = Database::in_memory().unwrap();
        let (reqs, errs, lat) = today_summary(&db).unwrap();
        assert_eq!(reqs, 0);
        assert_eq!(errs, 0);
        assert_eq!(lat, 0);
    }
}
