//! 端口 / TCP 连通性检测。
//!
//! 提供 `port_check` 命令:对指定 host:port 做带超时的 TCP 连接探测,
//! 返回连通状态与握手耗时。支持单端口与端口段(如 80,443,8000-8010)批量扫描。

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::command;

const DEFAULT_TIMEOUT_MS: u64 = 3000;
const MAX_PORTS: usize = 200;

/// 单个目标:可带端口范围(如 "8000-8010")。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortCheckInput {
    /// 主机名或 IP
    pub host: String,
    /// 端口列表,逗号分隔,支持区间 "80,443,8000-8010"
    pub ports: String,
    /// 单端口超时(毫秒),可选
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortResult {
    pub port: u16,
    pub open: bool,
    /// 连接耗时(毫秒),失败时为 0
    pub latency_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortCheckOutput {
    pub host: String,
    pub results: Vec<PortResult>,
    /// 总耗时(毫秒)
    pub total_ms: u64,
}

/// 解析端口段字符串,展开成具体端口列表。
/// 支持 "80,443,8000-8010" 这类混合写法。最多展开 MAX_PORTS 个,超出报错。
fn parse_ports(spec: &str) -> Result<Vec<u16>, String> {
    let mut out: Vec<u16> = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((a, b)) = part.split_once('-') {
            let start: u16 = a
                .trim()
                .parse()
                .map_err(|_| format!("非法端口段起始: {a}"))?;
            let end: u16 = b
                .trim()
                .parse()
                .map_err(|_| format!("非法端口段结束: {b}"))?;
            if start > end {
                return Err(format!("端口段起始大于结束: {start}-{end}"));
            }
            for p in start..=end {
                out.push(p);
            }
        } else {
            let p: u16 = part
                .parse()
                .map_err(|_| format!("非法端口: {part}"))?;
            out.push(p);
        }
        if out.len() > MAX_PORTS {
            return Err(format!("端口数量超过上限 {MAX_PORTS},请缩小范围"));
        }
    }
    if out.is_empty() {
        return Err("未指定有效端口".into());
    }
    Ok(out)
}

/// 探测单个 host:port,带超时。返回 (是否连通, 耗时ms, 错误)。
async fn probe(host: &str, port: u16, timeout_ms: u64) -> (bool, u64, Option<String>) {
    let addr = format!("{host}:{port}");
    let start = Instant::now();
    let connect = tokio::net::TcpStream::connect(&addr);
    match tokio::time::timeout(Duration::from_millis(timeout_ms), connect).await {
        Ok(Ok(_stream)) => (true, start.elapsed().as_millis() as u64, None),
        Ok(Err(e)) => (false, start.elapsed().as_millis() as u64, Some(e.to_string())),
        Err(_) => (false, timeout_ms, Some(format!("连接超时 ({timeout_ms}ms)"))),
    }
}

/// 前端调用:检测一个或多个端口的连通性。
#[command]
pub async fn port_check(input: PortCheckInput) -> Result<PortCheckOutput, String> {
    let host = input.host.trim();
    if host.is_empty() {
        return Err("主机不能为空".into());
    }
    let ports = parse_ports(&input.ports)?;
    let timeout = input.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS).max(100);
    let total_start = Instant::now();
    // 并发探测,限制并发数避免一次扫太多端口打爆
    let mut results = Vec::with_capacity(ports.len());
    // 用 buffer_unordered 限制并发到 32
    use futures_util::stream::{self, StreamExt};
    let mapped = stream::iter(ports.iter().cloned()).map(|port| {
        let host = host.to_string();
        async move {
            let (open, latency_ms, error) = probe(&host, port, timeout).await;
            PortResult {
                port,
                open,
                latency_ms,
                error,
            }
        }
    });
    let mut stream = mapped.buffer_unordered(32);
    while let Some(r) = stream.next().await {
        results.push(r);
    }
    // 按端口排序输出,方便阅读
    results.sort_by_key(|r| r.port);
    Ok(PortCheckOutput {
        host: host.to_string(),
        results,
        total_ms: total_start.elapsed().as_millis() as u64,
    })
}
