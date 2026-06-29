<template>
    <div class="usage-panel">
        <div class="usage-head">
            <span class="usage-title">请求统计</span>
            <button class="cli-refresh-btn" @click="onRefresh">刷新</button>
        </div>

        <div v-if="usageSummary" class="usage-grid">
            <div class="usage-stat">
                <span class="usage-stat-val">{{ usageSummary.totalRequests }}</span>
                <span class="usage-stat-label">总请求</span>
            </div>
            <div class="usage-stat">
                <span class="usage-stat-val ok">{{ usageSummary.successCount }}</span>
                <span class="usage-stat-label">成功</span>
            </div>
            <div class="usage-stat">
                <span class="usage-stat-val err">{{ usageSummary.errorCount }}</span>
                <span class="usage-stat-label">失败</span>
            </div>
            <div class="usage-stat">
                <span class="usage-stat-val">{{ fmtRate(usageSummary.successRate) }}</span>
                <span class="usage-stat-label">成功率</span>
            </div>
            <div class="usage-stat">
                <span class="usage-stat-val">{{ usageSummary.avgLatencyMs }}ms</span>
                <span class="usage-stat-label">平均延迟</span>
            </div>
            <div class="usage-stat">
                <span class="usage-stat-val">{{ usageSummary.totalInputTokens + usageSummary.totalOutputTokens }}</span>
                <span class="usage-stat-label">Tokens</span>
            </div>
            <div v-if="usageSummary.totalCostUsd > 0" class="usage-stat">
                <span class="usage-stat-val">${{ usageSummary.totalCostUsd.toFixed(4) }}</span>
                <span class="usage-stat-label">预估费用</span>
            </div>
        </div>
        <div v-else class="cli-empty">加载中…</div>

        <!-- 最近请求列表 -->
        <div class="logs-head">
            <span>最近请求 <span class="logs-count">共 {{ logsTotal }} 条</span></span>
            <div class="logs-head-actions">
                <span class="logs-page">第 {{ logsPage }} 页</span>
                <button v-if="logsTotal > logsPageSize" class="cli-mini-btn" :disabled="logsPage <= 1" @click="prevPage">上一页</button>
                <button v-if="logsTotal > logsPageSize" class="cli-mini-btn" :disabled="logsPage * logsPageSize >= logsTotal" @click="nextPage">下一页</button>
                <button v-if="requestLogs.length" class="cli-mini-btn danger" @click="onClearLogs">清空</button>
            </div>
        </div>
        <div v-if="requestLogs.length === 0" class="cli-empty">暂无请求记录（启动代理并接管后产生）</div>
        <div v-else class="logs-table">
            <div class="logs-row logs-row-head">
                <span>时间</span>
                <span>应用</span>
                <span>Provider</span>
                <span>模型</span>
                <span>状态</span>
                <span>延迟</span>
                <span>Tokens</span>
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
                <span class="mono small">${{ log.totalCostUsd.toFixed(4) }}</span>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, reactive } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useCliConfig } from '@/composables/useCliConfig';

const {
    usageSummary,
    requestLogs,
    logsTotal,
    logsPage,
    logsPageSize,
    refreshUsageSummary,
    refreshRequestLogs,
    clearRequestLogs,
} = useCliConfig();

// 局部 toast
const toast = reactive({ visible: false, message: '', type: 'success' });
let toastTimer: number | undefined;
function showToast(message: string, type: 'success' | 'error' = 'success') {
    toast.message = message;
    toast.type = type;
    toast.visible = true;
    if (toastTimer) window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => { toast.visible = false; }, 2500);
}
void toast;

const fmtRate = (r: number) => (r * 100).toFixed(1) + '%';
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

async function onRefresh() {
    await Promise.all([refreshUsageSummary(), refreshRequestLogs(logsPage.value)]);
}
async function prevPage() { if (logsPage.value > 1) await refreshRequestLogs(logsPage.value - 1); }
async function nextPage() { await refreshRequestLogs(logsPage.value + 1); }
async function onClearLogs() {
    if (!confirm('确定清空全部请求日志？此操作不可撤销。')) return;
    try {
        await clearRequestLogs();
        showToast('日志已清空', 'success');
    } catch (e) {
        showToast('清空失败: ' + e, 'error');
    }
}

// 实时刷新:后端 emit cli-usage-recorded 时防抖刷新
let usageRefreshTimer: number | undefined;
let unlistenUsage: (() => void) | undefined;
const scheduleUsageRefresh = () => {
    if (usageRefreshTimer) window.clearTimeout(usageRefreshTimer);
    usageRefreshTimer = window.setTimeout(() => {
        refreshUsageSummary();
        refreshRequestLogs(logsPage.value);
    }, 300);
};

onMounted(async () => {
    await Promise.all([refreshUsageSummary(), refreshRequestLogs(1)]);
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
    background: var(--xuya-card-bg, rgba(127,127,127,.08));
    border: 1px solid var(--xuya-border, rgba(127,127,127,.15));
    border-radius: 12px;
    padding: 18px 20px;
}
.usage-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; }
.usage-title { font-size: 15px; font-weight: 700; }
.cli-refresh-btn { padding: 4px 12px; font-size: 12px; border-radius: 6px; cursor: pointer; background: var(--xuya-card-bg, transparent); color: var(--xuya-text, inherit); border: 1px solid var(--xuya-border, rgba(127,127,127,.2)); transition: background .15s; }
.cli-refresh-btn:hover { background: rgba(59,130,246,.1); color: #3b82f6; }

.usage-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: 10px; margin-bottom: 18px; }
.usage-stat { display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 14px 8px; background: var(--xuya-input-bg, rgba(127,127,127,.06)); border-radius: 10px; }
.usage-stat-val { font-size: 20px; font-weight: 700; color: var(--xuya-text, inherit); }
.usage-stat-val.ok { color: #34d399; }
.usage-stat-val.err { color: #f87171; }
.usage-stat-label { font-size: 11px; color: var(--xuya-text-3, #888); }

.logs-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; font-size: 13px; font-weight: 600; }
.logs-count { font-size: 11px; font-weight: 400; color: var(--xuya-text-3, #888); }
.logs-head-actions { display: flex; align-items: center; gap: 6px; }
.logs-page { font-size: 11px; color: var(--xuya-text-3, #888); }

.logs-table { border: 1px solid var(--xuya-border, rgba(127,127,127,.12)); border-radius: 8px; overflow: hidden; }
.logs-row { display: grid; grid-template-columns: 130px 70px 1fr 110px 60px 70px 70px 80px; gap: 8px; padding: 8px 12px; font-size: 12px; align-items: center; border-bottom: 1px solid var(--xuya-border-light, rgba(127,127,127,.08)); }
.logs-row:last-child { border-bottom: none; }
.logs-row-head { background: var(--xuya-input-bg, rgba(127,127,127,.06)); font-size: 11px; font-weight: 600; color: var(--xuya-text-3, #888); }
.logs-row:not(.logs-row-head):hover { background: var(--xuya-input-bg, rgba(127,127,127,.04)); }
.logs-row .ok { color: #34d399; font-weight: 600; }
.logs-row .err { color: #f87171; font-weight: 600; }

.cli-empty { padding: 24px; text-align: center; color: var(--xuya-text-3, #888); font-size: 13px; }
.mono { font-family: var(--xuya-font-mono, 'Consolas', monospace); }
.small { font-size: 11px; color: var(--xuya-text-2, #888); }
.cli-mini-btn { padding: 3px 10px; font-size: 11px; color: var(--xuya-text-2, #888); background: var(--xuya-bg-elevated, rgba(127,127,127,.1)); border: 1px solid var(--xuya-border, rgba(127,127,127,.2)); border-radius: 6px; transition: .1s; }
.cli-mini-btn:hover:not(:disabled) { color: var(--xuya-text, inherit); border-color: var(--xuya-accent, #3b82f6); }
.cli-mini-btn:disabled { opacity: .4; cursor: not-allowed; }
.cli-mini-btn.danger:hover { color: #f87171; border-color: #f87171; }
</style>
