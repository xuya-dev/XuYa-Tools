<template>
  <ToolShell
    title="端口连通性测试"
    :icon="PlugZap"
    description="检测主机端口是否开放,支持端口段批量扫描,显示连通状态与握手耗时。"
  >
    <!-- 输入区 -->
    <div class="input-bar">
      <div class="field host-field">
        <label class="ctrl-label">主机 / IP</label>
        <input
          v-model="host"
          class="text-input"
          placeholder="如 example.com 或 127.0.0.1"
          spellcheck="false"
          @keydown.enter="runCheck"
        />
      </div>
      <div class="field ports-field">
        <label class="ctrl-label">端口(支持段)</label>
        <input
          v-model="ports"
          class="text-input"
          placeholder="如 80,443,8000-8010"
          spellcheck="false"
          @keydown.enter="runCheck"
        />
      </div>
      <div class="field timeout-field">
        <label class="ctrl-label">超时(ms)</label>
        <input v-model.number="timeout" type="number" min="100" class="text-input" />
      </div>
      <div class="run-btn-wrap">
        <BaseButton variant="primary" :disabled="loading || !canRun" @click="runCheck">
          <Search :size="14" />
          {{ loading ? '扫描中…' : '开始检测' }}
        </BaseButton>
      </div>
    </div>

    <!-- 预设 -->
    <div class="presets">
      <span class="preset-label">常用:</span>
      <button v-for="p in presets" :key="p.label" class="preset-chip" @click="applyPreset(p)">
        {{ p.label }}
      </button>
    </div>

    <!-- 错误 -->
    <div v-if="errorMsg" class="err-box">
      <AlertCircle :size="14" />
      {{ errorMsg }}
    </div>

    <!-- 结果 -->
    <div v-if="results" class="results">
      <div class="result-summary">
        <span class="summary-item">
          <strong>{{ results.host }}</strong>
        </span>
        <span class="summary-item ok">开放 {{ openCount }}</span>
        <span class="summary-item fail">关闭 {{ closedCount }}</span>
        <span class="summary-item total">
          共 {{ results.results.length }} 个 · 耗时 {{ results.totalMs }}ms
        </span>
      </div>

      <div class="result-table">
        <div class="thead">
          <span class="th th-port">端口</span>
          <span class="th th-status">状态</span>
          <span class="th th-latency">耗时</span>
          <span class="th th-err">说明</span>
        </div>
        <div class="tbody">
          <div
            v-for="r in results.results"
            :key="r.port"
            class="tr"
            :class="{ open: r.open, closed: !r.open }"
          >
            <span class="td td-port">{{ r.port }}</span>
            <span class="td td-status">
              <span class="status-dot" :class="r.open ? 'dot-open' : 'dot-closed'"></span>
              {{ r.open ? '开放' : '关闭' }}
            </span>
            <span class="td td-latency">{{ r.open ? r.latencyMs + ' ms' : '—' }}</span>
            <span class="td td-err" :title="r.error || ''">
              {{ r.error || (r.open ? '连接成功' : '—') }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <div v-else-if="!loading" class="placeholder">
      <PlugZap :size="40" />
      <p>输入主机和端口,检测连通性</p>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { PlugZap, Search, AlertCircle } from '@lucide/vue';
import { invoke } from '@tauri-apps/api/core';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToolState } from '@/composables/useToolState';

interface PortResult {
  port: number;
  open: boolean;
  latencyMs: number;
  error: string | null;
}
interface PortCheckOutput {
  host: string;
  results: PortResult[];
  totalMs: number;
}

const host = useToolState('portcheck', 'host', '');
const ports = useToolState('portcheck', 'ports', '');
const timeout = useToolState('portcheck', 'timeout', 3000);
const results = ref<PortCheckOutput | null>(null);
const loading = ref(false);
const errorMsg = ref('');

const canRun = computed(() => host.value.trim() && ports.value.trim());

const presets = [
  { label: 'Web 服务', host: '', ports: '80,443' },
  { label: '数据库', host: '', ports: '3306,5432,27017,6379' },
  { label: '常用端口', host: '', ports: '22,80,443,3306,8080,8443' },
  { label: '本机 1-1024', host: '127.0.0.1', ports: '1-1024' },
];

function applyPreset(p: { host: string; ports: string }) {
  if (p.host) host.value = p.host;
  ports.value = p.ports;
}

const openCount = computed(() => results.value?.results.filter((r) => r.open).length ?? 0);
const closedCount = computed(() => results.value?.results.filter((r) => !r.open).length ?? 0);

async function runCheck() {
  errorMsg.value = '';
  if (!canRun.value || loading.value) return;
  loading.value = true;
  results.value = null;
  try {
    results.value = await invoke<PortCheckOutput>('port_check', {
      input: {
        host: host.value.trim(),
        ports: ports.value.trim(),
        timeoutMs: timeout.value,
      },
    });
  } catch (e) {
    errorMsg.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.input-bar {
  display: flex;
  gap: 12px;
  align-items: flex-end;
  margin-bottom: 12px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.host-field {
  flex: 2;
}
.ports-field {
  flex: 2;
}
.timeout-field {
  flex: 0 0 110px;
}
.run-btn-wrap {
  padding-bottom: 1px;
}
.ctrl-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.text-input {
  width: 100%;
  padding: 8px 10px;
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text);
  outline: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.text-input:focus {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

.presets {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
.preset-label {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
}
.preset-chip {
  padding: 4px 10px;
  font-size: 11px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.preset-chip:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}

.err-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  margin-bottom: 16px;
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
  border-radius: var(--xuya-radius);
  font-size: 13px;
}

.results {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.result-summary {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 12px 16px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  flex-wrap: wrap;
}
.summary-item {
  font-size: 12.5px;
  color: var(--xuya-text-secondary);
}
.summary-item strong {
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
}
.summary-item.ok {
  color: var(--xuya-success);
  font-weight: 600;
}
.summary-item.fail {
  color: var(--xuya-danger);
  font-weight: 600;
}
.summary-item.total {
  margin-left: auto;
  font-size: 11px;
  color: var(--xuya-text-tertiary);
}

.result-table {
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  overflow: hidden;
}
.thead {
  display: grid;
  grid-template-columns: 80px 100px 100px 1fr;
  background: var(--xuya-card-bg);
  border-bottom: 1px solid var(--xuya-border);
}
.th {
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.tbody {
  max-height: 400px;
  overflow-y: auto;
}
.tr {
  display: grid;
  grid-template-columns: 80px 100px 100px 1fr;
  border-bottom: 1px solid var(--xuya-border);
}
.tr:last-child {
  border-bottom: none;
}
.td {
  padding: 8px 12px;
  font-size: 12.5px;
  font-family: var(--xuya-font-mono);
  color: var(--xuya-text);
}
.td-port {
  font-weight: 600;
}
.td-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: inherit;
}
.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot-open {
  background: var(--xuya-success);
  box-shadow: 0 0 6px var(--xuya-success);
}
.dot-closed {
  background: var(--xuya-text-tertiary);
}
.td-err {
  color: var(--xuya-text-tertiary);
  font-size: 11.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tr.open {
  background: rgba(16, 185, 129, 0.04);
}

.placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 0;
  color: var(--xuya-text-tertiary);
}
.placeholder p {
  font-size: 14px;
}
</style>
