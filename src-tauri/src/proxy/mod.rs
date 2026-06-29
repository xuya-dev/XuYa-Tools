//! 代理服务模块入口
//!
//! 管理: 服务器生命周期 / 当前上游 provider / 各 app 接管状态。
//!
//! 设计: ProxyService 不直接持有 CliConfigService 实例, 而是接收
//! 显式的上游信息 (provider_id / base_url / api_key)。两个 service 解耦,
//! 前端在切换上游时把数据传进来。

pub mod server;
pub mod takeover;
pub mod transform;
pub mod types;

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::cli_config::types::AppType;
use crate::cli_config::CliConfigService;
use crate::db::Database;

use server::{ProxyServer, ProxyState, UpstreamTarget};
use types::{ProxyServerInfo, ProxyStatus, TakeoverResult};

/// 持久化的自启动配置 — 文件存在即代表代理上次在运行,重启后应自动恢复。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AutoStartConfig {
    claude_backup: Option<String>,
    codex_auth_backup: Option<String>,
    codex_config_backup: Option<String>,
}

/// 接管状态记录
#[derive(Default, Clone)]
struct TakeoverState {
    claude_backup: Option<PathBuf>,
    codex_auth_backup: Option<PathBuf>,
    codex_config_backup: Option<PathBuf>,
}

/// 代理服务
pub struct ProxyService {
    server: Arc<RwLock<Option<ProxyServer>>>,
    state: ProxyState,
    takeover: Arc<RwLock<TakeoverState>>,
    data_dir: PathBuf,
    db: Option<Database>,
}

impl ProxyService {
    pub fn new(data_dir: PathBuf) -> Self {
        // 尝试打开数据库, 失败则降级为无日志模式
        let db_path = data_dir.join("xuya_proxy_logs.db");
        let db = Database::open(db_path).ok();
        let state = ProxyState {
            db: db.clone(),
            ..Default::default()
        };
        Self {
            server: Arc::new(RwLock::new(None)),
            state,
            takeover: Arc::new(RwLock::new(TakeoverState::default())),
            data_dir,
            db,
        }
    }

    /// 获取数据库引用 (供 widget snapshot 查询今日统计)
    pub fn db(&self) -> Option<&Database> {
        self.db.as_ref()
    }

    /// 注入 AppHandle, 用于 emit 告警事件 (在 setup 阶段调用)
    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        *self
            .state
            .app_handle
            .write()
            .expect("app_handle 写锁被毒化,无法注入 handle") = Some(handle);
    }

    // ==================== 自启动持久化 ====================

    fn autostart_path(&self) -> PathBuf {
        self.data_dir.join("proxy_autostart.json")
    }

    /// 把当前接管状态写入持久化文件 (代理运行中调用)
    async fn save_autostart(&self) {
        let tk = self.takeover.read().await;
        let config = AutoStartConfig {
            claude_backup: tk
                .claude_backup
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            codex_auth_backup: tk
                .codex_auth_backup
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            codex_config_backup: tk
                .codex_config_backup
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
        };
        drop(tk);
        let path = self.autostart_path();
        if let Ok(json) = serde_json::to_string_pretty(&config) {
            let _ = std::fs::write(&path, json);
        }
    }

    /// 删除持久化文件 (代理停止时调用)
    fn clear_autostart(&self) {
        let _ = std::fs::remove_file(self.autostart_path());
    }

    /// 应用重启后自动恢复代理: 启动 + 恢复接管 + 同步上游
    pub async fn restore_on_startup(&self, cli: &CliConfigService) {
        let config_path = self.autostart_path();
        let config: AutoStartConfig = match std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
        {
            Some(c) => c,
            None => return, // 不存在或解析失败 = 上次没开代理
        };

        // 1. 启动代理
        if self.start().await.is_err() {
            return;
        }

        let port = self.state.status.read().await.port;
        let proxy_url = format!("http://127.0.0.1:{port}");

        // 2. 恢复接管状态 (直接改 live URL 到新端口, 不重新备份)
        {
            let mut tk = self.takeover.write().await;
            // Claude
            if let Some(backup_str) = &config.claude_backup {
                let backup_path = PathBuf::from(backup_str);
                if backup_path.exists() {
                    let _ = takeover::takeover_claude(&proxy_url);
                    tk.claude_backup = Some(backup_path);
                    self.state.status.write().await.claude_taken_over = true;
                }
            }
            // Codex (需要 auth + config 两个备份路径)
            let codex_auth_ok = config.codex_auth_backup.as_ref().map(PathBuf::from);
            let codex_cfg_ok = config.codex_config_backup.as_ref().map(PathBuf::from);
            if let (Some(auth_bak), Some(cfg_bak)) = (&codex_auth_ok, &codex_cfg_ok) {
                if auth_bak.exists() && cfg_bak.exists() {
                    let _ = takeover::takeover_codex(&proxy_url);
                    tk.codex_auth_backup = Some(auth_bak.clone());
                    tk.codex_config_backup = Some(cfg_bak.clone());
                    self.state.status.write().await.codex_taken_over = true;
                }
            }
        }

        // 3. 同步两个 app 的代理上游 = 各自当前 provider
        for app in [AppType::Claude, AppType::Codex] {
            if let Some(p) = cli.current_provider(app) {
                if !p.base_url.is_empty() {
                    let _ = self
                        .set_target(
                            app,
                            p.id,
                            p.name,
                            p.base_url,
                            p.api_key,
                            p.api_format.as_str().to_string(),
                        )
                        .await;
                }
            }
        }

        // 4. 持久化最新状态 (端口变了但备份路径不变)
        self.save_autostart().await;
    }

    /// 启动代理。若已设置上游则自动生效。
    pub async fn start(&self) -> Result<ProxyServerInfo, String> {
        let server = ProxyServer::new(self.state.clone());
        let info = server.start("127.0.0.1", 0).await?;
        *self.server.write().await = Some(server);
        // 持久化: 文件存在 = 下次重启自动启动
        self.save_autostart().await;
        Ok(info)
    }

    pub async fn stop(&self) -> Result<(), String> {
        // 停止前恢复所有接管
        self.restore_all_takeovers().await;

        if let Some(server) = self.server.write().await.take() {
            server.stop().await?;
        }
        // 清除持久化文件
        self.clear_autostart();
        Ok(())
    }

    /// 设置/切换某 app 的代理上游 provider
    pub async fn set_target(
        &self,
        app: AppType,
        provider_id: String,
        provider_name: String,
        base_url: String,
        api_key: String,
        api_format: String,
    ) -> Result<(), String> {
        if base_url.is_empty() {
            return Err("该 Provider 没有 Base URL, 无法作为代理上游".into());
        }
        let target = UpstreamTarget {
            provider_id: provider_id.clone(),
            provider_name: provider_name.clone(),
            base_url,
            api_key,
            api_format,
        };
        self.state
            .targets
            .write()
            .await
            .insert(app.as_str().to_string(), target);
        let mut status = self.state.status.write().await;
        match app {
            AppType::Claude => {
                status.claude_provider_id = Some(provider_id);
                status.claude_provider_name = Some(provider_name);
            }
            AppType::Codex => {
                status.codex_provider_id = Some(provider_id);
                status.codex_provider_name = Some(provider_name);
            }
        }
        Ok(())
    }

    /// 为指定 app 启用/禁用接管
    pub async fn set_takeover(&self, app: AppType, enabled: bool) -> TakeoverResult {
        if enabled {
            self.enable_takeover(app).await
        } else {
            self.disable_takeover(app).await
        }
    }

    async fn enable_takeover(&self, app: AppType) -> TakeoverResult {
        // 确保代理在运行
        if !self.state.status.read().await.running {
            return TakeoverResult {
                success: false,
                message: "请先启动代理服务器".into(),
            };
        }
        let port = self.state.status.read().await.port;
        let proxy_url = format!("http://127.0.0.1:{port}");

        // 先备份当前 live config (Codex 需要备份 auth.json + config.toml 两个文件)
        let backup_dir = self.data_dir.join("backups").join(app.as_str());
        let backup = match app {
            AppType::Claude => crate::cli_config::backup::create_backup(
                &crate::cli_config::claude::claude_settings_path(),
                &backup_dir,
            ),
            AppType::Codex => crate::cli_config::backup::create_backup(
                &crate::cli_config::codex::codex_auth_path(),
                &backup_dir,
            ),
        };
        let backup_path = match backup {
            Ok(Some(p)) => p,
            Ok(None) => PathBuf::new(),
            Err(e) => {
                return TakeoverResult {
                    success: false,
                    message: format!("备份失败: {e}"),
                }
            }
        };

        // Codex 还要备份 config.toml
        let codex_config_backup = if app == AppType::Codex {
            match crate::cli_config::backup::create_backup(
                &crate::cli_config::codex::codex_config_path(),
                &backup_dir,
            ) {
                Ok(Some(p)) => Some(p),
                _ => None,
            }
        } else {
            None
        };

        // 执行接管写入
        let result = match app {
            AppType::Claude => takeover::takeover_claude(&proxy_url),
            AppType::Codex => takeover::takeover_codex(&proxy_url),
        };

        if result.success {
            let mut tk = self.takeover.write().await;
            match app {
                AppType::Claude => {
                    tk.claude_backup = Some(backup_path);
                    self.state.status.write().await.claude_taken_over = true;
                }
                AppType::Codex => {
                    tk.codex_auth_backup = Some(backup_path);
                    tk.codex_config_backup = codex_config_backup;
                    self.state.status.write().await.codex_taken_over = true;
                }
            }
            drop(tk);
            self.save_autostart().await;
        }
        result
    }

    async fn disable_takeover(&self, app: AppType) -> TakeoverResult {
        let (claude_bak, codex_auth_bak, codex_cfg_bak) = {
            let tk = self.takeover.read().await;
            (
                tk.claude_backup.clone(),
                tk.codex_auth_backup.clone(),
                tk.codex_config_backup.clone(),
            )
        };

        let result = match app {
            AppType::Claude => match &claude_bak {
                Some(p) => takeover::restore_claude(p),
                None => TakeoverResult {
                    success: false,
                    message: "没有备份记录, 无法恢复".into(),
                },
            },
            AppType::Codex => match (&codex_auth_bak, &codex_cfg_bak) {
                (Some(auth), Some(cfg)) => takeover::restore_codex(auth, cfg),
                _ => TakeoverResult {
                    success: false,
                    message: "没有备份记录, 无法恢复".into(),
                },
            },
        };

        if result.success {
            let mut tk = self.takeover.write().await;
            let mut status = self.state.status.write().await;
            match app {
                AppType::Claude => {
                    tk.claude_backup = None;
                    status.claude_taken_over = false;
                    status.claude_provider_id = None;
                    status.claude_provider_name = None;
                }
                AppType::Codex => {
                    tk.codex_auth_backup = None;
                    tk.codex_config_backup = None;
                    status.codex_taken_over = false;
                    status.codex_provider_id = None;
                    status.codex_provider_name = None;
                }
            }
            drop(tk);
            drop(status);
            self.save_autostart().await;
        }
        result
    }

    async fn restore_all_takeovers(&self) {
        // 注意:必须先 clone 出标志并释放读锁, 再调用 disable_takeover。
        // 否则 disable_takeover 内部会请求 status 的写锁, 与此处持有的读锁形成死锁,
        // 导致 stop() 永久挂起 (反代无法关闭)。
        let (claude_on, codex_on) = {
            let s = self.state.status.read().await;
            (s.claude_taken_over, s.codex_taken_over)
        };
        if claude_on {
            let _ = self.disable_takeover(AppType::Claude).await;
        }
        if codex_on {
            let _ = self.disable_takeover(AppType::Codex).await;
        }
    }

    /// 查询当前状态
    pub async fn status(&self) -> ProxyStatus {
        self.state.status.read().await.clone()
    }

    #[allow(dead_code)]
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 回归测试 1: stop() 在接管标志置位时不应死锁。
    /// 此前 restore_all_takeovers 持有 status 读锁的同时调用
    /// disable_takeover (内部请求 status 写锁), 形成读→写死锁。
    #[tokio::test]
    async fn stop_does_not_deadlock_with_takeover_flag_set() {
        let dir =
            std::env::temp_dir().join(format!("xuya_test_stop_deadlock_{}", std::process::id()));
        let svc = ProxyService::new(dir.clone());

        {
            let mut s = svc.state.status.write().await;
            s.claude_taken_over = true;
        }

        let _info = svc.start().await.expect("start failed");

        let result = tokio::time::timeout(std::time::Duration::from_secs(5), svc.stop()).await;
        assert!(result.is_ok(), "stop() 死锁, 未在 5s 内返回");
        assert!(result.unwrap().is_ok(), "stop() 返回错误");
        assert!(!svc.status().await.running, "停止后 running 仍为 true");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 回归测试 2: stop() 后 status.running 必须为 false。
    /// 此前服务器任务被 abort, 其内部的 status 重置代码不会执行,
    /// 导致前端始终认为代理在运行 (反代无法关闭)。
    #[tokio::test]
    async fn stop_resets_running_status() {
        let dir =
            std::env::temp_dir().join(format!("xuya_test_stop_status_{}", std::process::id()));
        let svc = ProxyService::new(dir.clone());

        let info = svc.start().await.expect("start failed");
        assert!(svc.status().await.running, "启动后 running 应为 true");
        assert!(info.port > 0, "应分配到非零端口");

        svc.stop().await.expect("stop failed");

        let st = svc.status().await;
        assert!(!st.running, "停止后 running 必须为 false");
        assert_eq!(st.port, 0, "停止后端口应清零");
        assert_eq!(st.started_at, 0, "停止后 started_at 应清零");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
