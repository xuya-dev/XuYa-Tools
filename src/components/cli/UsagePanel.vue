<template>
    <div class="usage-panel">
        <div class="usage-head">
            <span class="usage-title">请求统计</span>
            <div class="range-tabs">
                <button :class="{ active: statsTimeRange === 'today' }" @click="setStatsTimeRange('today')">今日</button>
                <button :class="{ active: statsTimeRange === '7d' }" @click="setStatsTimeRange('7d')">7 天</button>
                <button :class="{ active: statsTimeRange === '30d' }" @click="setStatsTimeRange('30d')">30 天</button>
            </div>
        </div>

        <!-- 统计卡片 -->
        <div v-if="usageSummary" class="stat-cards">
            <div class="stat-card">
                <div class="stat-val">{{ usageSummary.totalRequests }}</div>
                <div class="stat-lbl">总请求</div>
                <div class="stat-sub">
                    <span class="ok">{{ usageSummary.successCount }}</span> ·
                    <span class="err">{{ usageSummary.errorCount }}</span>
                </div>
            </div>
            <div class="stat-card">
                <div class="stat-val">{{ fmtRate(usageSummary.successRate) }}</div>
                <div class="stat-lbl">成功率</div>
            </div>
            <div class="stat-card">
                <div class="stat-val">{{ usageSummary.avgLatencyMs }}<span class="unit">ms</span></div>
                <div class="stat-lbl">平均延迟</div>
                <div v-if="usageSummary.avgFirstTokenMs > 0" class="stat-sub">首 Token {{ usageSummary.avgFirstTokenMs }}ms</div>
            </div>
            <div class="stat-card">
                <div class="stat-val">{{ fmtTokens(usageSummary.totalInputTokens + usageSummary.totalOutputTokens) }}</div>
                <div class="stat-lbl">总 Tokens</div>
                <div class="stat-sub">
                    入 {{ fmtTokens(usageSummary.totalInputTokens) }} · 出 {{ fmtTokens(usageSummary.totalOutputTokens) }}
                </div>
            </div>
            <div class="stat-card">
                <div class="stat-val">{{ fmtTokens(usageSummary.totalCacheReadTokens + usageSummary.totalCacheCreationTokens) }}</div>
                <div class="stat-lbl">缓存 Tokens</div>
                <div class="stat-sub">
                    读 {{ fmtTokens(usageSummary.totalCacheReadTokens) }} · 写 {{ fmtTokens(usageSummary.totalCacheCreationTokens) }}
                </div>
            </div>
            <div class="stat-card">
                <div class="stat-val">${{ usageSummary.totalCostUsd.toFixed(4) }}</div>
                <div class="stat-lbl">预估费用</div>
            </div>
        </div>
        <div v-else class="cli-empty">加载中…</div>

        <!-- GitHub 风格热力图 -->
        <div v-if="dailyStats.length" class="heatmap-section">
            <div class="section-label">请求热力图</div>
            <div class="heatmap">
                <div
                    v-for="day in heatmapData"
                    :key="day.date"
                    class="heatmap-cell"
                    :class="heatLevel(day.count)"
                    :title="`${day.date}: ${day.count} 次请求`"
                ></div>
            </div>
            <div class="heatmap-legend">
                <span>少</span>
                <div class="heatmap-cell l0"></div>
                <div class="heatmap-cell l1"></div>
                <div class="heatmap-cell l2"></div>
                <div class="heatmap-cell l3"></div>
                <div class="heatmap-cell l4"></div>
                <span>多</span>
            </div>
        </div>

        <!-- 每日趋势折线图 -->
        <div v-if="dailyStats.length > 1" class="chart-section">
            <div class="section-label">每日请求趋势</div>
            <svg class="trend-chart" :viewBox="`0 0 ${chartW} ${chartH}`" preserveAspectRatio="none">
                <!-- Y 轴网格线 -->
                <line v-for="i in 4" :key="'g'+i" :x1="0" :x2="chartW" :y1="(chartH/4)*i" :y2="(chartH/4)*i" stroke="var(--xuya-border)" stroke-width="0.5" stroke-dasharray="2,3" />
                <!-- 请求线 -->
                <polyline
                    :points="trendPoints"
                    fill="none"
                    stroke="var(--xuya-accent)"
                    stroke-width="2"
                    stroke-linejoin="round"
                />
                <!-- 请求面积 -->
                <polygon
                    :points="`0,${chartH} ${trendPoints} ${chartW},${chartH}`"
                    fill="var(--xuya-accent-soft)"
                    opacity="0.5"
                />
                <!-- Token 线 (输出) -->
                <polyline
                    :points="tokenTrendPoints"
                    fill="none"
                    stroke="var(--xuya-success)"
                    stroke-width="1.5"
                    stroke-dasharray="3,2"
                />
                <!-- 数据点 -->
                <circle v-for="(p, i) in trendDots" :key="'d'+i" :cx="p.x" :cy="p.y" r="2.5" fill="var(--xuya-accent)" />
            </svg>
            <div class="chart-legend">
                <span class="legend-dot accent"></span>请求数
                <span class="legend-dot success"></span>输出 Tokens (×1k)
            </div>
        </div>

        <!-- 最近请求列表 -->
        <div class="logs-head">
            <span>最近请求 <span class="logs-count">共 {{ logsTotal }} 条</span></span>
            <div class="logs-head-actions">
                <button v-if="requestLogs.length" class="mini-btn danger" @click="onClearLogs">清空</button>
            </div>
        </div>
        <div v-if="requestLogs.length === 0" class="cli-empty">暂无请求记录</div>
        <div v-else class="logs-table">
            <div class="logs-row logs-row-head">
                <span>时间</span>
                <span>应用</span>
                <span>Provider</span>
                <span>模型</span>
                <span>状态</span>
                <span>延迟</span>
                <span>Tokens</span>
                <span>缓存</span>
                <span>费用</span>
            </div>
            <div v-for="log in requestLogs" :key="log.id" class="logs-row">
                <span class="mono small">{{ fmtTime(log.createdAt) }}</span>
                <span>{{ log.appType }}</span>
                <span class="small">{{ log.providerName || '-' }}</span>
                <span class="small">{{ log.model || '-' }}</span>
                <span class="mono" :class="statusClass(log.statusCode)">{{ log.statusCode || 'ERR' }}</span>
                <span class="mono small">{{ log.latencyMs }}ms</span>
                <span class="mono small">{{ log.inputTokens + log.outputTokens }}</span>
                <span class="mono small">{{ (log.cacheReadTokens + log.cacheCreationTokens) || '-' }}</span>
                <span class="mono small">${{ log.totalCostUsd.toFixed(4) }}</span>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useCliConfig } from '@/composables/useCliConfig';

const {
    usageSummary,
    dailyStats,
    statsTimeRange,
    requestLogs,
    logsTotal,
    logsPage,
    refreshUsageSummary,
    refreshDailyStats,
    refreshRequestLogs,
    clearRequestLogs,
    setStatsTimeRange,
} = useCliConfig();

const fmtRate = (r: number) => (r * 100).toFixed(1) + '%';
const fmtTokens = (n: number) => {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return String(n);
};
const fmtTime = (ts: number) => {
    if (!ts) return '-';
    const d = new Date(ts * 1000);
    return d.toLocaleTimeString('zh-CN', { hour12: false }) + ' ' + (d.getMonth() + 1) + '/' + d.getDate();
};
const statusClass = (code: number) => {
    if (code === 0) return 'err';
    if (code < 400) return 'ok';
    return 'err';
};

async function onClearLogs() {
    if (!confirm('确定清空全部请求日志？此操作不可撤销。')) return;
    await clearRequestLogs();
}

// ===== 热力图 =====
const heatmapData = computed(() => {
    const map = new Map<string, number>();
    for (const d of dailyStats.value) {
        map.set(d.date, d.requestCount);
    }
    // 生成完整日期序列
    const days = statsTimeRange.value === 'today' ? 1 : statsTimeRange.value === '7d' ? 7 : 30;
    const result: { date: string; count: number }[] = [];
    const now = new Date();
    for (let i = days - 1; i >= 0; i--) {
        const d = new Date(now);
        d.setDate(d.getDate() - i);
        const dateStr = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
        result.push({ date: dateStr, count: map.get(dateStr) ?? 0 });
    }
    return result;
});

function heatLevel(count: number): string {
    if (count === 0) return 'l0';
    if (count <= 5) return 'l1';
    if (count <= 20) return 'l2';
    if (count <= 50) return 'l3';
    return 'l4';
}

// ===== 折线图 =====
const chartW = 600;
const chartH = 120;

const maxRequests = computed(() => Math.max(1, ...dailyStats.value.map((d) => d.requestCount)));
const maxTokens = computed(() => Math.max(1, ...dailyStats.value.map((d) => d.outputTokens)));

const trendPoints = computed(() => {
    const stats = dailyStats.value;
    if (stats.length < 2) return '';
    const step = chartW / (stats.length - 1);
    return stats.map((d, i) => `${(i * step).toFixed(1)},${(chartH - (d.requestCount / maxRequests.value) * chartH).toFixed(1)}`).join(' ');
});

const tokenTrendPoints = computed(() => {
    const stats = dailyStats.value;
    if (stats.length < 2) return '';
    const step = chartW / (stats.length - 1);
    return stats.map((d, i) => `${(i * step).toFixed(1)},${(chartH - Math.min(1, d.outputTokens / maxTokens.value) * chartH).toFixed(1)}`).join(' ');
});

const trendDots = computed(() => {
    const stats = dailyStats.value;
    if (stats.length < 2) return [];
    const step = chartW / (stats.length - 1);
    return stats.map((d, i) => ({
        x: i * step,
        y: chartH - (d.requestCount / maxRequests.value) * chartH,
    }));
});

// ===== 实时刷新 =====
let usageRefreshTimer: number | undefined;
let unlistenUsage: (() => void) | undefined;
const scheduleUsageRefresh = () => {
    if (usageRefreshTimer) window.clearTimeout(usageRefreshTimer);
    usageRefreshTimer = window.setTimeout(() => {
        refreshUsageSummary();
        refreshDailyStats();
        refreshRequestLogs(logsPage.value);
    }, 300);
};

onMounted(async () => {
    await Promise.all([refreshUsageSummary(), refreshDailyStats(), refreshRequestLogs(1)]);
    try {
        unlistenUsage = await listen('cli-usage-recorded', scheduleUsageRefresh);
    } catch { /* 非 Tauri 环境忽略 */ }
});
onUnmounted(() => {
    if (usageRefreshTimer) window.clearTimeout(usageRefreshTimer);
    unlistenUsage?.();
});
</script>

<style scoped>
.usage-panel {
    background: var(--xuya-card-bg);
    border: 1px solid var(--xuya-border);
    border-radius: var(--xuya-radius-lg);
    padding: 18px 20px;
}
.usage-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; }
.usage-title { font-size: 15px; font-weight: 700; color: var(--xuya-text); }
.range-tabs { display: inline-flex; gap: 2px; padding: 3px; background: var(--xuya-input-bg); border-radius: var(--xuya-radius); }
.range-tabs button { padding: 5px 14px; font-size: 12px; font-weight: 500; border: none; background: transparent; color: var(--xuya-text-secondary); border-radius: var(--xuya-radius-sm); cursor: pointer; transition: all var(--xuya-duration-fast) var(--xuya-ease); }
.range-tabs button.active { background: var(--xuya-bg-elevated); color: var(--xuya-accent); font-weight: 600; box-shadow: var(--xuya-shadow-sm); }

/* 统计卡片 */
.stat-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); gap: 10px; margin-bottom: 20px; }
.stat-card { display: flex; flex-direction: column; gap: 3px; padding: 14px 12px; background: var(--xuya-input-bg); border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border-light); }
.stat-val { font-size: 20px; font-weight: 700; color: var(--xuya-text); }
.stat-val .unit { font-size: 13px; font-weight: 500; opacity: 0.6; margin-left: 1px; }
.stat-lbl { font-size: 11px; color: var(--xuya-text-tertiary); }
.stat-sub { font-size: 10.5px; color: var(--xuya-text-tertiary); margin-top: 2px; }
.stat-sub .ok { color: var(--xuya-success); font-weight: 600; }
.stat-sub .err { color: var(--xuya-danger); font-weight: 600; }

/* 热力图 */
.heatmap-section { margin-bottom: 20px; }
.section-label { font-size: 12px; font-weight: 600; color: var(--xuya-text-secondary); margin-bottom: 10px; }
.heatmap { display: grid; grid-template-columns: repeat(auto-fill, minmax(12px, 1fr)); gap: 3px; }
.heatmap-cell { width: 12px; height: 12px; border-radius: 2px; }
.heatmap-cell.l0 { background: var(--xuya-border); }
.heatmap-cell.l1 { background: rgba(59, 130, 246, 0.25); }
.heatmap-cell.l2 { background: rgba(59, 130, 246, 0.5); }
.heatmap-cell.l3 { background: rgba(59, 130, 246, 0.75); }
.heatmap-cell.l4 { background: var(--xuya-accent); }
.heatmap-legend { display: flex; align-items: center; gap: 4px; margin-top: 8px; font-size: 10px; color: var(--xuya-text-tertiary); }
.heatmap-legend .heatmap-cell { width: 10px; height: 10px; }

/* 折线图 */
.chart-section { margin-bottom: 20px; }
.trend-chart { width: 100%; height: 120px; }
.chart-legend { display: flex; align-items: center; gap: 12px; margin-top: 6px; font-size: 11px; color: var(--xuya-text-tertiary); }
.legend-dot { display: inline-block; width: 10px; height: 3px; border-radius: 1px; margin-right: 2px; }
.legend-dot.accent { background: var(--xuya-accent); }
.legend-dot.success { background: var(--xuya-success); border-top: 1px dashed var(--xuya-success); }

/* 日志表格 */
.logs-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; font-size: 13px; font-weight: 600; color: var(--xuya-text); }
.logs-count { font-size: 11px; font-weight: 400; color: var(--xuya-text-tertiary); }
.logs-head-actions { display: flex; gap: 6px; }
.logs-table { border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius); overflow: hidden; }
.logs-row { display: grid; grid-template-columns: 120px 60px 1fr 100px 50px 60px 60px 50px 70px; gap: 6px; padding: 7px 10px; font-size: 11.5px; align-items: center; border-bottom: 1px solid var(--xuya-border-light); }
.logs-row:last-child { border-bottom: none; }
.logs-row-head { background: var(--xuya-input-bg); font-size: 10.5px; font-weight: 600; color: var(--xuya-text-tertiary); text-transform: uppercase; letter-spacing: 0.3px; }
.logs-row:not(.logs-row-head):hover { background: var(--xuya-input-bg); }
.logs-row .ok { color: var(--xuya-success); font-weight: 600; }
.logs-row .err { color: var(--xuya-danger); font-weight: 600; }

.cli-empty { padding: 24px; text-align: center; color: var(--xuya-text-tertiary); font-size: 13px; }
.mono { font-family: var(--xuya-font-mono); }
.small { font-size: 10.5px; color: var(--xuya-text-tertiary); }
.mini-btn { padding: 3px 10px; font-size: 11px; color: var(--xuya-text-secondary); background: transparent; border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-sm); cursor: pointer; transition: all var(--xuya-duration-fast) var(--xuya-ease); }
.mini-btn.danger:hover { color: var(--xuya-danger); border-color: var(--xuya-danger); background: var(--xuya-danger-soft); }
</style>
