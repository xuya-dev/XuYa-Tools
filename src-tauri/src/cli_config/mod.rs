//! CLI 配置切换模块入口
//!
//! 提供 Claude Code / Codex 的配置识别、provider 管理、一键切换。
//! live config 是「投影」，应用内 provider 才是源数据 (SSOT)。

pub mod backup;
pub mod balance;
pub mod claude;
pub mod codex;
pub mod model_fetch;
pub mod store;
pub mod types;

use std::path::PathBuf;

use store::ProviderStore;
use types::{AppType, CliProvider, CliStatus, SwitchResult};

/// CLI 配置服务，封装 store + 各 app 适配器
pub struct CliConfigService {
    store: ProviderStore,
    backup_dir: PathBuf,
}

impl CliConfigService {
    pub fn new(data_dir: PathBuf) -> Self {
        let backup_dir = data_dir.join("backups");
        Self {
            store: ProviderStore::new(data_dir),
            backup_dir,
        }
    }

    pub fn list_providers(&self) -> Vec<CliProvider> {
        self.store.list()
    }

    /// 按 id 查询单个 provider (供代理层使用)
    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Option<CliProvider> {
        self.store.get(id)
    }

    pub fn save_provider(&self, provider: CliProvider) -> CliProvider {
        self.store.upsert(provider)
    }

    pub fn delete_provider(&self, id: &str) -> bool {
        self.store.delete(id)
    }

    /// 探测 Claude / Codex 当前 live 状态，并尝试匹配到已保存的 provider
    pub fn detect_status(&self) -> CliStatus {
        let mut claude_status = claude::detect();
        let mut codex_status = codex::detect();

        for p in self.store.list() {
            // 匹配规则: baseUrl 完全相同，或 api_key 非空且相同
            if claude_status.base_url == p.base_url && !p.base_url.is_empty() {
                claude_status.matched_provider_id = Some(p.id.clone());
                claude_status.matched_provider_name = Some(p.name.clone());
            }
            if codex_status.base_url == p.base_url && !p.base_url.is_empty() {
                codex_status.matched_provider_id = Some(p.id.clone());
                codex_status.matched_provider_name = Some(p.name.clone());
            }
        }

        CliStatus {
            claude: claude_status,
            codex: codex_status,
        }
    }

    /// 读取指定 app 的 live 配置文件内容 (用于前端预览)。
    /// 返回 (path, content); 文件不存在时 content 为空字符串。
    pub fn read_live_config(&self, app: AppType) -> (String, String) {
        let (path, content) = match app {
            AppType::Claude => {
                let p = claude::claude_settings_path();
                let c = std::fs::read_to_string(&p).unwrap_or_default();
                (p.to_string_lossy().to_string(), c)
            }
            AppType::Codex => {
                // Codex 由两个文件组成, 这里拼装成可读形式
                let auth = codex::codex_auth_path();
                let cfg = codex::codex_config_path();
                let mut out = String::new();
                if auth.exists() {
                    out.push_str(&format!(
                        "// ===== {} =====\n",
                        auth.file_name().unwrap_or_default().to_string_lossy()
                    ));
                    out.push_str(&std::fs::read_to_string(&auth).unwrap_or_default());
                    out.push_str("\n\n");
                }
                if cfg.exists() {
                    out.push_str(&format!(
                        "// ===== {} =====\n",
                        cfg.file_name().unwrap_or_default().to_string_lossy()
                    ));
                    out.push_str(&std::fs::read_to_string(&cfg).unwrap_or_default());
                }
                (cfg.to_string_lossy().to_string(), out)
            }
        };
        (path, content)
    }

    /// 切换某 app 到指定 provider。
    /// 流程: backfill 旧 provider -> 备份 live config -> 写入新配置 -> 记录 current。
    pub fn switch(&self, app: AppType, provider_id: &str) -> SwitchResult {
        let Some(provider) = self.store.get(provider_id) else {
            return SwitchResult {
                success: false,
                message: format!("未找到 provider: {provider_id}"),
                backup_path: String::new(),
            };
        };

        if !provider.applies_to(app) {
            return SwitchResult {
                success: false,
                message: format!("该 provider 不适用于 {}", app.as_str()),
                backup_path: String::new(),
            };
        }

        // 0. Backfill: 把当前 live 配置回写到「旧当前 provider」, 避免切走丢失用户手改值
        if let Some(old_id) = self.store.current_id(app) {
            if old_id != provider_id {
                self.backfill_live_to_provider(app, &old_id);
            }
        }

        // 1. 备份 live config
        let backup_path = match app {
            AppType::Claude => backup::create_backup(
                &claude::claude_settings_path(),
                &self.backup_dir.join("claude"),
            ),
            AppType::Codex => {
                backup::create_backup(&codex::codex_auth_path(), &self.backup_dir.join("codex"))
            }
        };
        let backup_str = match &backup_path {
            Ok(Some(p)) => p.to_string_lossy().to_string(),
            _ => String::new(),
        };

        // 2. 写入 live config
        let write_result = match app {
            AppType::Claude => claude::write_live(&provider),
            AppType::Codex => codex::write_live(&provider),
        };

        if let Err(e) = write_result {
            return SwitchResult {
                success: false,
                message: format!("写入配置失败: {e}"),
                backup_path: backup_str,
            };
        }

        // 3. 记录 current
        self.store.set_current(app, Some(provider_id.to_string()));

        SwitchResult {
            success: true,
            message: format!("已切换 {} 到 {}", app.as_str(), provider.name),
            backup_path: backup_str,
        }
    }

    /// 仅更新 current provider 记录, 不写 live config。
    /// 接管模式下使用: live config 始终指向代理地址, 只改代理内部的上游指向。
    pub fn set_current_only(&self, app: AppType, provider_id: &str) {
        self.store.set_current(app, Some(provider_id.to_string()));
    }

    #[allow(dead_code)]
    pub fn current_provider(&self, app: AppType) -> Option<CliProvider> {
        let id = self.store.current_id(app)?;
        self.store.get(&id)
    }

    /// Backfill: 读取当前 live 配置, 把用户可能在 live 文件里手改的值回写到指定 provider。
    /// 仅回写可探测的 base_url / model, api_key 出于安全不回读 (live 里可能是明文/占位)。
    /// 这样用户切走再切回时, 保留手改而非被旧 provider 值覆盖。
    fn backfill_live_to_provider(&self, app: AppType, provider_id: &str) {
        let Some(mut p) = self.store.get(provider_id) else {
            return;
        };
        let live = match app {
            AppType::Claude => claude::detect(),
            AppType::Codex => codex::detect(),
        };
        let mut changed = false;
        // base_url: live 探测到的可能与 provider 记录的不同 (用户手改了端点)
        if !live.base_url.is_empty() && live.base_url != p.base_url {
            p.base_url = live.base_url.clone();
            changed = true;
        }
        // model: 仅 Claude 的 detect 能读到 ANTHROPIC_MODEL
        if !live.model.is_empty() && live.model != p.model {
            p.model = live.model.clone();
            changed = true;
        }
        if changed {
            p.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(p.updated_at);
            self.store.upsert(p);
        }
    }
}
