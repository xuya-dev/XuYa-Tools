//! 数据库 schema 定义与迁移

use rusqlite::Connection;

/// 当前 schema 版本
const SCHEMA_VERSION: u32 = 1;

/// 执行 schema 迁移 (幂等)
pub fn migrate(conn: &Connection) -> Result<(), rusqlite::Error> {
    // 版本表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        );",
    )?;

    let current: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current < 1 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS proxy_request_logs (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                request_id      TEXT NOT NULL,
                app_type        TEXT NOT NULL,
                provider_id     TEXT,
                provider_name   TEXT,
                model           TEXT,
                request_model   TEXT,
                input_tokens    INTEGER NOT NULL DEFAULT 0,
                output_tokens   INTEGER NOT NULL DEFAULT 0,
                total_cost_usd  REAL NOT NULL DEFAULT 0,
                latency_ms      INTEGER NOT NULL DEFAULT 0,
                first_token_ms  INTEGER,
                status_code     INTEGER NOT NULL DEFAULT 0,
                error_message   TEXT,
                is_streaming    INTEGER NOT NULL DEFAULT 0,
                created_at      INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_logs_created_at ON proxy_request_logs(created_at);
            CREATE INDEX IF NOT EXISTS idx_logs_provider  ON proxy_request_logs(provider_id);
            CREATE INDEX IF NOT EXISTS idx_logs_app       ON proxy_request_logs(app_type);
            CREATE INDEX IF NOT EXISTS idx_logs_status    ON proxy_request_logs(status_code);
            ",
        )?;
        conn.execute(
            "INSERT INTO schema_version(version) VALUES (?1)",
            rusqlite::params![1],
        )?;
    }

    let _ = SCHEMA_VERSION; // 预留未来版本管理
    Ok(())
}
