<template>
    <div class="proxy-panel">
        <div class="proxy-head">
            <div class="proxy-head-info">
                <span class="proxy-title">本地代理</span>
                <span class="proxy-status-dot" :class="{ on: proxyStatus?.running }"></span>
                <span class="proxy-status-text">{{ proxyStatus?.running ? '运行中' : '已停止' }}</span>
            </div>
            <button
                class="proxy-toggle-btn"
                :class="{ running: proxyStatus?.running }"
                :disabled="proxyBusy"
                @click="onToggleProxy"
            >{{ proxyStatus?.running ? '停止' : '启动' }}</button>
        </div>

        <div class="proxy-detail" v-if="proxyStatus?.running">
            <div class="proxy-detail-row">
                <span class="proxy-detail-label">地址</span>
                <span class="proxy-detail-val mono">{{ proxyUrl }}</span>
                <button class="cli-mini-btn ghost" @click="copy(proxyUrl)">复制</button>
            </div>
            <div class="proxy-detail-row" v-if="proxyStatus.active_provider_name">
                <span class="proxy-detail-label">上游</span>
                <span class="proxy-detail-val">{{ proxyStatus.active_provider_name }}</span>
            </div>
            <div class="proxy-detail-row" v-if="proxyStatus.started_at">
                <span class="proxy-detail-label">启动于</span>
                <span class="proxy-detail-val mono">{{ startedAtText }}</span>
            </div>
        </div>

        <div class="proxy-takeover" v-if="proxyStatus?.running">
            <div class="proxy-takeover-title">接管 CLI (将 CLI 流量导向代理)</div>
            <div class="proxy-takeover-item">
                <span>Claude Code 接管</span>
                <label class="switch">
                    <input
                        type="checkbox"
                        :checked="proxyStatus.claude_taken_over"
                        @change="onToggleTakeover('claude', ($event.target as HTMLInputElement).checked)"
                    >
                    <span class="slider"></span>
                </label>
            </div>
            <div class="proxy-takeover-item">
                <span>Codex CLI 接管</span>
                <label class="switch">
                    <input
                        type="checkbox"
                        :checked="proxyStatus.codex_taken_over"
                        @change="onToggleTakeover('codex', ($event.target as HTMLInputElement).checked)"
                    >
                    <span class="slider"></span>
                </label>
            </div>
        </div>

        <div class="proxy-hint" v-if="!proxyStatus?.running">
            启动本地代理后,可接管 Claude Code / Codex CLI,将它们的请求导向你配置的供应商上游,并记录请求日志用于统计。
        </div>

        <div class="proxy-quick-target" v-if="proxyStatus?.running && providers.length">
            <div class="proxy-quick-title">快速设为上游</div>
            <div class="proxy-quick-list">
                <button
                    v-for="p in targetableProviders"
                    :key="p.id"
                    class="proxy-quick-item"
                    :class="{ active: proxyStatus?.active_provider_id === p.id }"
                    @click="onSetTarget(p)"
                >
                    {{ p.name }}
                    <span class="proxy-quick-url mono" v-if="p.base_url">{{ p.base_url }}</span>
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive } from 'vue';
import { useCliConfig, type AppType, type CliProvider } from '@/composables/useCliConfig';
import { copyToClipboard } from '@/composables/useClipboard';

const {
    proxyStatus,
    proxyBusy,
    refreshProxyStatus,
    startProxy,
    stopProxy,
    setTakeover,
    switchProxyTarget,
    providers,
} = useCliConfig();

// 局部 toast (本面板独立提示)
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

const proxyUrl = computed(() => {
    const p = proxyStatus.value;
    if (!p?.running || !p.port) return '未运行';
    return `http://${p.address}:${p.port}`;
});

const startedAtText = computed(() => {
    if (!proxyStatus.value?.started_at) return '-';
    return new Date(proxyStatus.value.started_at * 1000).toLocaleTimeString('zh-CN', { hour12: false });
});

const targetableProviders = computed(() => providers.value.filter((p) => p.base_url));

async function onToggleProxy() {
    try {
        if (proxyStatus.value?.running) {
            await stopProxy();
            showToast('代理已停止', 'success');
        } else {
            const info = await startProxy();
            showToast(`代理已启动 @${info.port}`, 'success');
        }
    } catch (e) {
        showToast('代理操作失败: ' + e, 'error');
    }
}

async function onToggleTakeover(app: AppType, enabled: boolean) {
    try {
        const result = await setTakeover(app, enabled);
        showToast(result.message, result.success ? 'success' : 'error');
        if (!result.success) {
            await refreshProxyStatus();
        }
    } catch (e) {
        showToast('接管操作失败: ' + e, 'error');
    }
}

async function onSetTarget(p: CliProvider) {
    try {
        await switchProxyTarget(p);
        showToast(`上游已设为 ${p.name}`, 'success');
    } catch (e) {
        showToast('设置上游失败: ' + e, 'error');
    }
}

async function copy(text: string) {
    if (text && text !== '未运行') await copyToClipboard(text, '已复制代理地址');
}

onMounted(() => { refreshProxyStatus(); });
onUnmounted(() => { if (toastTimer) window.clearTimeout(toastTimer); });
</script>

<style scoped>
.proxy-panel {
    background: var(--xuya-card-bg, rgba(127,127,127,.08));
    border: 1px solid var(--xuya-border, rgba(127,127,127,.15));
    border-radius: 12px;
    padding: 18px 20px;
}
.proxy-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; }
.proxy-head-info { display: flex; align-items: center; gap: 8px; }
.proxy-title { font-size: 15px; font-weight: 700; }
.proxy-status-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--xuya-text-3, #888); }
.proxy-status-dot.on { background: #34d399; box-shadow: 0 0 8px rgba(52,211,153,.6); }
.proxy-status-text { font-size: 12px; color: var(--xuya-text-2, #888); }
.proxy-toggle-btn {
    padding: 7px 18px; font-size: 13px; font-weight: 600; border-radius: 8px; cursor: pointer;
    background: var(--xuya-accent, #3b82f6); color: #fff; border: none; transition: .12s;
}
.proxy-toggle-btn:hover:not(:disabled) { filter: brightness(1.1); }
.proxy-toggle-btn.running { background: var(--xuya-danger, #ef4444); }
.proxy-toggle-btn:disabled { opacity: .5; cursor: not-allowed; }

.proxy-detail { display: flex; flex-direction: column; gap: 8px; padding: 12px 0; border-top: 1px solid var(--xuya-border, rgba(127,127,127,.12)); }
.proxy-detail-row { display: flex; align-items: center; gap: 10px; font-size: 13px; }
.proxy-detail-label { width: 50px; color: var(--xuya-text-3, #888); }
.proxy-detail-val { flex: 1; color: var(--xuya-text, inherit); word-break: break-all; }

.proxy-takeover { margin-top: 14px; padding-top: 14px; border-top: 1px solid var(--xuya-border, rgba(127,127,127,.12)); }
.proxy-takeover-title { font-size: 12px; color: var(--xuya-text-3, #888); margin-bottom: 10px; }
.proxy-takeover-item { display: flex; align-items: center; justify-content: space-between; padding: 8px 0; font-size: 13px; }
.switch { position: relative; display: inline-block; width: 40px; height: 22px; }
.switch input { opacity: 0; width: 0; height: 0; }
.slider { position: absolute; cursor: pointer; inset: 0; background: var(--xuya-border, #444); border-radius: 22px; transition: .2s; }
.slider::before { content: ''; position: absolute; height: 16px; width: 16px; left: 3px; top: 3px; background: #fff; border-radius: 50%; transition: .2s; }
.switch input:checked + .slider { background: #34d399; }
.switch input:checked + .slider::before { transform: translateX(18px); }

.proxy-hint { margin-top: 14px; padding: 12px 14px; font-size: 12.5px; line-height: 1.6; color: var(--xuya-text-2, #888); background: var(--xuya-input-bg, rgba(127,127,127,.06)); border-radius: 8px; }

.proxy-quick-target { margin-top: 14px; padding-top: 14px; border-top: 1px solid var(--xuya-border, rgba(127,127,127,.12)); }
.proxy-quick-title { font-size: 12px; color: var(--xuya-text-3, #888); margin-bottom: 10px; }
.proxy-quick-list { display: flex; flex-direction: column; gap: 6px; }
.proxy-quick-item { display: flex; flex-direction: column; gap: 2px; text-align: left; padding: 8px 12px; font-size: 13px; color: var(--xuya-text, inherit); background: var(--xuya-input-bg, rgba(127,127,127,.06)); border: 1px solid var(--xuya-border, rgba(127,127,127,.12)); border-radius: 8px; transition: .1s; }
.proxy-quick-item:hover { border-color: var(--xuya-accent, #3b82f6); }
.proxy-quick-item.active { border-color: var(--xuya-accent, #3b82f6); background: rgba(59,130,246,.1); }
.proxy-quick-url { font-size: 10.5px; color: var(--xuya-text-3, #888); word-break: break-all; }

.mono { font-family: var(--xuya-font-mono, 'Consolas', monospace); }
.cli-mini-btn { display: inline-flex; align-items: center; gap: 4px; padding: 3px 8px; font-size: 11px; color: var(--xuya-text-2, #888); background: var(--xuya-bg-elevated, rgba(127,127,127,.1)); border: 1px solid var(--xuya-border, rgba(127,127,127,.2)); border-radius: 6px; transition: .1s; }
.cli-mini-btn:hover { color: var(--xuya-text, inherit); border-color: var(--xuya-accent, #3b82f6); }
</style>
