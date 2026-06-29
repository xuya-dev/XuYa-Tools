//! 使用统计聚合查询

use rusqlite::params_from_iter;

use crate::db::Database;

use super::types::{DailyStat, LogFilters, PaginatedLogs, RequestLogDetail, UsageSummary};

fn build_time_filter(
    start: Option<i64>,
    end: Option<i64>,
) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
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
    let clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    (clause, binds)
}

/// 起止时间内的汇总 (None 表示全部)
pub fn get_summary(
    db: &Database,
    start: Option<i64>,
    end: Option<i64>,
) -> Result<UsageSummary, String> {
    let conn = crate::lock_conn!(db.conn);
    let (where_clause, binds) = build_time_filter(start, end);

    let sql = format!(
        "SELECT
            COUNT(*) as total,
            COALESCE(SUM(CASE WHEN status_code >= 200 AND status_code < 400 THEN 1 ELSE 0 END), 0) as ok,
            COALESCE(SUM(CASE WHEN status_code >= 400 OR status_code = 0 THEN 1 ELSE 0 END), 0) as err,
            COALESCE(AVG(latency_ms), 0) as avg_lat,
            COALESCE(AVG(first_token_ms), 0) as avg_ft,
            COALESCE(SUM(is_streaming), 0) as streams,
            COALESCE(SUM(input_tokens), 0) as in_tok,
            COALESCE(SUM(output_tokens), 0) as out_tok,
            COALESCE(SUM(cache_read_tokens), 0) as cache_read,
            COALESCE(SUM(cache_creation_tokens), 0) as cache_create,
            COALESCE(SUM(total_cost_usd), 0) as cost
         FROM proxy_request_logs {where_clause}"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let row = stmt
        .query_row(params_from_iter(binds.iter().map(|b| b.as_ref())), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, i64>(9)?,
                row.get::<_, f64>(10)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let (total, ok, err, avg_lat, avg_ft, streams, in_tok, out_tok, cache_read, cache_create, cost) =
        row;
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
        avg_first_token_ms: avg_ft as u64,
        streaming_count: streams as u64,
        total_input_tokens: in_tok as u64,
        total_output_tokens: out_tok as u64,
        total_cache_read_tokens: cache_read as u64,
        total_cache_creation_tokens: cache_create as u64,
        total_cost_usd: cost,
    })
}

/// 每日聚合统计 (用于图表)
pub fn get_daily_stats(
    db: &Database,
    start: Option<i64>,
    end: Option<i64>,
) -> Result<Vec<DailyStat>, String> {
    let conn = crate::lock_conn!(db.conn);
    let (where_clause, binds) = build_time_filter(start, end);

    // SQLite: date(created_at, 'unixepoch', 'localtime') 把 Unix 秒转本地日期
    let sql = format!(
        "SELECT
            date(created_at, 'unixepoch', 'localtime') as day,
            MIN(created_at - (created_at % 86400)) as ts,
            COUNT(*) as cnt,
            COALESCE(SUM(CASE WHEN status_code >= 200 AND status_code < 400 THEN 1 ELSE 0 END), 0) as ok,
            COALESCE(SUM(CASE WHEN status_code >= 400 OR status_code = 0 THEN 1 ELSE 0 END), 0) as err,
            COALESCE(SUM(input_tokens), 0) as in_tok,
            COALESCE(SUM(output_tokens), 0) as out_tok,
            COALESCE(SUM(cache_read_tokens), 0) as cache_read,
            COALESCE(SUM(cache_creation_tokens), 0) as cache_create,
            COALESCE(SUM(total_cost_usd), 0) as cost,
            COALESCE(AVG(latency_ms), 0) as avg_lat
         FROM proxy_request_logs {where_clause}
         GROUP BY day
         ORDER BY day"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(binds.iter().map(|b| b.as_ref())), |row| {
            let day: String = row.get(0)?;
            let ts: i64 = row.get(1).unwrap_or(0);
            // The localtime adjustment means ts might not be exact midnight;
            // normalize to the date's local midnight approximation
            Ok(DailyStat {
                date: day.clone(),
                timestamp: ts,
                request_count: row.get::<_, i64>(2)? as u64,
                success_count: row.get::<_, i64>(3)? as u64,
                error_count: row.get::<_, i64>(4)? as u64,
                input_tokens: row.get::<_, i64>(5)? as u64,
                output_tokens: row.get::<_, i64>(6)? as u64,
                cache_read_tokens: row.get::<_, i64>(7)? as u64,
                cache_creation_tokens: row.get::<_, i64>(8)? as u64,
                cost_usd: row.get(9)?,
                avg_latency_ms: row.get::<_, f64>(10)? as u64,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

/// 今日 0 点的 Unix 秒 (本地时区近似)
pub fn today_start_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let secs_per_day: i64 = 86400;
    now - (now.rem_euclid(secs_per_day))
}

/// N 天前 0 点的 Unix 秒
#[allow(dead_code)]
pub fn days_ago_start_secs(days: i64) -> i64 {
    today_start_secs() - days * 86400
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

    let total_sql = format!("SELECT COUNT(*) FROM proxy_request_logs {where_clause}");
    let total: u32 = conn
        .query_row(
            &total_sql,
            params_from_iter(binds.iter().map(|b| b.as_ref())),
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let data_sql = format!(
        "SELECT id, request_id, app_type, provider_id, provider_name, model,
                input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens,
                total_cost_usd, latency_ms, first_token_ms,
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
                    cache_read_tokens: row.get::<_, i64>(8)? as u64,
                    cache_creation_tokens: row.get::<_, i64>(9)? as u64,
                    total_cost_usd: row.get(10)?,
                    latency_ms: row.get::<_, i64>(11)? as u64,
                    first_token_ms: row.get::<_, Option<i64>>(12)?.map(|v| v as u64),
                    status_code: row.get::<_, i64>(13)? as u16,
                    error_message: row.get(14)?,
                    is_streaming: row.get::<_, i64>(15)? != 0,
                    created_at: row.get(16)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let mut data = Vec::new();
    for row in rows {
        data.push(row.map_err(|e| e.to_string())?);
    }

    Ok(PaginatedLogs {
        total,
        page,
        page_size,
        data,
    })
}

/// 清空日志
pub fn clear_logs(db: &Database) -> Result<(), String> {
    let conn = crate::lock_conn!(db.conn);
    conn.execute_batch("DELETE FROM proxy_request_logs")
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_summary_empty() {
        let db = Database::in_memory().expect("打开内存数据库失败");
        let summary = get_summary(&db, None, None).unwrap();
        assert_eq!(summary.total_requests, 0);
    }

    #[test]
    fn test_get_summary_with_data() {
        let db = Database::in_memory().expect("打开内存数据库失败");
        let conn = crate::lock_conn!(db.conn);
        // 插入两条测试数据
        conn.execute(
            "INSERT INTO proxy_request_logs (request_id, app_type, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, status_code, latency_ms, total_cost_usd, created_at)
             VALUES ('r1', 'claude', 100, 50, 80, 20, 200, 500, 0.01, 1000),
                    ('r2', 'claude', 200, 100, 0, 0, 500, 800, 0.02, 2000)",
            [],
        )
        .unwrap();
        drop(conn);

        let summary = get_summary(&db, None, None).unwrap();
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.success_count, 1);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.total_input_tokens, 300);
        assert_eq!(summary.total_output_tokens, 150);
        assert_eq!(summary.total_cache_read_tokens, 80);
        assert_eq!(summary.total_cache_creation_tokens, 20);
    }

    #[test]
    fn test_get_daily_stats() {
        let db = Database::in_memory().expect("打开内存数据库失败");
        let conn = crate::lock_conn!(db.conn);
        // 同一天两条, 另一天一条
        let day1 = 1700000000i64; // 某天
        let day2 = day1 + 86400; // 第二天
        conn.execute(
            "INSERT INTO proxy_request_logs (request_id, app_type, input_tokens, output_tokens, status_code, latency_ms, total_cost_usd, created_at)
             VALUES ('r1', 'claude', 100, 50, 200, 300, 0.01, ?1),
                    ('r2', 'claude', 200, 100, 200, 400, 0.02, ?2),
                    ('r3', 'codex', 50, 25, 200, 200, 0.005, ?1)",
            rusqlite::params![day1, day2],
        )
        .unwrap();
        drop(conn);

        let daily = get_daily_stats(&db, None, None).unwrap();
        assert_eq!(daily.len(), 2, "应该有两天的数据");
    }

    #[test]
    fn test_clear_logs() {
        let db = Database::in_memory().expect("打开内存数据库失败");
        let conn = crate::lock_conn!(db.conn);
        conn.execute(
            "INSERT INTO proxy_request_logs (request_id, app_type, input_tokens, output_tokens, status_code, latency_ms, total_cost_usd, created_at)
             VALUES ('r1', 'claude', 100, 50, 200, 300, 0.01, 1000)",
            [],
        )
        .unwrap();
        drop(conn);
        assert_eq!(get_summary(&db, None, None).unwrap().total_requests, 1);
        clear_logs(&db).unwrap();
        assert_eq!(get_summary(&db, None, None).unwrap().total_requests, 0);
    }
}
