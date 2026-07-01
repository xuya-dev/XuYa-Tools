import { ref } from 'vue';

export interface SystemStats {
  cpuUsage: number;
  cpuCount: number;
  memTotalGb: number;
  memUsedGb: number;
  memPercent: number;
  netRxKbps: number;
  netTxKbps: number;
  uptimeSecs: number;
}

const stats = ref<SystemStats | null>(null);
let timer: ReturnType<typeof setInterval> | null = null;
let activeCount = 0;

async function fetchStats() {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    stats.value = await invoke<SystemStats>('get_system_stats');
  } catch {
    /* non-Tauri */
  }
}

export function useSystemStats(intervalMs = 1000) {
  activeCount++;
  if (!timer) {
    fetchStats();
    timer = setInterval(fetchStats, intervalMs);
  }

  function stop() {
    activeCount--;
    if (activeCount <= 0 && timer) {
      clearInterval(timer);
      timer = null;
      activeCount = 0;
    }
  }

  return { stats, stop };
}

export function formatSpeed(kbps: number): string {
  if (kbps < 1) return kbps.toFixed(1) + ' KB/s';
  if (kbps < 1024) return kbps.toFixed(0) + ' KB/s';
  return (kbps / 1024).toFixed(1) + ' MB/s';
}

export function formatUptime(secs: number): string {
  const d = Math.floor(secs / 86400);
  const h = Math.floor((secs % 86400) / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (d > 0) return `${d}d ${h}h ${m}m`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}
