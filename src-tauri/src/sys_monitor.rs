use serde::Serialize;
use std::sync::Mutex;
use sysinfo::{Networks, System};
use tauri::State;

pub struct SysState {
    pub sys: Mutex<System>,
    pub nets: Mutex<Networks>,
    pub prev_rx: Mutex<u64>,
    pub prev_tx: Mutex<u64>,
}

impl SysState {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys: Mutex::new(sys),
            nets: Mutex::new(Networks::new()),
            prev_rx: Mutex::new(0),
            prev_tx: Mutex::new(0),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub cpu_count: usize,
    pub mem_total_gb: f64,
    pub mem_used_gb: f64,
    pub mem_percent: f64,
    pub net_rx_kbps: f64,
    pub net_tx_kbps: f64,
    pub uptime_secs: u64,
}

#[tauri::command]
pub fn get_system_stats(state: State<SysState>) -> SystemStats {
    let mut sys = state.sys.lock().expect("sys 锁中毒");
    let mut nets = state.nets.lock().expect("nets 锁中毒");

    sys.refresh_cpu_usage();
    sys.refresh_memory();

    let cpu_usage = sys.global_cpu_usage();
    let cpu_count = sys.cpus().len();
    let mem_total = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let mem_used = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let mem_percent = if mem_total > 0.0 {
        (mem_used / mem_total) * 100.0
    } else {
        0.0
    };

    nets.refresh();
    let mut total_rx: u64 = 0;
    let mut total_tx: u64 = 0;
    for (_, data) in &*nets {
        total_rx += data.received();
        total_tx += data.transmitted();
    }

    let mut prev_rx = state.prev_rx.lock().expect("rx 锁中毒");
    let mut prev_tx = state.prev_tx.lock().expect("tx 锁中毒");

    let diff_rx = total_rx.saturating_sub(*prev_rx);
    let diff_tx = total_tx.saturating_sub(*prev_tx);
    *prev_rx = total_rx;
    *prev_tx = total_tx;

    let net_rx_kbps = (diff_rx as f64) / 1024.0;
    let net_tx_kbps = (diff_tx as f64) / 1024.0;

    let uptime = System::uptime();

    SystemStats {
        cpu_usage,
        cpu_count,
        mem_total_gb: mem_total,
        mem_used_gb: mem_used,
        mem_percent,
        net_rx_kbps,
        net_tx_kbps,
        uptime_secs: uptime,
    }
}
