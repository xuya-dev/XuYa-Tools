//! 代理 HTTP 服务器
//!
//! 基于 axum 的透明转发服务器。
//! 收到的请求会被转发到当前 active provider 的 base_url。
//! Phase 2 只做转发 + 状态维护，不做 usage 解析 (留给 Phase 3)。

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderName, HeaderValue},
    response::Response,
    routing::any,
    Router,
};
use http_body_util::BodyExt;
use tauri::Emitter;
use tokio::sync::{oneshot, RwLock};
use tokio::task::JoinHandle;

use super::types::{ProxyServerInfo, ProxyStatus};

/// 上游目标 (从当前 provider 解析而来)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UpstreamTarget {
    pub provider_id: String,
    pub provider_name: String,
    /// 完整的 base url, 如 https://api.example.com
    pub base_url: String,
    /// 转发时附加的 Authorization 头 (如果有 api_key)
    pub api_key: String,
    /// 上游 API 格式: anthropic(原生) / openai_chat(需转换) / openai_responses / gemini_native
    /// 决定请求体/响应体是否需要协议转换
    pub api_format: String,
}

/// 代理服务器共享状态
#[derive(Clone)]
pub struct ProxyState {
    /// per-app 上游目标 (key = "claude" / "codex")
    pub targets: Arc<RwLock<std::collections::HashMap<String, UpstreamTarget>>>,
    /// 运行状态
    pub status: Arc<RwLock<ProxyStatus>>,
    /// 累计请求数
    pub request_count: Arc<std::sync::atomic::AtomicU64>,
    pub error_count: Arc<std::sync::atomic::AtomicU64>,
    /// 请求数据库 (Phase 3)
    pub db: Option<crate::db::Database>,
    /// AppHandle, 用于 emit 告警事件 (Phase 4)。用 std RwLock 因为 log_request 是同步函数。
    pub app_handle: Arc<std::sync::RwLock<Option<tauri::AppHandle>>>,
}

impl Default for ProxyState {
    fn default() -> Self {
        Self {
            targets: Arc::new(RwLock::new(std::collections::HashMap::new())),
            status: Arc::new(RwLock::new(ProxyStatus::default())),
            request_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            error_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            db: None,
            app_handle: Arc::new(std::sync::RwLock::new(None)),
        }
    }
}

/// 代理服务器实例
pub struct ProxyServer {
    state: ProxyState,
    shutdown_tx: Arc<RwLock<Option<oneshot::Sender<()>>>>,
    server_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl ProxyServer {
    pub fn new(state: ProxyState) -> Self {
        Self {
            state,
            shutdown_tx: Arc::new(RwLock::new(None)),
            server_handle: Arc::new(RwLock::new(None)),
        }
    }

    #[allow(dead_code)]
    pub fn state(&self) -> ProxyState {
        self.state.clone()
    }

    /// 启动服务器, port=0 表示由系统分配空闲端口
    pub async fn start(&self, address: &str, port: u16) -> Result<ProxyServerInfo, String> {
        if self.shutdown_tx.read().await.is_some() {
            return Err("代理服务器已在运行".to_string());
        }

        let addr: SocketAddr = format!("{address}:{port}")
            .parse()
            .map_err(|e| format!("无效地址: {e}"))?;

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let app = build_router(self.state.clone());

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("端口绑定失败: {e}"))?;
        let actual_port = listener
            .local_addr()
            .map_err(|e| format!("获取端口失败: {e}"))?
            .port();

        let started_at = now_secs();
        let state = self.state.clone();
        let actual_address = address.to_string();

        // 更新状态
        {
            let mut status = self.state.status.write().await;
            status.running = true;
            status.address = actual_address.clone();
            status.port = actual_port;
            status.started_at = started_at;
        }

        let handle = tokio::spawn(async move {
            drop(shutdown_rx); // 持有以支持 graceful shutdown
            let make_svc = app.into_make_service();
            axum::serve(listener, make_svc)
                .with_graceful_shutdown(async {
                    // shutdown_rx 在外层 select 不再使用, 这里 await 阻塞直到任务被 abort
                    std::future::pending::<()>().await;
                })
                .await
                .ok();
            let mut status = state.status.write().await;
            status.running = false;
            status.port = 0;
            status.started_at = 0;
        });

        *self.shutdown_tx.write().await = Some(shutdown_tx);
        *self.server_handle.write().await = Some(handle);

        Ok(ProxyServerInfo {
            address: actual_address,
            port: actual_port,
            started_at,
        })
    }

    pub async fn stop(&self) -> Result<(), String> {
        // 清理 shutdown 通道 (虽然服务器任务用 abort 终止, 仍保留语义)
        if self.shutdown_tx.write().await.take().is_none()
            && self.server_handle.read().await.is_none()
        {
            return Err("代理服务器未运行".to_string());
        }
        if let Some(handle) = self.server_handle.write().await.take() {
            handle.abort();
            // 等待任务真正结束 (被 abort 时返回 Err, 也算正常停止)
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
            // 重要:任务被 abort 时, 其内部 axum::serve 之后的清理代码不会执行,
            // 因此必须在此显式重置运行状态, 否则前端始终认为代理在运行。
            let mut status = self.state.status.write().await;
            status.running = false;
            status.port = 0;
            status.started_at = 0;
            Ok(())
        } else {
            Ok(())
        }
    }
}

fn build_router(state: ProxyState) -> Router {
    Router::new()
        .route("/", any(root_handler))
        .fallback(any(move |req| forward_handler(req, state.clone())))
}

async fn root_handler() -> &'static str {
    "XuYa CLI Proxy running"
}

/// 核心转发处理: 把请求转发到上游 provider (按请求路径区分 Claude / Codex)
async fn forward_handler(req: Request<Body>, state: ProxyState) -> Response<Body> {
    let started = Instant::now();
    state
        .request_count
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // 先捕获路径, 用于推断 app 类型 + 日志
    let original_path = req
        .uri()
        .path_and_query()
        .map(|p| p.as_str().to_string())
        .unwrap_or_default();
    let app_type = infer_app_type(&original_path);

    // P0-1: 严格按 app_type 选 target, 不回落到其他 app (防止 Codex 请求泄漏到 Claude 上游)
    let target = {
        let targets = state.targets.read().await;
        targets.get(&app_type).cloned()
    };
    let Some(target) = target else {
        state
            .error_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        return error_response(503, &format!("{app_type} 未配置上游 Provider"));
    };

    let upstream = match build_upstream_url(&target.base_url, req.uri()) {
        Ok(u) => u,
        Err(e) => {
            state
                .error_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return error_response(500, &format!("上游地址无效: {e}"));
        }
    };

    // 构造转发请求
    let (mut parts, body) = req.into_parts();

    // 读取请求 body
    let body_bytes = match body.collect().await {
        Ok(b) => b.to_bytes(),
        Err(e) => {
            state
                .error_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return error_response(400, &format!("读取请求体失败: {e}"));
        }
    };

    // P2-7: 从请求体 JSON 提取 model 字段 (用于日志)
    let body_model: Option<String> = serde_json::from_slice::<serde_json::Value>(&body_bytes)
        .ok()
        .and_then(|v| v.get("model")?.as_str().map(|s| s.to_string()));

    // 协议转换: 当目标供应商是 openai_chat 格式, 而请求来自 Claude Code (anthropic /v1/messages)
    // 时, 把请求体 anthropic→openai 转换, 并把端点路径 /v1/messages → /v1/chat/completions。
    let mut fwd_body_bytes: bytes::Bytes = body_bytes.clone();
    let mut transformed_request = false; // 正向: anthropic→openai (请求) + openai→anthropic (响应)
    let mut reverse_transformed = false; // 反向: openai→anthropic (请求) + anthropic→openai (响应)

    if target.api_format == "openai_chat" && is_anthropic_messages_path(&original_path) {
        if let Ok(req_json) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
            if let Ok(converted) = super::transform::anthropic_to_openai(&req_json) {
                let vec = serde_json::to_vec(&converted).unwrap_or_else(|_| body_bytes.to_vec());
                fwd_body_bytes = bytes::Bytes::from(vec);
                transformed_request = true;
                if let Some(rewritten) =
                    rewrite_to_chat_completions(&target.base_url, &original_path)
                {
                    if let Ok(uri) = rewritten.parse::<http::Uri>() {
                        parts.uri = uri;
                    }
                }
            }
        }
    } else if target.api_format == "anthropic" && is_openai_chat_path(&original_path) {
        // 反向: Codex (OpenAI 协议) → Anthropic 原生上游
        if let Ok(req_json) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
            if let Ok(converted) = super::transform::openai_to_anthropic_request(&req_json) {
                let vec = serde_json::to_vec(&converted).unwrap_or_else(|_| body_bytes.to_vec());
                fwd_body_bytes = bytes::Bytes::from(vec);
                reverse_transformed = true;
                if let Some(rewritten) = rewrite_to_messages(&target.base_url) {
                    if let Ok(uri) = rewritten.parse::<http::Uri>() {
                        parts.uri = uri;
                    }
                }
            }
        }
    }
    // openai_responses / gemini_native 格式: 暂不做转换, 透传原样 (后续按需实现)
    // 前端已标注这些格式为"需转换",用户选择后自行承担兼容性

    // 替换 URI 指向上游 (未转换时用原始拼接)
    if !transformed_request {
        parts.uri = match upstream.parse::<http::Uri>() {
            Ok(u) => u,
            Err(e) => {
                state
                    .error_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return error_response(500, &format!("解析上游 URI 失败: {e}"));
            }
        };
    }

    // 移除 host 头 + 追踪/转发头, 让 hyper 自动填充, 避免内网信息泄漏到上游
    for h in [
        "host",
        "x-forwarded-host",
        "x-forwarded-proto",
        "x-forwarded-port",
        "x-forwarded-for",
        "forwarded",
        "x-real-ip",
        "cf-connecting-ip",
        "cf-ipcountry",
        "cf-ray",
        "cf-visitor",
        "x-amzn-trace-id",
        "x-azure-ref",
        "traceparent",
        "x-request-id",
    ] {
        parts.headers.remove(h);
    }
    // 剥离客户端自带的认证头, 防止旧凭据泄漏到新上游 (接管模式下 CLI 带的是旧 token)
    parts.headers.remove("authorization");
    parts.headers.remove("x-api-key");
    parts.headers.remove("anthropic-auth-token");
    // 转换后修正 content-length (body 长度变了)
    if transformed_request {
        if let Ok(cl) = HeaderValue::from_str(&fwd_body_bytes.len().to_string()) {
            parts.headers.insert("content-length", cl);
        }
    }
    // 注入认证: Anthropic 原生用 x-api-key, 其他用 Bearer
    if !target.api_key.is_empty() {
        let hv = match HeaderValue::from_str(&target.api_key) {
            Ok(v) => v,
            Err(_) => {
                state
                    .error_count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return error_response(500, "API Key 含非法字符");
            }
        };
        if target.api_format == "anthropic" {
            // Anthropic 原生: x-api-key 头
            parts
                .headers
                .insert(HeaderName::from_static("x-api-key"), hv);
            // 确保 anthropic-version 存在 (上游必需)
            if !parts.headers.contains_key("anthropic-version") {
                parts.headers.insert(
                    HeaderName::from_static("anthropic-version"),
                    HeaderValue::from_static("2023-06-01"),
                );
            }
        } else {
            // OpenAI / 其他: Authorization: Bearer
            let bearer = match HeaderValue::from_str(&format!("Bearer {}", target.api_key)) {
                Ok(v) => v,
                Err(_) => {
                    state
                        .error_count
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    return error_response(500, "API Key 含非法字符");
                }
            };
            parts
                .headers
                .insert(HeaderName::from_static("authorization"), bearer);
        }
    }

    let fwd_req = Request::from_parts(parts, FullBody::from(fwd_body_bytes));

    // 转发
    let client = build_http_client();
    let request_id = uuid::Uuid::new_v4().to_string();

    let result = client.request(fwd_req).await;
    let latency_ms = started.elapsed().as_millis() as u64;

    match result {
        Ok(resp) => {
            let status = resp.status();
            let status_code = status.as_u16();
            let (rparts, rbody) = resp.into_parts();

            // 判断是否流式响应 (SSE)。流式响应必须「边转发边读取」, 不能 collect 全量
            // 缓冲后再回写 —— 否则破坏 SSE 实时性, 客户端会卡到全部生成完才收到。
            let is_stream = original_path.contains("stream")
                || rparts
                    .headers
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .map(|ct| ct.contains("text/event-stream"))
                    .unwrap_or(false);

            if is_stream {
                // 真流式: 用 channel 把上游 body 分流 —— 一路回写给客户端, 一路收集用于日志。
                // 使用 mpsc 把收集任务解耦, 避免阻塞转发。
                let (tx, mut rx) =
                    tokio::sync::mpsc::channel::<Result<bytes::Bytes, std::io::Error>>(32);
                // 首 token 时间: 在转发流中记录第一个 data chunk 到达时刻
                let first_token_instant = Arc::new(std::sync::Mutex::new(Option::<Instant>::None));
                let ft_clone = first_token_instant.clone();

                // 收集任务: 累积所有 chunk 用于 usage 解析 + 日志 (非阻塞, 与转发并行)
                let collect_state = state.clone();
                let collect_target = target.clone();
                let collect_app_type = app_type.clone();
                let collect_path = original_path.clone();
                let collect_rid = request_id.clone();
                let collect_model = body_model.clone();
                tokio::spawn(async move {
                    let mut buf: Vec<u8> = Vec::new();
                    while let Some(chunk) = rx.recv().await {
                        if let Ok(b) = chunk {
                            buf.extend_from_slice(&b);
                        }
                    }
                    let parsed = parse_usage(&buf);
                    let (in_tok, out_tok, cache_read, cache_create) = (
                        parsed.input,
                        parsed.output,
                        parsed.cache_read,
                        parsed.cache_creation,
                    );
                    let cost = calculate_cost_full(&collect_app_type, &buf, &parsed);
                    // 首 token 延迟 (从请求开始到第一个 chunk)
                    let first_token_ms = ft_clone
                        .lock()
                        .map(|opt| opt.map(|t| t.duration_since(started).as_millis() as u64))
                        .ok()
                        .flatten();
                    let total_latency = started.elapsed().as_millis() as u64;
                    log_request_with_first_token(
                        &collect_state,
                        &collect_rid,
                        &collect_target,
                        &collect_app_type,
                        &collect_path,
                        status_code,
                        total_latency,
                        first_token_ms,
                        Some((in_tok, out_tok, cache_read, cache_create)),
                        cost,
                        if status_code >= 400 {
                            Some(format!("HTTP {status_code}"))
                        } else {
                            None
                        },
                        collect_model.as_deref(),
                        true,
                    );
                });

                // 分流: 遍历上游 frame, 同时发给 collect 通道和客户端流。
                // 若请求做了转换, 流式响应也需转换:
                //   正向 (transformed_request): OpenAI SSE → Anthropic SSE (SseConverter)
                //   反向 (reverse_transformed): Anthropic SSE → OpenAI SSE (AnthropicSseConverter)
                let needs_sse_conversion = transformed_request || reverse_transformed;
                let client_stream = http_body_util::StreamBody::new(async_stream::stream! {
                    let mut rbody = rbody;
                    let mut sse_converter: Option<Box<dyn super::transform::SseConvert>> = if needs_sse_conversion {
                        if reverse_transformed {
                            Some(Box::new(super::transform::AnthropicSseConverter::new()))
                        } else {
                            Some(Box::new(super::transform::SseConverter::new()))
                        }
                    } else {
                        None
                    };
                    while let Some(frame_res) = rbody.frame().await {
                        match frame_res {
                            Ok(frame) => {
                                if let Some(data) = frame.data_ref() {
                                    // 记录首 token 时刻 (仅第一次)
                                    if !data.is_empty() {
                                        if let Ok(mut guard) = first_token_instant.lock() {
                                            if guard.is_none() {
                                                *guard = Some(Instant::now());
                                            }
                                        }
                                    }
                                    // 原始数据发给收集通道 (用于 usage 日志)
                                    let _ = tx.send(Ok(data.clone())).await;

                                    // 若需转换, 把 openai SSE → anthropic SSE
                                    if let Some(ref mut conv) = sse_converter {
                                        let converted = conv.feed(data);
                                        if !converted.is_empty() {
                                            yield Ok::<_, std::convert::Infallible>(
                                                http_body::Frame::data(bytes::Bytes::from(converted)),
                                            );
                                        }
                                    } else {
                                        yield Ok::<_, std::convert::Infallible>(frame);
                                    }
                                } else {
                                    yield Ok::<_, std::convert::Infallible>(frame);
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    // flush 残留
                    if let Some(ref mut conv) = sse_converter {
                        let remaining = conv.flush();
                        if !remaining.is_empty() {
                            yield Ok::<_, std::convert::Infallible>(
                                http_body::Frame::data(bytes::Bytes::from(remaining)),
                            );
                        }
                    }
                });

                let mut resp_builder = Response::builder().status(status);
                for (k, v) in rparts.headers.iter() {
                    resp_builder = resp_builder.header(k, v);
                }
                return resp_builder
                    .body(Body::new(client_stream))
                    .unwrap_or_else(|_| error_response(500, "构造流式响应失败"));
            }

            // 非流式: 保持原有的 collect 行为 (一次性响应, 缓冲合理)
            let rbody_bytes = match rbody.collect().await {
                Ok(b) => b.to_bytes(),
                Err(e) => {
                    state
                        .error_count
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    log_request(
                        &state,
                        &request_id,
                        &target,
                        &app_type,
                        &original_path,
                        status_code,
                        latency_ms,
                        None,
                        0.0,
                        Some(format!("读取上游响应失败: {e}")),
                        body_model.as_deref(),
                        false,
                    );
                    return error_response(502, &format!("读取上游响应失败: {e}"));
                }
            };

            // 协议转换响应回转:
            //   正向 (transformed_request): OpenAI 响应 → Anthropic 响应
            //   反向 (reverse_transformed): Anthropic 响应 → OpenAI 响应
            let final_bytes: bytes::Bytes = if status.is_success() {
                if transformed_request {
                    if let Ok(resp_json) = serde_json::from_slice::<serde_json::Value>(&rbody_bytes)
                    {
                        match super::transform::openai_to_anthropic(&resp_json) {
                            Ok(converted) => {
                                let vec = serde_json::to_vec(&converted)
                                    .unwrap_or_else(|_| rbody_bytes.to_vec());
                                bytes::Bytes::from(vec)
                            }
                            Err(_) => rbody_bytes.clone(),
                        }
                    } else {
                        rbody_bytes.clone()
                    }
                } else if reverse_transformed {
                    if let Ok(resp_json) = serde_json::from_slice::<serde_json::Value>(&rbody_bytes)
                    {
                        match super::transform::anthropic_to_openai_response(&resp_json) {
                            Ok(converted) => {
                                let vec = serde_json::to_vec(&converted)
                                    .unwrap_or_else(|_| rbody_bytes.to_vec());
                                bytes::Bytes::from(vec)
                            }
                            Err(_) => rbody_bytes.clone(),
                        }
                    } else {
                        rbody_bytes.clone()
                    }
                } else {
                    rbody_bytes
                }
            } else {
                rbody_bytes
            };

            // 尝试从响应体解析 token usage + 计算成本 (基于转换后的 anthropic 格式, 含 cache)
            let parsed = parse_usage(&final_bytes);
            let (in_tok, out_tok, cache_read, cache_create) = (
                parsed.input,
                parsed.output,
                parsed.cache_read,
                parsed.cache_creation,
            );
            let cost = calculate_cost_full(&app_type, &final_bytes, &parsed);

            log_request(
                &state,
                &request_id,
                &target,
                &app_type,
                &original_path,
                status_code,
                latency_ms,
                Some((in_tok, out_tok, cache_read, cache_create)),
                cost,
                if status_code >= 400 {
                    Some(format!("HTTP {status_code}"))
                } else {
                    None
                },
                body_model.as_deref(),
                false,
            );

            let mut resp_builder = Response::builder().status(status);
            for (k, v) in rparts.headers.iter() {
                // 转换后修正 content-length
                if transformed_request && k == "content-length" {
                    continue;
                }
                resp_builder = resp_builder.header(k, v);
            }
            if transformed_request {
                if let Ok(cl) = HeaderValue::from_str(&final_bytes.len().to_string()) {
                    resp_builder = resp_builder.header("content-length", cl);
                }
            }
            resp_builder
                .body(Body::from(final_bytes))
                .unwrap_or_else(|_| error_response(500, "构造响应失败"))
        }
        Err(e) => {
            state
                .error_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            log_request(
                &state,
                &request_id,
                &target,
                &app_type,
                &original_path,
                0,
                latency_ms,
                None,
                0.0,
                Some(format!("转发到上游失败: {e}")),
                body_model.as_deref(),
                false,
            );
            error_response(502, &format!("转发到上游失败: {e}"))
        }
    }
}

/// 写入请求日志 (有 db 才写, 失败不阻塞转发)。严重错误时 emit cli-alert。
#[allow(clippy::too_many_arguments)]
fn log_request(
    state: &ProxyState,
    request_id: &str,
    target: &UpstreamTarget,
    app_type: &str,
    path: &str,
    status_code: u16,
    latency_ms: u64,
    usage: Option<(u64, u64, u64, u64)>,
    cost_usd: f64,
    error_message: Option<String>,
    body_model: Option<&str>,
    is_stream: bool,
) {
    log_request_with_cost(
        state,
        request_id,
        target,
        app_type,
        path,
        status_code,
        latency_ms,
        usage,
        cost_usd,
        error_message,
        body_model,
        is_stream,
    );
}

/// 带成本参数的日志写入 (log_request 的完整版, 流式与非流式共用)
#[allow(clippy::too_many_arguments)]
fn log_request_with_cost(
    state: &ProxyState,
    request_id: &str,
    target: &UpstreamTarget,
    app_type: &str,
    path: &str,
    status_code: u16,
    latency_ms: u64,
    usage: Option<(u64, u64, u64, u64)>,
    cost_usd: f64,
    error_message: Option<String>,
    body_model: Option<&str>,
    is_stream: bool,
) {
    log_request_with_first_token(
        state,
        request_id,
        target,
        app_type,
        path,
        status_code,
        latency_ms,
        None,
        usage,
        cost_usd,
        error_message,
        body_model,
        is_stream,
    );
}

/// 带首 token 延迟的日志写入 (流式专用, 非 token 恒 None)
#[allow(clippy::too_many_arguments)]
fn log_request_with_first_token(
    state: &ProxyState,
    request_id: &str,
    target: &UpstreamTarget,
    app_type: &str,
    path: &str,
    status_code: u16,
    latency_ms: u64,
    first_token_ms: Option<u64>,
    usage: Option<(u64, u64, u64, u64)>,
    cost_usd: f64,
    error_message: Option<String>,
    body_model: Option<&str>,
    is_stream: bool,
) {
    if let Some(db) = &state.db {
        let (in_tok, out_tok, cache_read, cache_create) = usage.unwrap_or((0, 0, 0, 0));
        let log = crate::usage::types::RequestLog {
            request_id: request_id.to_string(),
            app_type: app_type.to_string(),
            provider_id: Some(target.provider_id.clone()),
            provider_name: Some(target.provider_name.clone()),
            model: body_model
                .map(String::from)
                .or_else(|| extract_model_from_path(path)),
            request_model: body_model.map(String::from),
            input_tokens: in_tok,
            output_tokens: out_tok,
            cache_read_tokens: cache_read,
            cache_creation_tokens: cache_create,
            total_cost_usd: cost_usd,
            latency_ms,
            first_token_ms,
            status_code,
            error_message: error_message.clone(),
            is_streaming: is_stream,
            created_at: now_secs(),
        };
        let _ = crate::usage::logger::log_request(db, &log);

        // 通知前端有新日志 (前端自行防抖刷新)
        let handle_opt = state
            .app_handle
            .read()
            .expect("app_handle 锁被毒化,无法读取")
            .clone();
        if let Some(handle) = &handle_opt {
            let _ = handle.emit("cli-usage-recorded", ());
        }
    }

    // 严重错误 (status >= 500 或转发完全失败 status=0) 时通知前端告警抢占
    let is_severe = status_code == 0 || status_code >= 500;
    if is_severe {
        let handle_opt = state
            .app_handle
            .read()
            .expect("app_handle 锁被毒化,无法读取告警")
            .clone();
        if let Some(handle) = handle_opt {
            let _ = handle.emit(
                "cli-alert",
                serde_json::json!({
                    "message": error_message.clone().unwrap_or_else(|| format!("HTTP {status_code}")),
                    "appType": app_type,
                    "provider": target.provider_name,
                }),
            );
        }
    }
}

/// 带缓存 token 的完整成本计算。
/// 缓存读取 (cache_read) 按 input 价的 10% 计 (Anthropic 约定);
/// 缓存写入 (cache_creation) 按 input 价的 125% 计。
fn calculate_cost_full(app_type: &str, body: &[u8], usage: &ParsedUsage) -> f64 {
    if usage.input == 0 && usage.output == 0 && usage.cache_read == 0 && usage.cache_creation == 0 {
        return 0.0;
    }
    let model = extract_model_from_response(body, app_type);
    let Some(model) = model else {
        return 0.0;
    };
    let (in_price, out_price) = match model_price(&model) {
        Some(p) => p,
        None => return 0.0,
    };
    let per_mtok = 1_000_000.0;
    let cost = (usage.input as f64 / per_mtok) * in_price
        + (usage.output as f64 / per_mtok) * out_price
        + (usage.cache_read as f64 / per_mtok) * (in_price * 0.1)
        + (usage.cache_creation as f64 / per_mtok) * (in_price * 1.25);
    (cost * 1_000_000.0).round() / 1_000_000.0 // 保留 6 位
}

/// 从响应体提取模型名 (用于成本归因)
fn extract_model_from_response(body: &[u8], _app_type: &str) -> Option<String> {
    let text = std::str::from_utf8(body).ok()?;
    let json = serde_json::from_str::<serde_json::Value>(text).ok()?;
    // Anthropic: { model: "..." }; OpenAI: { model: "..." }
    json.get("model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// 内置模型定价表 (input_per_mtok_usd, output_per_mtok_usd)。
/// 数据为公开标价近似值, 仅供估算。前缀匹配, 越具体越优先。
fn model_price(model: &str) -> Option<(f64, f64)> {
    let m = model.to_lowercase();
    // Claude 4.x
    if m.starts_with("claude-opus-4") {
        return Some((15.0, 75.0));
    }
    if m.starts_with("claude-sonnet-4") {
        return Some((3.0, 15.0));
    }
    if m.starts_with("claude-haiku-4") {
        return Some((1.0, 5.0));
    }
    // Claude 3.7
    if m.starts_with("claude-3-7-sonnet") {
        return Some((3.0, 15.0));
    }
    if m.starts_with("claude-3-5-haiku") {
        return Some((1.0, 5.0));
    }
    if m.starts_with("claude-3-5-sonnet") {
        return Some((3.0, 15.0));
    }
    if m.starts_with("claude-3-opus") {
        return Some((15.0, 75.0));
    }
    if m.starts_with("claude-3-haiku") {
        return Some((0.25, 1.25));
    }
    // GPT
    if m.starts_with("gpt-4o-mini") {
        return Some((0.15, 0.6));
    }
    if m.starts_with("gpt-4o") {
        return Some((2.5, 10.0));
    }
    if m.starts_with("gpt-4-turbo") {
        return Some((10.0, 30.0));
    }
    if m.starts_with("gpt-4") {
        return Some((30.0, 60.0));
    }
    if m.starts_with("gpt-3.5") {
        return Some((0.5, 1.5));
    }
    // DeepSeek
    if m.starts_with("deepseek-chat") || m.starts_with("deepseek-v3") {
        return Some((0.27, 1.1));
    }
    if m.starts_with("deepseek-reasoner") || m.starts_with("deepseek-r1") {
        return Some((0.55, 2.19));
    }
    // Gemini
    if m.starts_with("gemini-2") {
        return Some((1.25, 5.0));
    }
    if m.starts_with("gemini-1.5-pro") {
        return Some((1.25, 5.0));
    }
    if m.starts_with("gemini-1.5-flash") {
        return Some((0.075, 0.3));
    }
    None
}

/// 判断请求路径是否为 Anthropic Messages 端点 (需转换)
fn is_anthropic_messages_path(path: &str) -> bool {
    let p = path.split('?').next().unwrap_or(path);
    p.ends_with("/v1/messages")
}

/// 判断请求路径是否为 OpenAI Chat Completions 端点 (需反向转换)
fn is_openai_chat_path(path: &str) -> bool {
    let p = path.split('?').next().unwrap_or(path);
    p.ends_with("/chat/completions") || p.ends_with("/v1/chat/completions")
}

/// 反向转换: 把上游 base_url 改写为 Anthropic Messages 端点
fn rewrite_to_messages(base_url: &str) -> Option<String> {
    let base = base_url.trim_end_matches('/');
    Some(format!("{base}/v1/messages"))
}

/// 把 anthropic /v1/messages 端点改写为 openai /v1/chat/completions
/// 保留 query string, 替换路径段
fn rewrite_to_chat_completions(base_url: &str, original_path: &str) -> Option<String> {
    let (path, query) = match original_path.find('?') {
        Some(idx) => (&original_path[..idx], &original_path[idx..]),
        None => (original_path, ""),
    };
    let new_path = path.replace("/v1/messages", "/v1/chat/completions");
    let base = base_url.trim_end_matches('/');
    Some(format!("{base}{new_path}{query}"))
}

/// 从请求路径推断 app 类型 (用于 per-app target 选择)
fn infer_app_type(path: &str) -> String {
    let p = path.to_lowercase();
    let path_only = p.split('?').next().unwrap_or(&p);
    if path_only.ends_with("/v1/messages") || path_only.contains("/claude/") {
        "claude".to_string()
    } else if path_only.ends_with("/chat/completions")
        || path_only.ends_with("/responses")
        || path_only.contains("/codex/")
    {
        "codex".to_string()
    } else {
        "unknown".to_string()
    }
}

/// 从路径提取可能的 model 名 (如 /v1/messages?model=xxx)
fn extract_model_from_path(path: &str) -> Option<String> {
    if let Some(idx) = path.find("model=") {
        let rest = &path[idx + 6..];
        let end = rest.find('&').unwrap_or(rest.len());
        return Some(rest[..end].to_string());
    }
    None
}

/// 解析出的 token 用量 (含缓存 token)
#[derive(Debug, Clone, Default)]
struct ParsedUsage {
    input: u64,
    output: u64,
    cache_read: u64,
    cache_creation: u64,
}

/// 从响应体解析 token 用量 (尽力而为, 支持 OpenAI / Anthropic 常见格式, 含缓存 token)
fn parse_usage(body: &[u8]) -> ParsedUsage {
    let Ok(text) = std::str::from_utf8(body) else {
        return ParsedUsage::default();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
        return ParsedUsage::default();
    };

    let Some(usage) = json.get("usage") else {
        return ParsedUsage::default();
    };

    // input: anthropic input_tokens / openai prompt_tokens
    let input = usage
        .get("input_tokens")
        .and_then(|v| v.as_u64())
        .or_else(|| usage.get("prompt_tokens").and_then(|v| v.as_u64()))
        .unwrap_or(0);
    let output = usage
        .get("output_tokens")
        .and_then(|v| v.as_u64())
        .or_else(|| usage.get("completion_tokens").and_then(|v| v.as_u64()))
        .unwrap_or(0);

    // 缓存 token: anthropic 直传 / openai nested
    let cache_read = usage
        .get("cache_read_input_tokens")
        .and_then(|v| v.as_u64())
        .or_else(|| {
            usage
                .pointer("/prompt_tokens_details/cached_tokens")
                .and_then(|v| v.as_u64())
        })
        .unwrap_or(0);
    let cache_creation = usage
        .get("cache_creation_input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    ParsedUsage {
        input,
        output,
        cache_read,
        cache_creation,
    }
}

/// 拼接 base_url + 原始 path/query
fn build_upstream_url(base_url: &str, original_uri: &http::Uri) -> Result<String, String> {
    let base = base_url.trim_end_matches('/');
    let path_and_query = original_uri
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or("/");
    Ok(format!("{base}{path_and_query}"))
}

/// 完整 body 类型别名 (collect 后的全量字节)
type FullBody = http_body_util::Full<bytes::Bytes>;

/// HTTP 客户端类型别名 (明确 connector + body 类型)
type HttpClient = hyper_util::client::legacy::Client<
    hyper_tls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
    FullBody,
>;

fn build_http_client() -> HttpClient {
    use hyper_tls::HttpsConnector;
    use hyper_util::client::legacy::connect::HttpConnector;

    let mut http = HttpConnector::new();
    http.enforce_http(false);
    let https = HttpsConnector::new_with_connector(http);

    hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build::<_, FullBody>(https)
}

fn error_response(status: u16, msg: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(msg.to_string()))
        .expect("构造错误响应失败: status 与 body 均合法,不应失败")
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
