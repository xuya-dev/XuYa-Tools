//! CLI 配置切换 - live config 备份
//!
//! 切换前先把当前 live config 备份到应用数据目录，
//! 这样既能回滚，也能在后续 Phase 2 接管时恢复原始配置。

use std::fs;
use std::path::{Path, PathBuf};

/// 创建时间戳备份，返回备份文件完整路径。
/// 若源文件不存在则返回 Ok(None)。
pub fn create_backup(src: &Path, backup_dir: &Path) -> std::io::Result<Option<PathBuf>> {
    if !src.exists() {
        return Ok(None);
    }
    fs::create_dir_all(backup_dir)?;

    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("config");
    let ext = src.extension().and_then(|s| s.to_str()).unwrap_or("json");
    let ts = chrono_like_ts();
    let backup_name = format!("{stem}_{ts}.{ext}");
    let backup_path = backup_dir.join(backup_name);
    fs::copy(src, &backup_path)?;
    cleanup_old_backups(backup_dir, 10)?;
    Ok(Some(backup_path))
}

fn cleanup_old_backups(dir: &Path, retain: usize) -> std::io::Result<()> {
    if retain == 0 {
        return Ok(());
    }
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .collect();
    if entries.len() <= retain {
        return Ok(());
    }
    entries.sort_by(|a, b| {
        let ta = a.metadata().and_then(|m| m.modified()).ok();
        let tb = b.metadata().and_then(|m| m.modified()).ok();
        ta.cmp(&tb)
    });
    let remove_count = entries.len().saturating_sub(retain);
    for entry in entries.into_iter().take(remove_count) {
        let _ = fs::remove_file(entry.path());
    }
    Ok(())
}

/// 轻量时间戳，不引入额外依赖
fn chrono_like_ts() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", now)
}
