//! 余额 / 套餐额度查询
//!
//! 参考 cc-switch 的 balance.rs + coding_plan.rs, 精简为统一返回结构。
//! 支持 5 个国产供应商: DeepSeek / StepFun (余额制), Kimi / 智谱 / MiniMax (套餐制)。
//! 火山因需 AK/SK + V4 签名暂不支持。

use serde::{Deserialize, Serialize};
use std::time::Duration;

const TIMEOUT_SECS: u64 = 15;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceItem {
    pub label: String,
    pub remaining: Option<f64>,
    pub total: Option<f64>,
    pub used: Option<f64>,
    pub unit: String,
    pub is_valid: bool,
    pub invalid_message: Option<String>,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResult {
    pub success: bool,
    pub items: Vec<BalanceItem>,
    pub error: Option<String>,
    pub is_plan: bool,
}

impl BalanceResult {
    fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            items: vec![],
            error: Some(msg.into()),
            is_plan: false,
        }
    }

    fn unsupported() -> Self {
        Self::error("该供应商暂不支持余额查询")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Provider {
    DeepSeek,
    StepFun,
    Kimi,
    Zhipu,
    MiniMax,
}

fn detect_provider(base_url: &str) -> Option<Provider> {
    let url = base_url.to_lowercase();
    if url.contains("api.deepseek.com") {
        Some(Provider::DeepSeek)
    } else if url.contains("api.stepfun.") {
        Some(Provider::StepFun)
    } else if url.contains("api.kimi.com") || url.contains("kimi.com/coding") {
        Some(Provider::Kimi)
    } else if url.contains("bigmodel.cn") || url.contains("z.ai") {
        Some(Provider::Zhipu)
    } else if url.contains("minimaxi.com") || url.contains("minimax.io") {
        Some(Provider::MiniMax)
    } else {
        None
    }
}

/// 统一入口
pub async fn get_balance(base_url: &str, api_key: &str) -> BalanceResult {
    if api_key.is_empty() {
        return BalanceResult::error("API Key 为空");
    }
    let Some(provider) = detect_provider(base_url) else {
        return BalanceResult::unsupported();
    };
    match provider {
        Provider::DeepSeek => query_deepseek(api_key).await,
        Provider::StepFun => query_stepfun(api_key).await,
        Provider::Kimi => query_kimi(api_key).await,
        Provider::Zhipu => query_zhipu(base_url, api_key).await,
        Provider::MiniMax => query_minimax(base_url, api_key).await,
    }
}

// ==================== 工具函数 ====================

fn parse_f64(obj: &serde_json::Value, field: &str) -> Option<f64> {
    obj.get(field).and_then(|v| {
        v.as_f64()
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    })
}

fn auth_error_item(msg: &str) -> BalanceItem {
    BalanceItem {
        label: "API Key".into(),
        remaining: None,
        total: None,
        used: None,
        unit: String::new(),
        is_valid: false,
        invalid_message: Some(msg.into()),
        resets_at: None,
    }
}

async fn http_get(url: &str, auth_header: &str) -> Result<serde_json::Value, BalanceResult> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| BalanceResult::error(format!("HTTP 客户端构建失败: {e}")))?;

    let resp = client
        .get(url)
        .header("Authorization", auth_header)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| BalanceResult::error(format!("请求失败: {e}")))?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(BalanceResult {
            success: true,
            items: vec![auth_error_item("API Key 失效或无权限")],
            error: None,
            is_plan: false,
        });
    }
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        let truncated = text.chars().take(300).collect::<String>();
        return Err(BalanceResult::error(format!("HTTP {status}: {truncated}")));
    }

    resp.json::<serde_json::Value>()
        .await
        .map_err(|e| BalanceResult::error(format!("响应解析失败: {e}")))
}

// ==================== DeepSeek ====================

async fn query_deepseek(api_key: &str) -> BalanceResult {
    let auth = format!("Bearer {api_key}");
    let body = match http_get("https://api.deepseek.com/user/balance", &auth).await {
        Ok(v) => v,
        Err(e) => return e,
    };

    let mut items = Vec::new();
    if let Some(infos) = body.get("balance_infos").and_then(|v| v.as_array()) {
        for info in infos {
            let currency = info
                .get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("CNY");
            let total = parse_f64(info, "total_balance").unwrap_or(0.0);
            let granted = parse_f64(info, "granted_balance").unwrap_or(total);
            let topped = parse_f64(info, "topped_up_balance").unwrap_or(0.0);
            let is_available = info
                .get("is_available")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            items.push(BalanceItem {
                label: currency.to_string(),
                remaining: Some(total),
                total: Some(granted + topped),
                used: Some((granted + topped - total).max(0.0)),
                unit: currency.to_string(),
                is_valid: is_available,
                invalid_message: if !is_available {
                    Some("余额不足".into())
                } else {
                    None
                },
                resets_at: None,
            });
        }
    }

    if items.is_empty() {
        return BalanceResult::error("未能解析余额数据");
    }

    BalanceResult {
        success: true,
        items,
        error: None,
        is_plan: false,
    }
}

// ==================== StepFun ====================

async fn query_stepfun(api_key: &str) -> BalanceResult {
    let auth = format!("Bearer {api_key}");
    let body = match http_get("https://api.stepfun.com/v1/accounts", &auth).await {
        Ok(v) => v,
        Err(e) => return e,
    };

    let data = body.get("data").unwrap_or(&body);
    let balance = parse_f64(data, "balance").or_else(|| parse_f64(data, "total_balance"));

    BalanceResult {
        success: true,
        items: vec![BalanceItem {
            label: "余额".into(),
            remaining: balance,
            total: None,
            used: None,
            unit: "CNY".into(),
            is_valid: balance.map(|b| b > 0.0).unwrap_or(false),
            invalid_message: if balance.map(|b| b <= 0.0).unwrap_or(false) {
                Some("余额不足".into())
            } else {
                None
            },
            resets_at: None,
        }],
        error: None,
        is_plan: false,
    }
}

// ==================== Kimi (套餐制) ====================

async fn query_kimi(api_key: &str) -> BalanceResult {
    let auth = format!("Bearer {api_key}");
    let body = match http_get("https://api.kimi.com/coding/v1/usages", &auth).await {
        Ok(v) => v,
        Err(e) => return e,
    };

    let mut items = Vec::new();

    // limits[].detail → 5 小时窗口
    if let Some(limits) = body.get("limits").and_then(|v| v.as_array()) {
        for limit in limits {
            if let Some(detail) = limit.get("detail") {
                let lim = parse_f64(detail, "limit").unwrap_or(0.0);
                let rem = parse_f64(detail, "remaining").unwrap_or(0.0);
                let reset = detail
                    .get("resetTime")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                if lim > 0.0 {
                    let pct = ((lim - rem) / lim * 100.0).round();
                    items.push(BalanceItem {
                        label: "5 小时窗口".into(),
                        remaining: Some(rem),
                        total: Some(lim),
                        used: Some(lim - rem),
                        unit: "%".into(),
                        is_valid: true,
                        invalid_message: None,
                        resets_at: reset,
                    });
                    // 存百分比到 used 字段
                    items.last_mut().unwrap().used = Some(pct);
                }
            }
        }
    }

    // usage → 周窗口
    if let Some(usage) = body.get("usage") {
        let lim = parse_f64(usage, "limit").unwrap_or(0.0);
        let rem = parse_f64(usage, "remaining").unwrap_or(0.0);
        let reset = usage
            .get("resetTime")
            .and_then(|v| v.as_str())
            .map(String::from);
        if lim > 0.0 {
            let pct = ((lim - rem) / lim * 100.0).round();
            items.push(BalanceItem {
                label: "周窗口".into(),
                remaining: Some(rem),
                total: Some(lim),
                used: Some(pct),
                unit: "%".into(),
                is_valid: true,
                invalid_message: None,
                resets_at: reset,
            });
        }
    }

    if items.is_empty() {
        return BalanceResult::error("未能解析套餐数据");
    }

    BalanceResult {
        success: true,
        items,
        error: None,
        is_plan: true,
    }
}

// ==================== 智谱 GLM (套餐制) ====================

async fn query_zhipu(base_url: &str, api_key: &str) -> BalanceResult {
    // 智谱鉴权不加 Bearer 前缀
    let api_base = if base_url.to_lowercase().contains("z.ai") {
        "https://api.z.ai"
    } else {
        "https://open.bigmodel.cn"
    };
    let url = format!("{api_base}/api/monitor/usage/quota/limit");

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()
    {
        Ok(c) => c,
        Err(e) => return BalanceResult::error(format!("HTTP 客户端构建失败: {e}")),
    };

    let resp = match client
        .get(&url)
        .header("Authorization", api_key)
        .header("Accept", "application/json")
        .header("Accept-Language", "en-US,en")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return BalanceResult::error(format!("请求失败: {e}")),
    };

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return BalanceResult {
            success: true,
            items: vec![auth_error_item("API Key 失效")],
            error: None,
            is_plan: true,
        };
    }
    if !status.is_success() {
        return BalanceResult::error(format!("HTTP {status}"));
    }

    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return BalanceResult::error(format!("响应解析失败: {e}")),
    };

    if body.get("success").and_then(|v| v.as_bool()) == Some(false) {
        let msg = body
            .get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or("查询失败");
        return BalanceResult::error(msg);
    }

    let mut items = Vec::new();
    let level = body
        .pointer("/data/level")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if let Some(limits) = body.pointer("/data/limits").and_then(|v| v.as_array()) {
        for lim in limits {
            let ltype = lim.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if !ltype.eq_ignore_ascii_case("TOKENS_LIMIT") {
                continue;
            }
            let unit_val = lim.get("unit").and_then(|v| v.as_u64()).unwrap_or(0);
            let pct = parse_f64(lim, "percentage").unwrap_or(0.0);
            let reset_ms = lim
                .get("nextResetTime")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let reset_iso = if reset_ms > 0 {
                Some(format_iso_from_millis(reset_ms))
            } else {
                None
            };
            let label = match unit_val {
                3 => "5 小时窗口",
                6 => "周窗口",
                _ => "套餐窗口",
            };
            items.push(BalanceItem {
                label: label.into(),
                remaining: Some(100.0 - pct),
                total: Some(100.0),
                used: Some(pct),
                unit: "%".into(),
                is_valid: pct < 100.0,
                invalid_message: if pct >= 100.0 {
                    Some("已用尽".into())
                } else {
                    None
                },
                resets_at: reset_iso,
            });
        }
    }

    if items.is_empty() {
        return BalanceResult::error("未能解析套餐数据");
    }

    // 把套餐等级附加到第一个 item 的 label
    if !level.is_empty() {
        items[0].label = format!("{} ({})", items[0].label, level);
    }

    BalanceResult {
        success: true,
        items,
        error: None,
        is_plan: true,
    }
}

// ==================== MiniMax (套餐制) ====================

async fn query_minimax(base_url: &str, api_key: &str) -> BalanceResult {
    let domain = if base_url.to_lowercase().contains("minimax.io") {
        "https://api.minimax.io"
    } else {
        "https://api.minimaxi.com"
    };
    let url = format!("{domain}/v1/api/openplatform/coding_plan/remains");
    let auth = format!("Bearer {api_key}");

    let body = match http_get(&url, &auth).await {
        Ok(v) => v,
        Err(e) => return e,
    };

    // 业务校验
    if let Some(status_code) = body
        .pointer("/base_resp/status_code")
        .and_then(|v| v.as_i64())
    {
        if status_code != 0 {
            let msg = body
                .pointer("/base_resp/status_msg")
                .and_then(|v| v.as_str())
                .unwrap_or("查询失败");
            return BalanceResult::error(msg);
        }
    }

    let mut items = Vec::new();
    if let Some(remains) = body.get("model_remains").and_then(|v| v.as_array()) {
        for r in remains {
            let model = r.get("model_name").and_then(|v| v.as_str()).unwrap_or("");
            if model != "general" {
                continue;
            }
            // 5h 窗口
            if let Some(rem_pct) = parse_f64(r, "current_interval_remaining_percent") {
                let used_pct = 100.0 - rem_pct;
                let reset_ms = r.get("end_time").and_then(|v| v.as_u64()).unwrap_or(0);
                items.push(BalanceItem {
                    label: "5 小时窗口".into(),
                    remaining: Some(rem_pct),
                    total: Some(100.0),
                    used: Some(used_pct),
                    unit: "%".into(),
                    is_valid: used_pct < 100.0,
                    invalid_message: None,
                    resets_at: if reset_ms > 0 {
                        Some(format_iso_from_millis(reset_ms))
                    } else {
                        None
                    },
                });
            }
            // 周窗口 (仅 status==1 时有效)
            let weekly_status = r
                .get("current_weekly_status")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if weekly_status == 1 {
                if let Some(rem_pct) = parse_f64(r, "current_weekly_remaining_percent") {
                    let used_pct = 100.0 - rem_pct;
                    let reset_ms = r
                        .get("weekly_end_time")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    items.push(BalanceItem {
                        label: "周窗口".into(),
                        remaining: Some(rem_pct),
                        total: Some(100.0),
                        used: Some(used_pct),
                        unit: "%".into(),
                        is_valid: used_pct < 100.0,
                        invalid_message: None,
                        resets_at: if reset_ms > 0 {
                            Some(format_iso_from_millis(reset_ms))
                        } else {
                            None
                        },
                    });
                }
            }
        }
    }

    if items.is_empty() {
        return BalanceResult::error("未能解析套餐数据");
    }

    BalanceResult {
        success: true,
        items,
        error: None,
        is_plan: true,
    }
}

// ==================== 工具 ====================

fn format_iso_from_millis(ms: u64) -> String {
    let secs = ms / 1000;
    chrono_like_format(secs)
}

fn chrono_like_format(unix_secs: u64) -> String {
    let now_secs = unix_secs as i64;
    let days_from_epoch = now_secs / 86400;
    let secs_in_day = now_secs % 86400;
    let h = secs_in_day / 3600;
    let m = (secs_in_day % 3600) / 60;
    let s = secs_in_day % 60;

    // 简化日期计算 (从 1970-01-01 起)
    let (year, month, day) = epoch_to_date(days_from_epoch);
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

fn epoch_to_date(days: i64) -> (i64, u32, u32) {
    let mut y = 1970i64;
    let mut d = days;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let yd = if leap { 366 } else { 365 };
        if d < yd {
            break;
        }
        d -= yd;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let mdays = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0u32;
    for (i, &md) in mdays.iter().enumerate() {
        if d < md as i64 {
            m = i as u32 + 1;
            break;
        }
        d -= md as i64;
    }
    (y, m, (d + 1) as u32)
}
