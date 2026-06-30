//! CLI 配置切换 - 数据类型定义
//!
//! 定义 provider 模型和 live config 状态结构。
//! provider 是应用内的源数据 (SSOT)，live config 文件是其投影。

use serde::{Deserialize, Serialize};

/// 支持的 CLI 应用类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppType {
    Claude,
    Codex,
}

impl AppType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }
}

/// provider 适用范围
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderScope {
    /// 仅 Claude Code
    Claude,
    /// 仅 Codex
    Codex,
    /// 两者通用 (Claude Code 兼容 Anthropic 协议的 Codex 中转亦可)
    Both,
}

impl ProviderScope {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "codex" => Self::Codex,
            "both" => Self::Both,
            _ => Self::Claude,
        }
    }

    pub fn applies_to(&self, app: AppType) -> bool {
        match self {
            Self::Both => true,
            Self::Claude => app == AppType::Claude,
            Self::Codex => app == AppType::Codex,
        }
    }
}

/// provider 分类 (对齐 cc-switch 的 ProviderCategory)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCategory {
    /// 官方
    Official,
    /// 国内官方
    CnOfficial,
    /// 云厂商 (Bedrock 等)
    CloudProvider,
    /// 聚合中转
    Aggregator,
    /// 第三方
    ThirdParty,
    /// 自定义
    #[default]
    Custom,
}

/// Claude API 认证字段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthField {
    #[default]
    AnthropicAuthToken,
    AnthropicApiKey,
}

/// API 格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ApiFormat {
    /// Anthropic Messages 原生
    #[default]
    Anthropic,
    /// OpenAI Chat Completions (需转换)
    OpenaiChat,
    /// OpenAI Responses (需转换)
    OpenaiResponses,
    /// Gemini Native (需转换)
    GeminiNative,
}

impl ApiFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenaiChat => "openai_chat",
            Self::OpenaiResponses => "openai_responses",
            Self::GeminiNative => "gemini_native",
        }
    }
}

/// provider 配置 (应用内 SSOT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliProvider {
    /// 唯一 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 适用范围
    pub scope: ProviderScope,
    /// 来源类型 (保留兼容)
    pub kind: ProviderKind,
    /// 分类
    #[serde(default)]
    pub category: ProviderCategory,
    /// 上游 base URL (官方默认时可为空)
    #[serde(default)]
    pub base_url: String,
    /// API Key / Token (官方登录时可为空)
    #[serde(default)]
    pub api_key: String,
    /// 默认模型 (可为空，表示用 CLI 默认)
    #[serde(default)]
    pub model: String,
    /// Sonnet 角色模型 (Claude Code 便捷记忆, 写入时作为主模型优先候选)
    #[serde(default)]
    pub model_sonnet: String,
    /// Haiku 角色模型 (Claude Code 便捷记忆)
    #[serde(default)]
    pub model_haiku: String,
    /// Opus 角色模型 (Claude Code 便捷记忆)
    #[serde(default)]
    pub model_opus: String,
    /// Sonnet 角色显示名 (写入 ANTHROPIC_DEFAULT_SONNET_MODEL_NAME, 影响 Claude Code /model 菜单)
    #[serde(default)]
    pub sonnet_name: String,
    /// Opus 角色显示名
    #[serde(default)]
    pub opus_name: String,
    /// Haiku 角色显示名
    #[serde(default)]
    pub haiku_name: String,
    /// Sonnet 角色启用 1M 上下文 (写入时给模型名追加 [1M] 后缀)
    #[serde(default)]
    pub sonnet_1m: bool,
    /// Opus 角色启用 1M 上下文
    #[serde(default)]
    pub opus_1m: bool,
    /// Haiku 角色启用 1M 上下文
    #[serde(default)]
    pub haiku_1m: bool,
    /// 备注
    #[serde(default)]
    pub note: String,
    /// 官网链接
    #[serde(default)]
    pub website_url: String,
    /// Claude 认证字段名 (scope=claude/both 时生效)
    #[serde(default)]
    pub auth_field: AuthField,
    /// API 格式 (影响代理转换, Phase 2+)
    #[serde(default)]
    pub api_format: ApiFormat,
    /// 自定义 User-Agent (可选)
    #[serde(default)]
    pub custom_user_agent: String,
    /// 模型列表端点覆写 (可选, 解决部分供应商 /models 不在默认路径)
    #[serde(default)]
    pub models_url: String,
    /// 预设来源 id (可选, 标记从哪个预设创建)
    #[serde(default)]
    pub preset_id: String,
    /// 图标标识 (前端渲染用, 对齐 cc-switch icon)
    #[serde(default)]
    pub icon: String,
    /// 图标颜色 (hex, 对齐 cc-switch iconColor)
    #[serde(default)]
    pub icon_color: String,
    /// Codex auth.json 内容 (scope 含 codex 时生效)
    #[serde(default)]
    pub codex_auth_json: String,
    /// Codex config.toml 内容 (scope 含 codex 时生效)
    #[serde(default)]
    pub codex_config_toml: String,
    /// Claude settings.json 完整内容 (高级模式, 留空则自动生成)
    #[serde(default)]
    pub claude_settings_json: String,
    /// 余额查询类型: "newapi" | "sub2api" | "" (空=自动识别)
    #[serde(default)]
    pub quota_provider_type: String,
    /// NewAPI 访问令牌 (仅 newapi 类型)
    #[serde(default)]
    pub quota_access_token: String,
    /// NewAPI 用户 ID (仅 newapi 类型)
    #[serde(default)]
    pub quota_user_id: String,
    /// 最后更新时间 (Unix 秒)
    pub updated_at: i64,
}

impl CliProvider {
    pub fn applies_to(&self, app: AppType) -> bool {
        self.scope.applies_to(app)
    }
}

/// provider 来源类型 (保留兼容)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    /// 官方登录 (走 CLI 原生 OAuth)
    Official,
    /// 第三方中转
    #[default]
    Relay,
    /// 自定义
    Custom,
}

/// 某 CLI 当前的 live config 状态快照 (只读探测结果)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliLiveStatus {
    /// 配置文件是否存在
    pub installed: bool,
    /// 配置文件路径
    pub config_path: String,
    /// 当前 base URL (可能为空)
    pub base_url: String,
    /// 当前模型 (兜底, 可能为空)
    pub model: String,
    /// Sonnet 角色当前模型 (含可能的 [1M] 后缀)
    #[serde(default)]
    pub model_sonnet: String,
    /// Opus 角色当前模型
    #[serde(default)]
    pub model_opus: String,
    /// Haiku 角色当前模型
    #[serde(default)]
    pub model_haiku: String,
    /// 是否已被本应用接管 (Phase 2 才会变 true)
    pub taken_over: bool,
    /// 是否匹配到某个已保存 provider (匹配 key/baseUrl)
    pub matched_provider_id: Option<String>,
    pub matched_provider_name: Option<String>,
}

/// get_cli_status 返回的聚合状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliStatus {
    pub claude: CliLiveStatus,
    pub codex: CliLiveStatus,
}

/// 切换结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchResult {
    pub success: bool,
    pub message: String,
    /// 备份文件路径 (失败时为空)
    pub backup_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 回归测试:AuthField 必须以 SCREAMING_SNAKE_CASE 序列化,
    /// 与前端 useCliConfig.ts / providerPresets.ts 中 'ANTHROPIC_AUTH_TOKEN' / 'ANTHROPIC_API_KEY' 对齐。
    /// 此前用 UPPERCASE 导致 save_cli_provider 反序列化失败 (unknown variant)。
    #[test]
    fn auth_field_serializes_as_screaming_snake_case() {
        assert_eq!(
            serde_json::to_string(&AuthField::AnthropicAuthToken).unwrap(),
            "\"ANTHROPIC_AUTH_TOKEN\""
        );
        assert_eq!(
            serde_json::to_string(&AuthField::AnthropicApiKey).unwrap(),
            "\"ANTHROPIC_API_KEY\""
        );
        // 反序列化也必须接受前端发来的字面量
        let v: AuthField = serde_json::from_str("\"ANTHROPIC_AUTH_TOKEN\"").unwrap();
        assert_eq!(v, AuthField::AnthropicAuthToken);
        let v: AuthField = serde_json::from_str("\"ANTHROPIC_API_KEY\"").unwrap();
        assert_eq!(v, AuthField::AnthropicApiKey);
    }

    #[test]
    fn api_format_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&ApiFormat::Anthropic).unwrap(),
            "\"anthropic\""
        );
        assert_eq!(
            serde_json::to_string(&ApiFormat::OpenaiChat).unwrap(),
            "\"openai_chat\""
        );
        assert_eq!(
            serde_json::to_string(&ApiFormat::OpenaiResponses).unwrap(),
            "\"openai_responses\""
        );
        assert_eq!(
            serde_json::to_string(&ApiFormat::GeminiNative).unwrap(),
            "\"gemini_native\""
        );
    }

    #[test]
    fn provider_scope_serializes_as_lowercase() {
        assert_eq!(
            serde_json::to_string(&ProviderScope::Claude).unwrap(),
            "\"claude\""
        );
        assert_eq!(
            serde_json::to_string(&ProviderScope::Codex).unwrap(),
            "\"codex\""
        );
        assert_eq!(
            serde_json::to_string(&ProviderScope::Both).unwrap(),
            "\"both\""
        );
    }
}
