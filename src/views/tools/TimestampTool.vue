<template>
  <ToolShell
    title="时间戳转换"
    :icon="Clock"
    description="Unix 时间戳与可读时间互转，支持秒 / 毫秒、UTC、ISO 8601、相对时间。"
  >
    <!-- 实时当前 -->
    <div class="now-bar">
      <div v-for="n in nowItems" :key="n.label" class="now-item" @click="copyText(n.value)">
        <span class="now-label">{{ n.label }}</span>
        <code class="now-val">{{ n.value }}</code>
      </div>
    </div>

    <div class="ts-grid">
      <!-- 时间戳 → 时间 -->
      <div class="ts-card">
        <div class="card-head">
          <span>时间戳 → 时间</span>
          <div class="seg">
            <button :class="{ active: tsUnit === 's' }" @click="tsUnit = 's'">秒</button>
            <button :class="{ active: tsUnit === 'ms' }" @click="tsUnit = 'ms'">毫秒</button>
          </div>
        </div>
        <input
          v-model="tsInput"
          class="ts-input"
          type="number"
          placeholder="如 1700000000"
          spellcheck="false"
        />
        <div class="result-list">
          <div v-for="r in tsResults" :key="r.label" class="result-row">
            <span class="result-label">{{ r.label }}</span>
            <code class="result-val" :class="{ err: r.error }">{{ r.value || '—' }}</code>
            <button v-if="r.value && !r.error" class="copy-mini" @click="copyText(r.value)">
              <Copy :size="11" />
            </button>
          </div>
        </div>
        <BaseButton
          variant="ghost"
          @click="tsInput = String(tsUnit === 's' ? Math.floor(Date.now() / 1000) : Date.now())"
        >
          用当前时间戳
        </BaseButton>
      </div>

      <!-- 时间 → 时间戳 -->
      <div class="ts-card">
        <div class="card-head">
          <span>时间 → 时间戳</span>
          <div class="seg">
            <button :class="{ active: dtUnit === 's' }" @click="dtUnit = 's'">秒</button>
            <button :class="{ active: dtUnit === 'ms' }" @click="dtUnit = 'ms'">毫秒</button>
          </div>
        </div>
        <input v-model="dtInput" type="datetime-local" step="1" class="ts-input dt-input" />
        <div class="result-list">
          <div v-for="r in dtResults" :key="r.label" class="result-row">
            <span class="result-label">{{ r.label }}</span>
            <code class="result-val">{{ r.value || '—' }}</code>
            <button v-if="r.value" class="copy-mini" @click="copyText(r.value)">
              <Copy :size="11" />
            </button>
          </div>
        </div>
        <BaseButton variant="ghost" @click="useNowDt">用当前时间</BaseButton>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Clock, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToolState } from '@/composables/useToolState';
import { copyToClipboard } from '@/composables/useClipboard';

// ============ 实时时钟 ============
const nowSec = ref('');
const nowMs = ref('');
const nowLocal = ref('');
const nowUtc = ref('');
const nowIso = ref('');
let timer: number | undefined;

function tick() {
  const d = new Date();
  nowSec.value = String(Math.floor(d.getTime() / 1000));
  nowMs.value = String(d.getTime());
  nowLocal.value = d.toLocaleString('zh-CN', { hour12: false });
  nowUtc.value = d.toUTCString();
  nowIso.value = d.toISOString();
}

const nowItems = computed(() => [
  { label: '秒时间戳', value: nowSec.value },
  { label: '毫秒时间戳', value: nowMs.value },
  { label: '本地时间', value: nowLocal.value },
  { label: 'UTC', value: nowUtc.value },
]);

onMounted(() => {
  tick();
  timer = window.setInterval(tick, 1000);
});
onUnmounted(() => clearInterval(timer));

// ============ 工具函数 ============
const WEEKDAYS = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六'];

function relativeTime(ms: number): string {
  const diff = ms - Date.now();
  const abs = Math.abs(diff);
  const min = 60000;
  const hr = 3600000;
  const day = 86400000;
  if (abs < 5000) return '刚刚';
  if (abs < min)
    return diff < 0 ? `${Math.floor(abs / 1000)} 秒前` : `${Math.floor(abs / 1000)} 秒后`;
  if (abs < hr)
    return diff < 0 ? `${Math.floor(abs / min)} 分钟前` : `${Math.floor(abs / min)} 分钟后`;
  if (abs < day)
    return diff < 0 ? `${Math.floor(abs / hr)} 小时前` : `${Math.floor(abs / hr)} 小时后`;
  const days = Math.floor(abs / day);
  return diff < 0 ? `${days} 天前` : `${days} 天后`;
}

function padLocal(d: Date): string {
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

// ============ 时间戳 → 时间 ============
const tsInput = useToolState('timestamp', 'tsInput', '');
const tsUnit = useToolState<'s' | 'ms'>('timestamp', 'tsUnit', 's');

const tsDate = computed<Date | null>(() => {
  const input = tsInput.value.trim();
  if (!input) return null;
  const n = Number(input);
  if (!Number.isFinite(n) || n < 0) return null;
  const ms = tsUnit.value === 's' ? n * 1000 : n;
  const d = new Date(ms);
  return isNaN(d.getTime()) ? null : d;
});

const tsResults = computed(() => {
  const d = tsDate.value;
  if (!d) {
    if (!tsInput.value.trim()) return [{ label: '结果', value: '', error: false }];
    return [{ label: '结果', value: '无效时间戳', error: true }];
  }
  return [
    { label: '本地时间', value: padLocal(d), error: false },
    { label: 'UTC', value: d.toUTCString(), error: false },
    { label: 'ISO 8601', value: d.toISOString(), error: false },
    { label: '相对时间', value: relativeTime(d.getTime()), error: false },
    { label: '星期', value: WEEKDAYS[d.getDay()] ?? '', error: false },
  ];
});

// ============ 时间 → 时间戳 ============
const dtInput = useToolState('timestamp', 'dtInput', '');
const dtUnit = useToolState<'s' | 'ms'>('timestamp', 'dtUnit', 's');

const dtDate = computed<Date | null>(() => {
  if (!dtInput.value) return null;
  const d = new Date(dtInput.value);
  return isNaN(d.getTime()) ? null : d;
});

const dtResults = computed(() => {
  const d = dtDate.value;
  if (!d) return [{ label: '结果', value: '' }];
  const t = d.getTime();
  return [
    { label: '秒时间戳', value: String(Math.floor(t / 1000)) },
    { label: '毫秒时间戳', value: String(t) },
    { label: 'ISO 8601', value: d.toISOString() },
    { label: '相对时间', value: relativeTime(t) },
    { label: '星期', value: WEEKDAYS[d.getDay()] ?? '' },
  ];
});

function useNowDt() {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  dtInput.value = `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

async function copyText(text: string) {
  await copyToClipboard(text, '已复制');
}
</script>

<style scoped>
.now-bar {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
  margin-bottom: 20px;
}
.now-item {
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 12px 14px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  cursor: pointer;
  transition: border-color var(--xuya-duration-fast);
}
.now-item:hover {
  border-color: var(--xuya-accent);
}
.now-label {
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
  font-weight: 600;
}
.now-val {
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-accent);
  word-break: break-all;
}

.ts-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 16px;
}
.ts-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 14px;
  font-weight: 600;
  color: var(--xuya-text);
}
.seg {
  display: inline-flex;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  padding: 2px;
  gap: 2px;
}
.seg button {
  padding: 4px 14px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-radius: var(--xuya-radius-sm);
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  cursor: pointer;
}
.seg button:hover {
  color: var(--xuya-text);
}
.seg button.active {
  background: var(--xuya-bg-elevated);
  color: var(--xuya-accent);
  font-weight: 600;
  box-shadow: var(--xuya-shadow-sm);
}

.ts-input {
  width: 100%;
  padding: 9px 12px;
  font-size: 14px;
  font-family: var(--xuya-font-mono);
  color: var(--xuya-text);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  background: var(--xuya-input-bg);
  outline: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.ts-input:focus {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.dt-input {
  font-family: inherit;
}

.result-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}
.result-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border-light);
  border-radius: var(--xuya-radius-sm);
}
.result-label {
  width: 72px;
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
  flex-shrink: 0;
}
.result-val {
  flex: 1;
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  color: var(--xuya-text);
  word-break: break-all;
}
.result-val.err {
  color: var(--xuya-danger);
}
.copy-mini {
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  color: var(--xuya-text-tertiary);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-bg-elevated);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
  flex-shrink: 0;
}
.copy-mini:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}

@media (max-width: 560px) {
  .now-bar {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .ts-grid {
    grid-template-columns: 1fr;
  }
}
</style>
