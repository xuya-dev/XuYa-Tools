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

use tokio::sync::RwLock;

use crate::cli_config::types::AppType;
use crate::db::Database;

use server::{ProxyServer, ProxyState, UpstreamTarget};
use types::{ProxyServerInfo, ProxyStatus, TakeoverResult};

/// 接管状态记录 (内存, Phase 2 MVP)
#[derive(Default, Clone)]
struct TakeoverState {
    claude_backup: Option<PathBuf>,
    codex_backup: Option<PathBuf>,
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
        let mut state = ProxyState::default();
        state.db = db.clone();
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
        *self.state.app_handle.write().unwrap() = Some(handle);
    }

    /// 启动代理。若已设置上游则自动生效。
    pub async fn start(&self) -> Result<ProxyServerInfo, String> {
        let server = ProxyServer::new(self.state.clone());
        let info = server.start("127.0.0.1", 0).await?;
        *self.server.write().await = Some(server);
        Ok(info)
    }

    pub async fn stop(&self) -> Result<(), String> {
        // 停止前恢复所有接管
        self.restore_all_takeovers().await;

        if let Some(server) = self.server.write().await.take() {
            server.stop().await?;
        }
        Ok(())
    }

    /// 设置/切换代理的上游 provider (前端显式传入)
    pub async fn set_target(
        &self,
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
        *self.state.target.write().await = Some(target);
        let mut status = self.state.status.write().await;
        status.active_provider_id = Some(provider_id);
        status.active_provider_name = Some(provider_name);
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

        // 先备份当前 live config
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
                    tk.codex_backup = Some(backup_path);
                    self.state.status.write().await.codex_taken_over = true;
                }
            }
        }
        result
    }

    async fn disable_takeover(&self, app: AppType) -> TakeoverResult {
        let backup = {
            let tk = self.takeover.read().await;
            match app {
                AppType::Claude => tk.claude_backup.clone(),
                AppType::Codex => tk.codex_backup.clone(),
            }
        };

        let result = match (&backup, app) {
            (Some(p), AppType::Claude) => takeover::restore_claude(p),
            (Some(p), AppType::Codex) => takeover::restore_codex(p),
            (None, _) => TakeoverResult {
                success: false,
                message: "没有备份记录, 无法恢复".into(),
            },
        };

        if result.success {
            let mut tk = self.takeover.write().await;
            match app {
                AppType::Claude => {
                    tk.claude_backup = None;
                    self.state.status.write().await.claude_taken_over = false;
                }
                AppType::Codex => {
                    tk.codex_backup = None;
                    self.state.status.write().await.codex_taken_over = false;
                }
            }
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
