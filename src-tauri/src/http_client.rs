// HTTP / WebSocket 请求工具后端
// - HTTP: 使用 reqwest 发起请求 (绕过浏览器 CORS)
// - WebSocket: 使用 tokio-tungstenite, 后台任务读循环 + AppHandle emit 推送消息

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::mpsc;

// ==================== HTTP 请求 ====================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestInput {
    pub method: String,                       // GET/POST/PUT/PATCH/DELETE/HEAD/OPTIONS
    pub url: String,
    #[serde(default)]
    pub headers: Vec<KeyValue>,               // 自定义请求头
    #[serde(default)]
    pub query: Vec<KeyValue>,                 // 查询参数
    #[serde(default)]
    pub body: Option<String>,                 // 请求体
    #[serde(default)]
    pub body_type: Option<String>,            // "json" | "form" | "raw"
    #[serde(default)]
    pub timeout_ms: Option<u64>,              // 超时(毫秒)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponseOutput {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<KeyValue>,
    pub body: String,
    pub elapsed_ms: u64,
    pub size_bytes: usize,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn http_request(req: HttpRequestInput) -> Result<HttpResponseOutput, String> {
    let started = Instant::now();

    // 校验 URL
    let url = reqwest::Url::parse(&req.url).map_err(|e| format!("URL 无效: {e}"))?;

    // 构建客户端
    let mut client_builder = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10));
    if let Some(ms) = req.timeout_ms {
        if ms > 0 {
            client_builder = client_builder.timeout(Duration::from_millis(ms));
        }
    }
    let client = client_builder
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {e}"))?;

    // 解析方法
    let method = reqwest::Method::from_bytes(req.method.to_uppercase().as_bytes())
        .map_err(|e| format!("无效的 HTTP 方法: {e}"))?;

    // 构建 request
    let mut builder = client.request(method, url);

    // 注入 headers
    for h in &req.headers {
        if h.enabled.unwrap_or(true) && !h.key.is_empty() {
            builder = builder.header(&h.key, &h.value);
        }
    }

    // 注入 query
    let active_query: Vec<(&str, &str)> = req
        .query
        .iter()
        .filter(|q| q.enabled.unwrap_or(true) && !q.key.is_empty())
        .map(|q| (q.key.as_str(), q.value.as_str()))
        .collect();
    if !active_query.is_empty() {
        builder = builder.query(&active_query);
    }

    // 注入 body
    if let Some(body) = &req.body {
        if !body.is_empty() {
            let body_type = req.body_type.as_deref().unwrap_or("raw");
            match body_type {
                "json" => {
                    builder = builder.header("content-type", "application/json").body(body.clone());
                }
                "form" => {
                    // form body: 解析 key=value&key2=value2
                    let mut form_vec: Vec<(String, String)> = Vec::new();
                    for pair in body.split('&') {
                        let mut it = pair.splitn(2, '=');
                        if let (Some(k), Some(v)) = (it.next(), it.next()) {
                            if !k.is_empty() {
                                form_vec.push((k.to_string(), v.to_string()));
                            }
                        }
                    }
                    builder = builder.form(&form_vec);
                }
                _ => {
                    builder = builder.body(body.clone());
                }
            }
        }
    }

    // 发送
    let response = match builder.send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(HttpResponseOutput {
                status: 0,
                status_text: "Error".to_string(),
                headers: vec![],
                body: String::new(),
                elapsed_ms: started.elapsed().as_millis() as u64,
                size_bytes: 0,
                error: Some(format!("请求失败: {e}")),
            });
        }
    };

    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("")
        .to_string();

    // 收集响应头
    let mut headers: Vec<KeyValue> = Vec::new();
    for (name, value) in response.headers() {
        if let Ok(v) = value.to_str() {
            headers.push(KeyValue {
                key: name.as_str().to_string(),
                value: v.to_string(),
                enabled: Some(true),
            });
        }
    }

    // 读取 body (二进制安全: 转 lossy 字符串)
    let bytes = response.bytes().await.map_err(|e| format!("读取响应体失败: {e}"))?;
    let size_bytes = bytes.len();
    let body_raw = String::from_utf8_lossy(&bytes).to_string();

    // 尝试美化 JSON
    let body = if body_raw.trim_start().starts_with('{') || body_raw.trim_start().starts_with('[') {
        serde_json::from_str::<serde_json::Value>(&body_raw)
            .ok()
            .and_then(|v| serde_json::to_string_pretty(&v).ok())
            .unwrap_or(body_raw)
    } else {
        body_raw
    };

    Ok(HttpResponseOutput {
        status,
        status_text,
        headers,
        body,
        elapsed_ms: started.elapsed().as_millis() as u64,
        size_bytes,
        error: None,
    })
}

// ==================== WebSocket ====================

/// 单条 WS 推送消息 (emit 给前端)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WsEvent {
    pub connection_id: String,
    pub kind: String,     // "open" | "message" | "close" | "error"
    pub data: String,
    pub ts: i64,          // unix 毫秒
}

/// 一个连接的写端 + 取消句柄
struct WsConnection {
    tx: mpsc::Sender<WsOutgoing>,
}

enum WsOutgoing {
    Text(String),
    Close,
}

#[derive(Default)]
pub struct WsManager {
    connections: Mutex<HashMap<String, WsConnection>>,
    seq: AtomicU64,
}

impl WsManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn next_id(&self) -> String {
        format!("ws_{}", self.seq.fetch_add(1, Ordering::SeqCst))
    }
}

#[tauri::command]
pub async fn ws_connect(
    app: AppHandle,
    state: State<'_, WsManager>,
    url: String,
    headers: Option<Vec<KeyValue>>,
    protocols: Option<Vec<String>>,
) -> Result<String, String> {
    use tokio_tungstenite::tungstenite::handshake::client::Request;
    use tokio_tungstenite::tungstenite::http::Uri;

    let conn_id = state.next_id();
    let conn_id_for_task = conn_id.clone();
    let app_for_task = app.clone();

    // 构建请求 (带自定义 headers)
    let uri: Uri = url.as_str().parse().map_err(|e| format!("URL 无效: {e}"))?;
    let mut req_builder = Request::builder().uri(uri);
    if let Some(hs) = &headers {
        for h in hs {
            if h.enabled.unwrap_or(true) && !h.key.is_empty() {
                req_builder = req_builder.header(&h.key, &h.value);
            }
        }
    }
    // 子协议
    if let Some(ps) = &protocols {
        let joined = ps.join(", ");
        if !joined.is_empty() {
            req_builder = req_builder.header("Sec-WebSocket-Protocol", joined);
        }
    }
    let ws_request = req_builder
        .header("User-Agent", "XuYa-Tools-WS")
        .body(())
        .map_err(|e| format!("构建 WS 请求失败: {e}"))?;

    // 写通道: 前端 ws_send → tx → 后台写任务
    let (out_tx, mut out_rx) = mpsc::channel::<WsOutgoing>(32);

    // 尝试连接
    let (ws_stream, _response) = match tokio_tungstenite::connect_async(ws_request).await {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("连接失败: {e}");
            let _ = app.emit(
                "ws-message",
                WsEvent {
                    connection_id: conn_id.clone(),
                    kind: "error".into(),
                    data: msg.clone(),
                    ts: now_ms(),
                },
            );
            return Err(msg);
        }
    };

    // 注册连接
    {
        let mut conns = state
            .connections
            .lock()
            .expect("WebSocket 连接表锁被毒化");
        conns.insert(conn_id.clone(), WsConnection { tx: out_tx });
    }

    // emit open
    let _ = app.emit(
        "ws-message",
        WsEvent {
            connection_id: conn_id.clone(),
            kind: "open".into(),
            data: "连接已建立".into(),
            ts: now_ms(),
        },
    );

    // 后台任务: 读循环 + 写循环
    use tokio_tungstenite::WebSocketStream;
    use futures_util::{SinkExt, StreamExt};
    let mut stream: WebSocketStream<_> = ws_stream;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                // 读: 服务端消息
                msg = stream.next() => {
                    match msg {
                        Some(Ok(frame)) => {
                            let data = match &frame {
                                tokio_tungstenite::tungstenite::Message::Text(t) => t.to_string(),
                                tokio_tungstenite::tungstenite::Message::Binary(b) => {
                                    String::from_utf8_lossy(b).to_string()
                                }
                                tokio_tungstenite::tungstenite::Message::Ping(_) => continue,
                                tokio_tungstenite::tungstenite::Message::Pong(_) => continue,
                                tokio_tungstenite::tungstenite::Message::Close(_) => {
                                    let _ = app_for_task.emit("ws-message", WsEvent {
                                        connection_id: conn_id_for_task.clone(),
                                        kind: "close".into(),
                                        data: "服务端关闭连接".into(),
                                        ts: now_ms(),
                                    });
                                    break;
                                }
                                tokio_tungstenite::tungstenite::Message::Frame(_) => continue,
                            };
                            let _ = app_for_task.emit("ws-message", WsEvent {
                                connection_id: conn_id_for_task.clone(),
                                kind: "message".into(),
                                data,
                                ts: now_ms(),
                            });
                        }
                        Some(Err(e)) => {
                            let _ = app_for_task.emit("ws-message", WsEvent {
                                connection_id: conn_id_for_task.clone(),
                                kind: "error".into(),
                                data: format!("读取错误: {e}"),
                                ts: now_ms(),
                            });
                            break;
                        }
                        None => {
                            let _ = app_for_task.emit("ws-message", WsEvent {
                                connection_id: conn_id_for_task.clone(),
                                kind: "close".into(),
                                data: "连接已断开".into(),
                                ts: now_ms(),
                            });
                            break;
                        }
                    }
                }
                // 写: 前端发来的消息
                out = out_rx.recv() => {
                    match out {
                        Some(WsOutgoing::Text(t)) => {
                            if stream.send(tokio_tungstenite::tungstenite::Message::text(t)).await.is_err() {
                                let _ = app_for_task.emit("ws-message", WsEvent {
                                    connection_id: conn_id_for_task.clone(),
                                    kind: "error".into(),
                                    data: "发送失败".into(),
                                    ts: now_ms(),
                                });
                                break;
                            }
                        }
                        Some(WsOutgoing::Close) | None => {
                            let _ = stream.close(None).await;
                            let _ = app_for_task.emit("ws-message", WsEvent {
                                connection_id: conn_id_for_task.clone(),
                                kind: "close".into(),
                                data: "连接已关闭".into(),
                                ts: now_ms(),
                            });
                            break;
                        }
                    }
                }
            }
        }
    });

    Ok(conn_id)
}

#[tauri::command]
pub async fn ws_send(
    state: State<'_, WsManager>,
    connection_id: String,
    message: String,
) -> Result<(), String> {
    let tx = {
        let conns = state
            .connections
            .lock()
            .expect("WebSocket 连接表锁被毒化");
        conns.get(&connection_id).map(|c| c.tx.clone())
    };
    match tx {
        Some(tx) => {
            tx.send(WsOutgoing::Text(message))
                .await
                .map_err(|_| "连接已关闭,无法发送".to_string())
        }
        None => Err("连接不存在".to_string()),
    }
}

#[tauri::command]
pub async fn ws_disconnect(
    state: State<'_, WsManager>,
    connection_id: String,
) -> Result<(), String> {
    let conn = {
        let mut conns = state
            .connections
            .lock()
            .expect("WebSocket 连接表锁被毒化");
        conns.remove(&connection_id)
    };
    if let Some(c) = conn {
        let _ = c.tx.send(WsOutgoing::Close).await;
        Ok(())
    } else {
        Err("连接不存在".to_string())
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// 避免未使用导入告警
#[allow(dead_code)]
fn _unused() {
    let _: Arc<()> = Arc::new(());
}
