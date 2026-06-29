<template>
  <ToolShell title="时间戳转换" :icon="Clock" description="Unix 时间戳与可读时间互转,支持秒 / 毫秒,实时当前时间。">
    <!-- 实时当前时间戳 -->
    <div class="now-bar">
      <div class="now-item">
        <span class="now-label">当前时间戳 (秒)</span>
        <code class="now-val">{{ nowSec }}</code>
      </div>
      <div class="now-item">
        <span class="now-label">当前时间戳 (毫秒)</span>
        <code class="now-val">{{ nowMs }}</code>
      </div>
      <div class="now-item">
        <span class="now-label">本地时间</span>
        <code class="now-val">{{ nowLocal }}</code>
      </div>
    </div>

    <div class="convert-grid">
      <!-- 时间戳 → 可读 -->
      <BaseCard class="convert-card">
        <h3 class="card-title">时间戳 → 时间</h3>
        <div class="unit-toggle">
          <button :class="{ active: tsUnit === 's' }" @click="tsUnit = 's'">秒</button>
          <button :class="{ active: tsUnit === 'ms' }" @click="tsUnit = 'ms'">毫秒</button>
        </div>
        <BaseInput v-model="tsInput" type="number" placeholder="如 1700000000" />
        <div class="result-box" :class="{ err: tsError }" @click="copyResult">
          {{ tsError || tsResult || '—' }}
        </div>
        <div class="row-actions">
          <BaseButton variant="ghost" @click="tsInput = String(Math.floor(Date.now() / (tsUnit === 's' ? 1000 : 1)))">用当前时间戳</BaseButton>
        </div>
      </BaseCard>

      <!-- 可读 → 时间戳 -->
      <BaseCard class="convert-card">
        <h3 class="card-title">时间 → 时间戳</h3>
        <div class="unit-toggle">
          <button :class="{ active: dtUnit === 's' }" @click="dtUnit = 's'">秒</button>
          <button :class="{ active: dtUnit === 'ms' }" @click="dtUnit = 'ms'">毫秒</button>
        </div>
        <input v-model="dtInput" type="datetime-local" step="1" class="dt-input" />
        <div class="result-box" @click="copyResult">
          {{ dtResult || '—' }}
        </div>
        <div class="row-actions">
          <BaseButton variant="ghost" @click="useNowDt">用当前时间</BaseButton>
        </div>
      </BaseCard>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Clock } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseCard from '@/components/ui/BaseCard.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToolState } from '@/composables/useToolState';
import BaseInput from '@/components/ui/BaseInput.vue';
import { copyToClipboard } from '@/composables/useClipboard';

// 实时时钟
const nowSec = ref('');
const nowMs = ref('');
const nowLocal = ref('');
let timer: number | undefined;

function tick() {
  const d = new Date();
  nowSec.value = String(Math.floor(d.getTime() / 1000));
  nowMs.value = String(d.getTime());
  nowLocal.value = d.toLocaleString('zh-CN', { hour12: false });
}
onMounted(() => {
  tick();
  timer = window.setInterval(tick, 1000);
});
onUnmounted(() => clearInterval(timer));

// 时间戳 → 时间 (pure compute,error/value 由同一计算派生,避免副作用)
const tsInput = useToolState('timestamp', 'tsInput', '');
const tsUnit = useToolState<'s' | 'ms'>('timestamp', 'tsUnit', 's');
const tsComputed = computed<{ value: string; error: string }>(() => {
  const input = tsInput.value.trim();
  if (!input) return { value: '', error: '' };
  const n = Number(input);
  if (!Number.isFinite(n) || n < 0) return { value: '', error: '请输入有效的非负数字' };
  const ms = tsUnit.value === 's' ? n * 1000 : n;
  const d = new Date(ms);
  if (isNaN(d.getTime())) return { value: '', error: '时间戳超出范围' };
  return { value: d.toLocaleString('zh-CN', { hour12: false }), error: '' };
});
const tsResult = computed(() => tsComputed.value.value);
const tsError = computed(() => tsComputed.value.error);

// 时间 → 时间戳
const dtInput = ref('');
const dtUnit = ref<'s' | 'ms'>('s');
const dtResult = computed(() => {
  if (!dtInput.value) return '';
  const d = new Date(dtInput.value);
  if (isNaN(d.getTime())) return '无效时间';
  const t = d.getTime();
  return String(tsUnit.value === 's' ? Math.floor(t / 1000) : t);
});

function useNowDt() {
  const d = new Date();
  // datetime-local 需要本地时间格式 YYYY-MM-DDTHH:mm:ss
  const pad = (n: number) => String(n).padStart(2, '0');
  dtInput.value = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

// 让结果框点击复制
function copyResult(e: MouseEvent) {
  const el = e.currentTarget as HTMLElement;
  const text = el.textContent?.replace(/—/g, '').trim();
  if (text) copyToClipboard(text, '已复制');
}
</script>

<style scoped>
.now-bar {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
  margin-bottom: 24px;
}
.now-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px 16px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.now-label {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.now-val {
  font-family: var(--xuya-font-mono);
  font-size: 14px;
  font-weight: 600;
  color: var(--xuya-accent);
}

.convert-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}
.convert-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--xuya-text);
  margin: 0;
}
.unit-toggle {
  display: inline-flex;
  align-self: flex-start;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  padding: 2px;
  gap: 2px;
}
.unit-toggle button {
  padding: 4px 14px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-radius: 4px;
  transition: all 0.12s;
}
.unit-toggle button.active {
  background: var(--xuya-bg-elevated);
  color: var(--xuya-accent);
  font-weight: 600;
  box-shadow: var(--xuya-shadow);
}
.dt-input {
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  font-family: inherit;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  transition: border-color 0.12s, box-shadow 0.12s;
}
.dt-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.result-box {
  padding: 12px 14px;
  font-family: var(--xuya-font-mono);
  font-size: 14px;
  font-weight: 600;
  color: var(--xuya-text);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  min-height: 46px;
  word-break: break-all;
  cursor: pointer;
}
.result-box.err {
  color: var(--xuya-danger);
  font-weight: 500;
}
.row-actions {
  display: flex;
  gap: 8px;
}
</style>
