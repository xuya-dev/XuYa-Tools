//! SQLite 数据库模块
//!
//! Phase 3: 存储代理请求日志。
//! 用 rusqlite + bundled, 单文件数据库, 线程安全的 Mutex<Connection>。

pub mod schema;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

/// 线程安全的数据库句柄
#[derive(Clone)]
pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// 打开/创建数据库, 自动执行 schema 迁移
    pub fn open(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(&path).map_err(|e| e.to_string())?;
        // 提升 WAL 性能
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| e.to_string())?;
        schema::migrate(&conn).map_err(|e| e.to_string())?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 内存数据库 (供测试)
    #[allow(dead_code)]
    pub fn in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        schema::migrate(&conn).map_err(|e| e.to_string())?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

/// 获取锁的辅助宏 (复制自 cc-switch 的模式)
/// 说明:锁被毒化 (持有线程 panic) 时仍取出内部数据,避免单点 panic 扩散;
/// 此时数据可能不一致,但日志功能降级运行优于整体崩溃。
#[macro_export]
macro_rules! lock_conn {
    ($conn:expr) => {
        $conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_db_schema_migration() {
        let db = Database::in_memory().expect("打开内存数据库失败");
        // schema_version 表应存在且版本=2
        let conn = db.conn.lock().unwrap();
        let version: u32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, 2, "schema 版本应为 2");
        // proxy_request_logs 表应存在
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM proxy_request_logs", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "新建表应为空");
    }
}
