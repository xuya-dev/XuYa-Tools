//! 模型列表获取服务
//!
//! 通过 OpenAI 兼容的 GET /v1/models 端点获取供应商可用模型列表。
//! 主要面向第三方聚合站（硅基流动、OpenRouter 等），以及把 Anthropic
//! 协议挂在兼容子路径上的官方供应商（DeepSeek、Kimi、智谱 GLM 等）。
//!
//! 逻辑搬运自 cc-switch (src-tauri/src/services/model_fetch.rs)，精简去掉了
//! 对全局 http_client 单例和 proxy 的依赖，改为命令内自建 reqwest::Client。

use reqwest::header::HeaderValue;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 获取到的模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchedModel {
    pub id: String,
    pub owned_by: Option<String>,
}

/// OpenAI 兼容的 /v1/models 响应格式
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    #[serde(default)]
    data: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    id: String,
    #[serde(default)]
    owned_by: Option<String>,
}

const FETCH_TIMEOUT_SECS: u64 = 15;

/// 404/405 响应体截断长度：避免把几十 KB HTML 404 页整页保留到错误串里。
const ERROR_BODY_MAX_CHARS: usize = 512;

/// 已知的「Anthropic 协议兼容子路径」后缀；按长度降序，最长前缀优先匹配。
/// baseURL 命中这些后缀时，候选列表会追加「剥离后缀再拼 /v1/models / /models」的版本。
const KNOWN_COMPAT_SUFFIXES: &[&str] = &[
    "/api/claudecode",
    "/api/anthropic",
    "/apps/anthropic",
    "/api/coding",
    "/claudecode",
    "/anthropic",
    "/step_plan",
    "/coding",
    "/claude",
];

/// 获取供应商的可用模型列表
///
/// 使用 OpenAI 兼容的 GET /v1/models 端点，按候选列表顺序尝试。
/// `models_url_override` 非空时优先使用 (解决部分供应商 /models 不在默认路径)。
pub async fn fetch_models(
    base_url: &str,
    api_key: &str,
    custom_user_agent: Option<&str>,
    models_url_override: Option<&str>,
) -> Result<Vec<FetchedModel>, String> {
    if api_key.trim().is_empty() {
        return Err("API Key is required to fetch models".to_string());
    }

    let candidates = build_models_url_candidates(base_url, models_url_override)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("Build client failed: {e}"))?;

    // 可选自定义 UA (部分 /models 端点有 UA 白名单, 如 Kimi Coding Plan)
    let ua_header = custom_user_agent.and_then(|ua| HeaderValue::from_str(ua).ok());

    let mut last_err: Option<String> = None;
    for url in &candidates {
        let mut request = client
            .get(url)
            .header("Authorization", format!("Bearer {api_key}"));
        if let Some(ref ua) = ua_header {
            request = request.header(reqwest::header::USER_AGENT, ua);
        }
        let response = match request.send().await {
            Ok(r) => r,
            Err(e) => return Err(format!("Request failed: {e}")),
        };

        let status = response.status();
        if status.is_success() {
            let resp: ModelsResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {e}"))?;
            let mut models: Vec<FetchedModel> = resp
                .data
                .into_iter()
                .map(|m| FetchedModel {
                    id: m.id,
                    owned_by: m.owned_by,
                })
                .collect();
            models.sort_by(|a, b| a.id.cmp(&b.id));
            return Ok(models);
        }

        if status == StatusCode::NOT_FOUND || status == StatusCode::METHOD_NOT_ALLOWED {
            let body = truncate_body(response.text().await.unwrap_or_default());
            last_err = Some(format!("HTTP {status}: {body}"));
            continue;
        }

        let body = truncate_body(response.text().await.unwrap_or_default());
        return Err(format!("HTTP {status}: {body}"));
    }

    Err(format!(
        "All candidates failed: {}",
        last_err.unwrap_or_else(|| "no candidates".to_string())
    ))
}

/// 构造「模型列表端点」的候选 URL 列表
///
/// 候选顺序：
/// 1. baseURL 拼 `/v1/models`；若已以版本段 `/v{N}` 结尾（`/v1`、智谱
///    `/api/coding/paas/v4` 等），版本号已在路径里，改拼 `/models`
/// 2. 版本段非 `/v1`（如 `/v4`）时再追加 `/v1/models` 作为兜底次候选
/// 3. 若 baseURL 命中 [`KNOWN_COMPAT_SUFFIXES`]，剥离后缀再拼 `/v1/models`、`/models`
fn build_models_url_candidates(
    base_url: &str,
    models_url_override: Option<&str>,
) -> Result<Vec<String>, String> {
    // 1. 显式覆写优先
    if let Some(raw) = models_url_override {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(vec![trimmed.to_string()]);
        }
    }

    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("Base URL is empty".to_string());
    }

    let mut candidates: Vec<String> = Vec::new();

    // baseURL 已以版本段 /v{N} 结尾时（如 `/v1`、智谱 `/api/coding/paas/v4`），
    // OpenAI 惯例的模型端点是 `{base}/models`，不能再补 `/v1`。
    if ends_with_version_segment(trimmed) {
        candidates.push(format!("{trimmed}/models"));
        if !trimmed.ends_with("/v1") {
            candidates.push(format!("{trimmed}/v1/models"));
        }
    } else {
        candidates.push(format!("{trimmed}/v1/models"));
    }

    if let Some(stripped) = strip_compat_suffix(trimmed) {
        let root = stripped.trim_end_matches('/');
        if !root.is_empty() && root.contains("://") {
            candidates.push(format!("{root}/v1/models"));
            candidates.push(format!("{root}/models"));
        }
    }

    // 去重并保持首次出现顺序
    let mut unique: Vec<String> = Vec::with_capacity(candidates.len());
    for url in candidates {
        if !unique.iter().any(|u| u == &url) {
            unique.push(url);
        }
    }
    Ok(unique)
}

/// 判断 URL 是否以版本段 `/v{N}` 结尾
fn ends_with_version_segment(url: &str) -> bool {
    if let Some(idx) = url.rfind('/') {
        let last = &url[idx + 1..];
        return last.len() >= 2
            && last.starts_with('v')
            && last[1..].chars().all(|c| c.is_ascii_digit());
    }
    false
}

/// 若 url 以已知兼容子路径结尾，返回剥离后的根
fn strip_compat_suffix(url: &str) -> Option<&str> {
    for suffix in KNOWN_COMPAT_SUFFIXES {
        if url.ends_with(suffix) {
            return Some(&url[..url.len() - suffix.len()]);
        }
    }
    None
}

/// 截断响应体到 [`ERROR_BODY_MAX_CHARS`] 字符，避免 HTML 404 页占用错误串。
fn truncate_body(body: String) -> String {
    if body.chars().count() <= ERROR_BODY_MAX_CHARS {
        body
    } else {
        let mut s: String = body.chars().take(ERROR_BODY_MAX_CHARS).collect();
        s.push('…');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ends_with_version_segment() {
        assert!(ends_with_version_segment("https://api.example.com/v1"));
        assert!(ends_with_version_segment("https://api.example.com/v4"));
        assert!(ends_with_version_segment(
            "https://open.bigmodel.cn/api/paas/v4"
        ));
        assert!(!ends_with_version_segment("https://api.example.com"));
        assert!(!ends_with_version_segment("https://api.example.com/claude"));
    }

    #[test]
    fn test_strip_compat_suffix() {
        assert_eq!(
            strip_compat_suffix("https://api.example.com/anthropic"),
            Some("https://api.example.com")
        );
        assert_eq!(
            strip_compat_suffix("https://api.example.com/claudecode"),
            Some("https://api.example.com")
        );
        assert_eq!(strip_compat_suffix("https://api.example.com"), None);
    }

    #[test]
    fn test_build_candidates_plain_base() {
        let c = build_models_url_candidates("https://api.example.com", None).unwrap();
        assert_eq!(c[0], "https://api.example.com/v1/models");
    }

    #[test]
    fn test_build_candidates_v1_base() {
        let c = build_models_url_candidates("https://api.example.com/v1", None).unwrap();
        assert_eq!(c[0], "https://api.example.com/v1/models");
    }

    #[test]
    fn test_build_candidates_v4_base() {
        let c = build_models_url_candidates("https://open.bigmodel.cn/api/paas/v4", None).unwrap();
        // 版本段结尾 → /models 优先
        assert_eq!(c[0], "https://open.bigmodel.cn/api/paas/v4/models");
        // 非 /v1 → 追加 /v1/models 兜底
        assert!(c.iter().any(|u| u == "https://open.bigmodel.cn/api/paas/v4/v1/models"));
    }

    #[test]
    fn test_build_candidates_compat_suffix() {
        let c = build_models_url_candidates("https://api.example.com/anthropic", None).unwrap();
        // 直接拼 /v1/models
        assert!(c.iter().any(|u| u == "https://api.example.com/anthropic/v1/models"));
        // 剥离后缀再拼
        assert!(c.iter().any(|u| u == "https://api.example.com/v1/models"));
        assert!(c.iter().any(|u| u == "https://api.example.com/models"));
    }

    #[test]
    fn test_build_candidates_override() {
        // 显式覆写优先, 只返回它
        let c = build_models_url_candidates(
            "https://api.example.com",
            Some("https://api.example.com/internal/models"),
        )
        .unwrap();
        assert_eq!(c, vec!["https://api.example.com/internal/models".to_string()]);
    }

    #[test]
    fn test_build_candidates_empty() {
        assert!(build_models_url_candidates("", None).is_err());
        assert!(build_models_url_candidates("   ", None).is_err());
    }

    #[test]
    fn test_fetch_models_requires_api_key() {
        // 缺 API Key 时应直接返回错误，不发起请求
        let rt = tokio::runtime::Runtime::new().unwrap();
        let res = rt.block_on(fetch_models("https://api.example.com", "", None, None));
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("API Key"));
    }
}
