<template>
  <div class="usage-panel">
    <div class="usage-head">
      <span class="usage-title">请求统计</span>
      <div class="range-tabs">
        <button :class="{ active: statsTimeRange === 'today' }" @click="setStatsTimeRange('today')">
          今日
        </button>
        <button :class="{ active: statsTimeRange === '7d' }" @click="setStatsTimeRange('7d')">
          7 天
        </button>
        <button :class="{ active: statsTimeRange === '30d' }" @click="setStatsTimeRange('30d')">
          30 天
        </button>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div v-if="usageSummary" class="stat-cards">
      <div class="stat-card">
        <div class="stat-val">{{ usageSummary.totalRequests }}</div>
        <div class="stat-lbl">总请求</div>
        <div class="stat-sub">
          <span class="ok">{{ usageSummary.successCount }}</span>
          成功 ·
          <span class="err">{{ usageSummary.errorCount }}</span>
          失败
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-val">{{ fmtRate(usageSummary.successRate) }}</div>
        <div class="stat-lbl">成功率</div>
      </div>
      <div class="stat-card">
        <div class="stat-val">
          {{ usageSummary.avgLatencyMs }}
          <span class="unit">ms</span>
        </div>
        <div class="stat-lbl">平均延迟</div>
        <div v-if="usageSummary.avgFirstTokenMs > 0" class="stat-sub">
          首 Token {{ usageSummary.avgFirstTokenMs }}ms
        </div>
      </div>
      <div class="stat-card cost">
        <div class="stat-val">${{ usageSummary.totalCostUsd.toFixed(4) }}</div>
        <div class="stat-lbl">总费用</div>
        <div class="stat-sub">${{ avgCostPerReq.toFixed(4) }}/次</div>
      </div>
      <div class="stat-card input">
        <div class="stat-val">{{ fmtTokens(usageSummary.totalInputTokens) }}</div>
        <div class="stat-lbl">输入 Tokens</div>
      </div>
      <div class="stat-card output">
        <div class="stat-val">{{ fmtTokens(usageSummary.totalOutputTokens) }}</div>
        <div class="stat-lbl">输出 Tokens</div>
        <div class="stat-sub">{{ avgTokensPerReq.toFixed(0) }} Tokens/次</div>
      </div>
      <div class="stat-card cache-read">
        <div class="stat-val">{{ fmtTokens(usageSummary.totalCacheReadTokens) }}</div>
        <div class="stat-lbl">缓存读</div>
        <div class="stat-sub">命中率 {{ cacheHitRate.toFixed(1) }}%</div>
      </div>
      <div class="stat-card cache-write">
        <div class="stat-val">{{ fmtTokens(usageSummary.totalCacheCreationTokens) }}</div>
        <div class="stat-lbl">缓存写</div>
      </div>
    </div>
    <div v-else class="cli-empty">加载中…</div>

    <!-- 多维度分析 -->
    <div v-if="requestLogs.length" class="breakdown-section">
      <div class="breakdown-grid">
        <!-- Provider 分布 -->
        <div class="breakdown-card">
          <div class="breakdown-title">供应商分布</div>
          <div class="breakdown-list">
            <div v-for="p in providerStats" :key="p.name" class="breakdown-row">
              <span class="bd-name">{{ p.name }}</span>
              <div class="bd-bar-wrap">
                <div
                  class="bd-bar"
                  :style="{ width: barPct(p.requests, maxProviderReq) + '%' }"
                ></div>
              </div>
              <span class="bd-num">{{ p.requests }}</span>
              <span class="bd-cost">${{ p.cost.toFixed(4) }}</span>
            </div>
          </div>
        </div>
        <!-- 模型分布 -->
        <div class="breakdown-card">
          <div class="breakdown-title">模型分布</div>
          <div class="breakdown-list">
            <div v-for="m in modelStats" :key="m.name" class="breakdown-row">
              <span class="bd-name mono">{{ m.name }}</span>
              <div class="bd-bar-wrap">
                <div
                  class="bd-bar success"
                  :style="{ width: barPct(m.requests, maxModelReq) + '%' }"
                ></div>
              </div>
              <span class="bd-num">{{ m.requests }}</span>
              <span class="bd-cost">${{ m.cost.toFixed(4) }}</span>
            </div>
          </div>
        </div>
        <!-- 应用分布 -->
        <div class="breakdown-card">
          <div class="breakdown-title">应用分布</div>
          <div class="breakdown-list">
            <div v-for="a in appStats" :key="a.name" class="breakdown-row">
              <span class="bd-name">{{ a.name }}</span>
              <div class="bd-bar-wrap">
                <div
                  class="bd-bar warn"
                  :style="{ width: barPct(a.requests, maxAppReq) + '%' }"
                ></div>
              </div>
              <span class="bd-num">{{ a.requests }}</span>
              <span class="bd-cost">${{ a.cost.toFixed(4) }}</span>
            </div>
          </div>
        </div>
      </div>
      <div class="breakdown-hint">基于最近 {{ requestLogs.length }} 条请求记录</div>
    </div>

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
      <div class="section-label">每日趋势</div>
      <svg class="trend-chart" :viewBox="`0 0 ${chartW} ${chartH}`" preserveAspectRatio="none">
        <line
          v-for="i in 4"
          :key="'g' + i"
          :x1="0"
          :x2="chartW"
          :y1="(chartH / 4) * i"
          :y2="(chartH / 4) * i"
          stroke="var(--xuya-border)"
          stroke-width="0.5"
          stroke-dasharray="2,3"
        />
        <polygon
          :points="`0,${chartH} ${trendPoints} ${chartW},${chartH}`"
          fill="var(--xuya-accent-soft)"
          opacity="0.4"
        />
        <polyline
          :points="trendPoints"
          fill="none"
          stroke="var(--xuya-accent)"
          stroke-width="2"
          stroke-linejoin="round"
        />
        <polyline
          :points="costTrendPoints"
          fill="none"
          stroke="var(--xuya-warn)"
          stroke-width="1.5"
          stroke-dasharray="3,2"
        />
        <polyline
          :points="tokenTrendPoints"
          fill="none"
          stroke="var(--xuya-success)"
          stroke-width="1.5"
          stroke-dasharray="3,2"
        />
        <circle
          v-for="(p, i) in trendDots"
          :key="'d' + i"
          :cx="p.x"
          :cy="p.y"
          r="2.5"
          fill="var(--xuya-accent)"
        />
      </svg>
      <div class="chart-legend">
        <span class="legend-dot accent"></span>
        请求数
        <span class="legend-dot warn"></span>
        费用 ($)
        <span class="legend-dot success"></span>
        输出 Tokens (×1k)
      </div>
    </div>

    <!-- 最近请求列表 -->
    <div class="logs-head">
      <span>
        最近请求
        <span class="logs-count">共 {{ logsTotal }} 条</span>
      </span>
      <div class="logs-head-actions">
        <button v-if="requestLogs.length" class="mini-btn danger" @click="onClearLogs">清空</button>
      </div>
    </div>
    <div v-if="requestLogs.length === 0" class="cli-empty">暂无请求记录</div>
    <div v-else class="logs-scroll">
      <div class="logs-table">
        <div class="logs-row logs-row-head">
          <span>时间</span>
          <span>应用</span>
          <span>供应商</span>
          <span>模型</span>
          <span>状态</span>
          <span>延迟</span>
          <span>输入</span>
          <span>输出</span>
          <span>缓存读</span>
          <span>缓存写</span>
          <span>费用</span>
        </div>
        <div v-for="log in requestLogs" :key="log.id" class="logs-row">
          <span class="mono small">{{ fmtTime(log.createdAt) }}</span>
          <span>{{ log.appType }}</span>
          <span class="small">{{ log.providerName || '-' }}</span>
          <span class="small">{{ log.model || '-' }}</span>
          <span class="mono" :class="statusClass(log.statusCode)">
            {{ log.statusCode || 'ERR' }}
          </span>
          <span class="mono small">{{ log.latencyMs }}ms</span>
          <span class="mono small">{{ log.inputTokens || '-' }}</span>
          <span class="mono small">{{ log.outputTokens || '-' }}</span>
          <span class="mono small">{{ log.cacheReadTokens || '-' }}</span>
          <span class="mono small">{{ log.cacheCreationTokens || '-' }}</span>
          <span class="mono small">${{ log.totalCostUsd.toFixed(4) }}</span>
        </div>
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
  return (
    d.toLocaleTimeString('zh-CN', { hour12: false }) + ' ' + (d.getMonth() + 1) + '/' + d.getDate()
  );
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

// ===== 衍生指标 =====
const cacheHitRate = computed(() => {
  if (!usageSummary.value) return 0;
  const total = usageSummary.value.totalCacheReadTokens + usageSummary.value.totalInputTokens;
  return total === 0 ? 0 : (usageSummary.value.totalCacheReadTokens / total) * 100;
});
const avgCostPerReq = computed(() => {
  if (!usageSummary.value || usageSummary.value.totalRequests === 0) return 0;
  return usageSummary.value.totalCostUsd / usageSummary.value.totalRequests;
});
const avgTokensPerReq = computed(() => {
  if (!usageSummary.value || usageSummary.value.totalRequests === 0) return 0;
  return (
    (usageSummary.value.totalInputTokens + usageSummary.value.totalOutputTokens) /
    usageSummary.value.totalRequests
  );
});

// ===== 多维度聚合 (基于当前页日志) =====
interface AggStat {
  name: string;
  requests: number;
  cost: number;
  inputTokens: number;
  outputTokens: number;
}

function aggregate(
  keyFn: (log: { providerName: string | null; model: string | null; appType: string }) => string,
): AggStat[] {
  const map = new Map<string, AggStat>();
  for (const log of requestLogs.value) {
    const key = keyFn(log) || '未知';
    const entry = map.get(key) ?? {
      name: key,
      requests: 0,
      cost: 0,
      inputTokens: 0,
      outputTokens: 0,
    };
    entry.requests++;
    entry.cost += log.totalCostUsd;
    entry.inputTokens += log.inputTokens;
    entry.outputTokens += log.outputTokens;
    map.set(key, entry);
  }
  return [...map.values()].sort((a, b) => b.requests - a.requests);
}

const providerStats = computed(() => aggregate((l) => l.providerName || ''));
const modelStats = computed(() => aggregate((l) => l.model || ''));
const appStats = computed(() => aggregate((l) => l.appType));

const maxProviderReq = computed(() => Math.max(1, ...providerStats.value.map((p) => p.requests)));
const maxModelReq = computed(() => Math.max(1, ...modelStats.value.map((m) => m.requests)));
const maxAppReq = computed(() => Math.max(1, ...appStats.value.map((a) => a.requests)));

function barPct(val: number, max: number): number {
  return Math.max(3, (val / max) * 100);
}

// ===== 热力图 =====
const heatmapData = computed(() => {
  const map = new Map<string, number>();
  for (const d of dailyStats.value) map.set(d.date, d.requestCount);
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
const maxCost = computed(() => Math.max(0.0001, ...dailyStats.value.map((d) => d.costUsd)));

const trendPoints = computed(() => {
  const stats = dailyStats.value;
  if (stats.length < 2) return '';
  const step = chartW / (stats.length - 1);
  return stats
    .map(
      (d, i) =>
        `${(i * step).toFixed(1)},${(chartH - (d.requestCount / maxRequests.value) * chartH).toFixed(1)}`,
    )
    .join(' ');
});
const tokenTrendPoints = computed(() => {
  const stats = dailyStats.value;
  if (stats.length < 2) return '';
  const step = chartW / (stats.length - 1);
  return stats
    .map(
      (d, i) =>
        `${(i * step).toFixed(1)},${(chartH - Math.min(1, d.outputTokens / maxTokens.value) * chartH).toFixed(1)}`,
    )
    .join(' ');
});
const costTrendPoints = computed(() => {
  const stats = dailyStats.value;
  if (stats.length < 2) return '';
  const step = chartW / (stats.length - 1);
  return stats
    .map(
      (d, i) =>
        `${(i * step).toFixed(1)},${(chartH - (d.costUsd / maxCost.value) * chartH).toFixed(1)}`,
    )
    .join(' ');
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
  } catch {
    /* 非 Tauri 环境忽略 */
  }
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
.usage-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}
.usage-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--xuya-text);
}
.range-tabs {
  display: inline-flex;
  gap: 2px;
  padding: 3px;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius);
}
.range-tabs button {
  padding: 5px 14px;
  font-size: 12px;
  font-weight: 500;
  border: none;
  background: transparent;
  color: var(--xuya-text-secondary);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
}
.range-tabs button.active {
  background: var(--xuya-bg-elevated);
  color: var(--xuya-accent);
  font-weight: 600;
  box-shadow: var(--xuya-shadow-sm);
}

.stat-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 8px;
  margin-bottom: 20px;
}
.stat-card {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 12px;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border-light);
}
.stat-card.input .stat-val {
  color: var(--xuya-accent);
}
.stat-card.output .stat-val {
  color: var(--xuya-success);
}
.stat-card.cache-read .stat-val {
  color: var(--xuya-text-secondary);
}
.stat-card.cache-write .stat-val {
  color: var(--xuya-warn);
}
.stat-card.cost .stat-val {
  color: var(--xuya-text);
}
.stat-val {
  font-size: 18px;
  font-weight: 700;
  color: var(--xuya-text);
}
.stat-val .unit {
  font-size: 12px;
  font-weight: 500;
  opacity: 0.6;
  margin-left: 1px;
}
.stat-lbl {
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
}
.stat-sub {
  font-size: 10px;
  color: var(--xuya-text-tertiary);
  margin-top: 2px;
}
.stat-sub .ok {
  color: var(--xuya-success);
  font-weight: 600;
}
.stat-sub .err {
  color: var(--xuya-danger);
  font-weight: 600;
}

/* 多维度分析 */
.breakdown-section {
  margin-bottom: 20px;
}
.breakdown-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 14px;
}
.breakdown-card {
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border-light);
  border-radius: var(--xuya-radius);
  padding: 12px 14px;
}
.breakdown-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
  margin-bottom: 10px;
}
.breakdown-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.breakdown-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 60px 24px 56px;
  gap: 6px;
  align-items: center;
  font-size: 11.5px;
}
.bd-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--xuya-text);
}
.bd-bar-wrap {
  grid-column: 1;
  height: 18px;
  position: relative;
}
.bd-bar {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  border-radius: 3px;
  background: var(--xuya-accent-soft-strong);
  transition: width var(--xuya-duration);
}
.bd-bar.success {
  background: var(--xuya-success-soft);
}
.bd-bar.warn {
  background: var(--xuya-warn-soft);
}
.bd-num {
  font-family: var(--xuya-font-mono);
  font-weight: 600;
  color: var(--xuya-text);
  text-align: right;
}
.bd-cost {
  font-family: var(--xuya-font-mono);
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
  text-align: right;
}
.breakdown-hint {
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
  margin-top: 8px;
}

/* 热力图 */
.heatmap-section {
  margin-bottom: 20px;
}
.section-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
  margin-bottom: 10px;
}
.heatmap {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(12px, 1fr));
  gap: 3px;
}
.heatmap-cell {
  width: 12px;
  height: 12px;
  border-radius: 2px;
}
.heatmap-cell.l0 {
  background: var(--xuya-border);
}
.heatmap-cell.l1 {
  background: var(--xuya-accent-soft);
}
.heatmap-cell.l2 {
  background: var(--xuya-accent-soft-strong);
}
.heatmap-cell.l3 {
  background: var(--xuya-accent);
}
.heatmap-cell.l4 {
  background: var(--xuya-accent);
}
.heatmap-legend {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 8px;
  font-size: 10px;
  color: var(--xuya-text-tertiary);
}
.heatmap-legend .heatmap-cell {
  width: 10px;
  height: 10px;
}

/* 折线图 */
.chart-section {
  margin-bottom: 20px;
}
.trend-chart {
  width: 100%;
  height: 120px;
}
.chart-legend {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 6px;
  font-size: 11px;
  color: var(--xuya-text-tertiary);
}
.legend-dot {
  display: inline-block;
  width: 10px;
  height: 3px;
  border-radius: 1px;
  margin-right: 2px;
}
.legend-dot.accent {
  background: var(--xuya-accent);
}
.legend-dot.success {
  background: var(--xuya-success);
  border-top: 1px dashed var(--xuya-success);
}
.legend-dot.warn {
  background: var(--xuya-warn);
  border-top: 1px dashed var(--xuya-warn);
}

/* 日志表格 */
.logs-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
}
.logs-count {
  font-size: 11px;
  font-weight: 400;
  color: var(--xuya-text-tertiary);
}
.logs-head-actions {
  display: flex;
  gap: 6px;
}
.logs-scroll {
  overflow-x: auto;
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.logs-table {
  min-width: 760px;
}
.logs-row {
  display: grid;
  grid-template-columns: 100px 60px 1fr 1fr 50px 60px 60px 60px 60px 60px 70px;
  gap: 5px;
  padding: 7px 10px;
  font-size: 11px;
  align-items: center;
  border-bottom: 1px solid var(--xuya-border-light);
}
.logs-row:last-child {
  border-bottom: none;
}
.logs-row-head {
  background: var(--xuya-input-bg);
  font-size: 10.5px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}
.logs-row:not(.logs-row-head):hover {
  background: var(--xuya-input-bg);
}
.logs-row .ok {
  color: var(--xuya-success);
  font-weight: 600;
}
.logs-row .err {
  color: var(--xuya-danger);
  font-weight: 600;
}

.cli-empty {
  padding: 24px;
  text-align: center;
  color: var(--xuya-text-tertiary);
  font-size: 13px;
}
.mono {
  font-family: var(--xuya-font-mono);
}
.small {
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
}
.mini-btn {
  padding: 3px 10px;
  font-size: 11px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
}
.mini-btn.danger:hover {
  color: var(--xuya-danger);
  border-color: var(--xuya-danger);
  background: var(--xuya-danger-soft);
}

@media (max-width: 700px) {
  .breakdown-grid {
    grid-template-columns: 1fr;
  }
}
</style>
