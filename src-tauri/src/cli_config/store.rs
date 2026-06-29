//! CLI 配置切换 - JSON 存储
//!
//! provider 列表与当前激活状态持久化到应用数据目录下的 JSON 文件。
//! Phase 1 不引入 SQLite，用 JSON 即可承载配置切换的少量数据。

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::types::{AppType, CliProvider};

/// 持久化结构: providers 数组 + 每个 app 的当前 provider id
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoreData {
    #[serde(default)]
    pub providers: Vec<CliProvider>,
    #[serde(default)]
    pub current_claude: Option<String>,
    #[serde(default)]
    pub current_codex: Option<String>,
}

/// 进程内单例存储，封装文件读写与锁
pub struct ProviderStore {
    data: Mutex<StoreData>,
    path: PathBuf,
}

impl ProviderStore {
    pub fn new(data_dir: PathBuf) -> Self {
        fs::create_dir_all(&data_dir).ok();
        let path = data_dir.join("cli_providers.json");
        let data = load_file(&path).unwrap_or_default();
        Self {
            data: Mutex::new(data),
            path,
        }
    }

    pub fn list(&self) -> Vec<CliProvider> {
        self.data
            .lock()
            .expect("ProviderStore 数据锁被毒化")
            .providers
            .clone()
    }

    #[allow(dead_code)]
    pub fn current_id(&self, app: AppType) -> Option<String> {
        let data = self.data.lock().expect("ProviderStore 数据锁被毒化");
        match app {
            AppType::Claude => data.current_claude.clone(),
            AppType::Codex => data.current_codex.clone(),
        }
    }

    pub fn get(&self, id: &str) -> Option<CliProvider> {
        self.data
            .lock()
            .expect("ProviderStore 数据锁被毒化")
            .providers
            .iter()
            .find(|p| p.id == id)
            .cloned()
    }

    /// 保存或更新 provider。返回保存后的 provider (含 id/时间戳)
    pub fn upsert(&self, mut provider: CliProvider) -> CliProvider {
        if provider.id.is_empty() {
            provider.id = uuid::Uuid::new_v4().to_string();
        }
        provider.updated_at = now_secs();
        let mut data = self.data.lock().expect("ProviderStore 数据锁被毒化");
        if let Some(existing) = data.providers.iter_mut().find(|p| p.id == provider.id) {
            *existing = provider.clone();
        } else {
            data.providers.push(provider.clone());
        }
        let _ = persist_file(&self.path, &data);
        provider
    }

    pub fn delete(&self, id: &str) -> bool {
        let mut data = self.data.lock().expect("ProviderStore 数据锁被毒化");
        let before = data.providers.len();
        data.providers.retain(|p| p.id != id);
        let changed = data.providers.len() != before;
        // 清理已被删除的 current 引用
        if data.current_claude.as_deref() == Some(id) {
            data.current_claude = None;
        }
        if data.current_codex.as_deref() == Some(id) {
            data.current_codex = None;
        }
        if changed {
            let _ = persist_file(&self.path, &data);
        }
        changed
    }

    /// 设置某 app 的当前 provider。传 None 表示清除。
    pub fn set_current(&self, app: AppType, id: Option<String>) {
        let mut data = self.data.lock().expect("ProviderStore 数据锁被毒化");
        match app {
            AppType::Claude => data.current_claude = id,
            AppType::Codex => data.current_codex = id,
        }
        let _ = persist_file(&self.path, &data);
    }
}

fn load_file(path: &PathBuf) -> Option<StoreData> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn persist_file(path: &PathBuf, data: &StoreData) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(data)?;
    // 原子写: 先写临时文件再 rename
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_config::types::{
        ApiFormat, AppType, AuthField, CliProvider, ProviderCategory, ProviderKind, ProviderScope,
    };

    fn tmp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xuya-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_provider(name: &str, scope: ProviderScope) -> CliProvider {
        CliProvider {
            id: String::new(),
            name: name.to_string(),
            scope,
            kind: ProviderKind::Relay,
            category: ProviderCategory::ThirdParty,
            base_url: "https://api.example.com".to_string(),
            api_key: "sk-test".to_string(),
            model: String::new(),
            model_sonnet: String::new(),
            model_haiku: String::new(),
            model_opus: String::new(),
            sonnet_name: String::new(),
            opus_name: String::new(),
            haiku_name: String::new(),
            sonnet_1m: false,
            opus_1m: false,
            haiku_1m: false,
            note: String::new(),
            website_url: String::new(),
            auth_field: AuthField::AnthropicAuthToken,
            api_format: ApiFormat::Anthropic,
            custom_user_agent: String::new(),
            models_url: String::new(),
            preset_id: String::new(),
            icon: String::new(),
            icon_color: String::new(),
            codex_auth_json: String::new(),
            codex_config_toml: String::new(),
            updated_at: 0,
        }
    }

    #[test]
    fn test_store_crud() {
        let dir = tmp_dir();
        let store = ProviderStore::new(dir.clone());

        // 初始为空
        assert!(store.list().is_empty());

        // upsert
        let p1 = store.upsert(sample_provider("A", ProviderScope::Claude));
        assert!(!p1.id.is_empty());
        assert_eq!(store.list().len(), 1);

        // get
        assert_eq!(store.get(&p1.id).unwrap().name, "A");

        // set_current
        store.set_current(AppType::Claude, Some(p1.id.clone()));
        assert_eq!(store.current_id(AppType::Claude), Some(p1.id.clone()));

        // upsert 更新同名 id
        let mut p1_updated = p1.clone();
        p1_updated.name = "A2".to_string();
        store.upsert(p1_updated);
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.get(&p1.id).unwrap().name, "A2");

        // delete
        assert!(store.delete(&p1.id));
        assert!(store.list().is_empty());
        // 删除后 current 应被清空
        assert_eq!(store.current_id(AppType::Claude), None);

        // 清理
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_store_persistence_across_instances() {
        let dir = tmp_dir();
        {
            let store = ProviderStore::new(dir.clone());
            store.upsert(sample_provider("Persist", ProviderScope::Claude));
        }
        // 重新打开应能读到之前的数据
        let store2 = ProviderStore::new(dir.clone());
        assert_eq!(store2.list().len(), 1);
        assert_eq!(store2.list()[0].name, "Persist");
        std::fs::remove_dir_all(&dir).ok();
    }
}
