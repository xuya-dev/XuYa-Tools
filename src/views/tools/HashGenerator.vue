<template>
  <ToolShell title="UUID / 密码 / 哈希" :icon="Fingerprint" description="生成 UUID、随机密码,计算 MD5 / SHA-1 / SHA-256 等哈希。">
    <div class="tabs">
      <button
        v-for="t in tabs"
        :key="t.id"
        class="tab"
        :class="{ active: tab === t.id }"
        @click="tab = t.id"
      >
        {{ t.label }}
      </button>
    </div>

    <!-- UUID 生成器 -->
    <div v-if="tab === 'uuid'" class="panel">
      <div class="uuid-row">
        <BaseInput v-model="uuidResult" readonly placeholder="点击下方按钮生成" />
        <BaseButton variant="primary" @click="genUuid"><RefreshCw :size="14" /> 生成</BaseButton>
        <BaseButton variant="ghost" :disabled="!uuidResult" @click="copy(uuidResult)"><Copy :size="14" /></BaseButton>
      </div>
      <div class="uuid-options">
        <label class="check"><input v-model="uuidUppercase" type="checkbox" /> 大写</label>
        <label class="check"><input v-model="uuidNoDash" type="checkbox" /> 去掉连字符</label>
        <span class="hint">UUID v4 · 基于浏览器 crypto.randomUUID</span>
      </div>
      <div v-if="uuidHistory.length" class="history">
        <div class="history-head">历史</div>
        <div v-for="(u, i) in uuidHistory" :key="i" class="history-item" @click="copy(u)">
          <code>{{ u }}</code>
          <Copy :size="13" />
        </div>
      </div>
    </div>

    <!-- 密码生成器 -->
    <div v-else-if="tab === 'password'" class="panel">
      <div class="pwd-row">
        <BaseInput :model-value="pwdResult" readonly placeholder="点击生成密码" />
        <BaseButton variant="primary" @click="genPwd"><RefreshCw :size="14" /> 生成</BaseButton>
        <BaseButton variant="ghost" :disabled="!pwdResult" @click="copy(pwdResult)"><Copy :size="14" /></BaseButton>
      </div>
      <div class="pwd-strength">
        <span class="strength-label">强度:</span>
        <div class="strength-bar"><div class="strength-fill" :class="strengthLevel" :style="{ width: strengthPct + '%' }"></div></div>
        <span class="strength-text" :class="strengthLevel">{{ strengthLabel }}</span>
      </div>
      <div class="pwd-controls">
        <label class="control-row">
          <span>长度</span>
          <input v-model.number="pwdLen" type="range" min="4" max="64" />
          <span class="num-badge">{{ pwdLen }}</span>
        </label>
      </div>
      <div class="charset-grid">
        <label class="check"><input v-model="useUpper" type="checkbox" /> 大写 A-Z</label>
        <label class="check"><input v-model="useLower" type="checkbox" /> 小写 a-z</label>
        <label class="check"><input v-model="useDigits" type="checkbox" /> 数字 0-9</label>
        <label class="check"><input v-model="useSymbols" type="checkbox" /> 符号 !@#$…</label>
      </div>
    </div>

    <!-- 哈希计算 -->
    <div v-else class="panel">
      <textarea
        v-model="hashInput"
        class="hash-input"
        placeholder="输入要计算哈希的文本"
        spellcheck="false"
      ></textarea>
      <div class="hash-results">
        <div v-for="algo in algos" :key="algo.id" class="hash-item">
          <div class="hash-head">
            <span class="hash-algo">{{ algo.label }}</span>
            <button class="mini-btn" :disabled="!hashOutputs[algo.id]" @click="copy(hashOutputs[algo.id])">
              <Copy :size="12" /> 复制
            </button>
          </div>
          <code class="hash-value">{{ hashOutputs[algo.id] || '—' }}</code>
        </div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Fingerprint, Copy, RefreshCw } from '@lucide/vue';
import { md5 } from 'js-md5';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import BaseInput from '@/components/ui/BaseInput.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';

const toast = useToast();
const tabs = [
  { id: 'uuid' as const, label: 'UUID 生成' },
  { id: 'password' as const, label: '密码生成' },
  { id: 'hash' as const, label: '哈希计算' },
];
const tab = ref<'uuid' | 'password' | 'hash'>('uuid');

async function copy(text: string | undefined) {
  if (text) await copyToClipboard(text, '已复制');
}

// ============ UUID ============
const uuidUppercase = ref(false);
const uuidNoDash = ref(false);
const uuidResult = ref('');
const uuidHistory = ref<string[]>([]);

function genUuid() {
  let u: string = crypto.randomUUID();
  if (uuidNoDash.value) u = u.replace(/-/g, '');
  if (uuidUppercase.value) u = u.toUpperCase();
  uuidResult.value = u;
  uuidHistory.value.unshift(u);
  if (uuidHistory.value.length > 8) uuidHistory.value.pop();
}

// ============ 密码 ============
const pwdLen = ref(16);
const useUpper = ref(true);
const useLower = ref(true);
const useDigits = ref(true);
const useSymbols = ref(false);
const pwdResult = ref('');

const UPPER = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
const LOWER = 'abcdefghijklmnopqrstuvwxyz';
const DIGITS = '0123456789';
const SYMBOLS = '!@#$%^&*()-_=+[]{};:,.<>?';

function genPwd() {
  const pools: string[] = [];
  if (useUpper.value) pools.push(UPPER);
  if (useLower.value) pools.push(LOWER);
  if (useDigits.value) pools.push(DIGITS);
  if (useSymbols.value) pools.push(SYMBOLS);
  if (pools.length === 0) {
    toast.error('至少选择一种字符集');
    return;
  }
  const all = pools.join('');
  const len = Math.max(4, Math.min(64, pwdLen.value));
  const out: string[] = [];
  // 使用 crypto.getRandomValues 生成安全随机
  const buf = new Uint32Array(len);
  crypto.getRandomValues(buf);
  for (let i = 0; i < len; i++) {
    const idx = buf[i] ?? 0;
    out.push(all[idx % all.length] ?? '');
  }
  pwdResult.value = out.join('');
}

const strengthLevel = computed(() => {
  const len = pwdResult.value.length;
  if (!len) return 'none';
  const variety = [useUpper.value, useLower.value, useDigits.value, useSymbols.value].filter(Boolean).length;
  const score = len * variety;
  if (score >= 60) return 'strong';
  if (score >= 30) return 'medium';
  return 'weak';
});
const strengthPct = computed(() => ({ none: 0, weak: 33, medium: 66, strong: 100 }[strengthLevel.value]));
const strengthLabel = computed(() => ({ none: '—', weak: '弱', medium: '中', strong: '强' }[strengthLevel.value]));

// ============ 哈希 ============
const hashInput = ref('');
const algos = [
  { id: 'md5' as const, label: 'MD5' },
  { id: 'sha1' as const, label: 'SHA-1' },
  { id: 'sha256' as const, label: 'SHA-256' },
  { id: 'sha384' as const, label: 'SHA-384' },
  { id: 'sha512' as const, label: 'SHA-512' },
];
const hashOutputs = ref<Record<string, string>>({});

async function computeHash() {
  const text = hashInput.value;
  if (!text) {
    hashOutputs.value = {};
    return;
  }
  const enc = new TextEncoder().encode(text);
  const out: Record<string, string> = {};
  out.md5 = md5(text);
  for (const algo of ['SHA-1', 'SHA-256', 'SHA-384', 'SHA-512']) {
    const digest = await crypto.subtle.digest(algo, enc);
    out[algo.replace('-', '').toLowerCase()] = bufToHex(digest);
  }
  hashOutputs.value = out;
}
function bufToHex(buf: ArrayBuffer): string {
  return Array.from(new Uint8Array(buf))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

watch(hashInput, computeHash);

// 首次进入各 tab 时预生成一个
watch(tab, (t) => {
  if (t === 'uuid' && !uuidResult.value) genUuid();
  if (t === 'password' && !pwdResult.value) genPwd();
}, { immediate: true });
</script>

<style scoped>
.tabs {
  display: flex;
  gap: 6px;
  margin-bottom: 20px;
  border-bottom: 1px solid var(--xuya-border);
}
.tab {
  padding: 8px 16px;
  font-size: 13px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: color 0var(--xuya-duration-fast), border-color 0var(--xuya-duration-fast);
}
.tab:hover {
  color: var(--xuya-text);
}
.tab.active {
  color: var(--xuya-accent);
  border-bottom-color: var(--xuya-accent);
  font-weight: 600;
}

.panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 通用输入行 */
.uuid-row,
.pwd-row {
  display: flex;
  gap: 8px;
  align-items: stretch;
}
.uuid-row :deep(.base-input),
.pwd-row :deep(.base-input) {
  font-family: var(--xuya-font-mono);
}

/* 复选框 */
.check {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--xuya-text-secondary);
  cursor: pointer;
}
.check input {
  accent-color: var(--xuya-accent);
  width: 15px;
  height: 15px;
}

.uuid-options {
  display: flex;
  align-items: center;
  gap: 20px;
  flex-wrap: wrap;
}
.hint {
  font-size: 11.5px;
  color: var(--xuya-text-tertiary);
  margin-left: auto;
}

/* 历史 */
.history {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.history-head {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--xuya-text-tertiary);
}
.history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 12px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text-tertiary);
  cursor: pointer;
  transition: all 0var(--xuya-duration-fast);
}
.history-item:hover {
  color: var(--xuya-text);
  border-color: var(--xuya-accent);
}
.history-item code {
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
}

/* 密码强度 */
.pwd-strength {
  display: flex;
  align-items: center;
  gap: 10px;
}
.strength-label {
  font-size: 12.5px;
  color: var(--xuya-text-secondary);
}
.strength-bar {
  flex: 1;
  height: 6px;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  overflow: hidden;
}
.strength-fill {
  height: 100%;
  border-radius: var(--xuya-radius-sm);
  transition: width 0var(--xuya-duration), background 0var(--xuya-duration);
}
.strength-fill.weak {
  background: var(--xuya-danger);
}
.strength-fill.medium {
  background: var(--xuya-warn);
}
.strength-fill.strong {
  background: var(--xuya-success);
}
.strength-text {
  font-size: 12.5px;
  font-weight: 600;
  min-width: 28px;
}
.strength-text.weak {
  color: var(--xuya-danger);
}
.strength-text.medium {
  color: var(--xuya-warn);
}
.strength-text.strong {
  color: var(--xuya-success);
}

/* 滑块 */
.pwd-controls {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.control-row {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 13px;
  color: var(--xuya-text-secondary);
}
.control-row input[type='range'] {
  flex: 1;
  accent-color: var(--xuya-accent);
}
.num-badge {
  font-family: var(--xuya-font-mono);
  font-weight: 600;
  color: var(--xuya-accent);
  min-width: 28px;
  text-align: center;
}

.charset-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

/* 哈希 */
.hash-input {
  width: 100%;
  min-height: 100px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  resize: vertical;
  transition: border-color 0var(--xuya-duration-fast), box-shadow 0var(--xuya-duration-fast);
}
.hash-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.hash-results {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.hash-item {
  padding: 10px 12px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
}
.hash-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}
.hash-algo {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-accent);
}
.mini-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  font-size: 11px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-bg-elevated);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  transition: all 0var(--xuya-duration-fast);
}
.mini-btn:hover:not(:disabled) {
  color: var(--xuya-text);
  border-color: var(--xuya-accent);
}
.mini-btn:disabled {
  opacity: 0.4;
}
.hash-value {
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  word-break: break-all;
  color: var(--xuya-text);
}
</style>
