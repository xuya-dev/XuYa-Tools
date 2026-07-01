<template>
  <ToolShell
    title="ID 生成器"
    :icon="Fingerprint"
    description="生成 UUID、雪花 ID、随机密码，支持批量与自定义选项。"
  >
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

    <!-- UUID / 雪花 ID -->
    <div v-if="tab === 'uuid'" class="panel">
      <div class="uuid-type-row">
        <div class="seg">
          <button :class="{ active: uuidType === 'v4' }" @click="uuidType = 'v4'">UUID v4</button>
          <button :class="{ active: uuidType === 'snowflake' }" @click="uuidType = 'snowflake'">
            雪花 ID
          </button>
        </div>
        <template v-if="uuidType === 'v4'">
          <label class="check">
            <input v-model="uuidUppercase" type="checkbox" />
            大写
          </label>
          <label class="check">
            <input v-model="uuidNoDash" type="checkbox" />
            去连字符
          </label>
        </template>
        <template v-else>
          <label class="check">
            机器 ID
            <input v-model.number="sfMachineId" type="number" min="0" max="1023" class="sf-input" />
          </label>
        </template>
        <span style="flex: 1"></span>
        <label class="check">
          批量
          <input v-model.number="batchCount" type="number" min="1" max="50" class="sf-input" />
        </label>
      </div>
      <div class="result-row">
        <BaseInput :model-value="uuidResult" readonly placeholder="点击生成" />
        <BaseButton variant="primary" @click="genId">
          <RefreshCw :size="14" />
          生成
        </BaseButton>
        <BaseButton variant="ghost" :disabled="!uuidResult" @click="copy(uuidResult)">
          <Copy :size="14" />
        </BaseButton>
      </div>
      <div v-if="uuidType === 'snowflake' && uuidResult" class="sf-decode">
        <div class="sf-decode-item">
          <span class="sf-label">时间戳</span>
          <span class="sf-value">{{ sfDecoded.timeStr }}</span>
        </div>
        <div class="sf-decode-item">
          <span class="sf-label">机器 ID</span>
          <span class="sf-value">{{ sfDecoded.machineId }}</span>
        </div>
        <div class="sf-decode-item">
          <span class="sf-label">序列号</span>
          <span class="sf-value">{{ sfDecoded.sequence }}</span>
        </div>
      </div>
      <div v-if="uuidHistory.length" class="history">
        <div class="history-head">
          <span>历史 ({{ uuidHistory.length }})</span>
          <button class="clear-hist" @click="uuidHistory = []">清空</button>
        </div>
        <div v-for="(u, i) in uuidHistory" :key="i" class="history-item" @click="copy(u)">
          <code>{{ u }}</code>
          <Copy :size="13" />
        </div>
      </div>
    </div>

    <!-- 密码 -->
    <div v-else class="panel">
      <div class="result-row">
        <BaseInput :model-value="pwdResult" readonly placeholder="点击生成密码" />
        <BaseButton variant="primary" @click="genPwd">
          <RefreshCw :size="14" />
          生成
        </BaseButton>
        <BaseButton variant="ghost" :disabled="!pwdResult" @click="copy(pwdResult)">
          <Copy :size="14" />
        </BaseButton>
      </div>
      <div class="pwd-strength">
        <span class="strength-label">强度</span>
        <div class="strength-bar">
          <div
            class="strength-fill"
            :class="strengthLevel"
            :style="{ width: strengthPct + '%' }"
          ></div>
        </div>
        <span class="strength-text" :class="strengthLevel">{{ strengthLabel }}</span>
      </div>
      <div class="control-row">
        <span>长度</span>
        <input v-model.number="pwdLen" type="range" min="4" max="64" />
        <span class="num-badge">{{ pwdLen }}</span>
      </div>
      <div class="charset-grid">
        <label class="check">
          <input v-model="useUpper" type="checkbox" />
          大写 A-Z
        </label>
        <label class="check">
          <input v-model="useLower" type="checkbox" />
          小写 a-z
        </label>
        <label class="check">
          <input v-model="useDigits" type="checkbox" />
          数字 0-9
        </label>
        <label class="check">
          <input v-model="useSymbols" type="checkbox" />
          符号 !@#$
        </label>
        <label class="check">
          <input v-model="excludeAmbiguous" type="checkbox" />
          排除易混淆 (0O1lI)
        </label>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Fingerprint, Copy, RefreshCw } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import BaseInput from '@/components/ui/BaseInput.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const toast = useToast();
const tabs = [
  { id: 'uuid' as const, label: 'UUID / 雪花 ID' },
  { id: 'password' as const, label: '密码生成' },
];
const tab = ref<'uuid' | 'password'>('uuid');

async function copy(text: string | undefined) {
  if (text) await copyToClipboard(text, '已复制');
}

const uuidType = useToolState<'v4' | 'snowflake'>('idgen', 'uuidType', 'v4');
const uuidUppercase = useToolState('idgen', 'uuidUpper', false);
const uuidNoDash = useToolState('idgen', 'uuidNoDash', false);
const sfMachineId = useToolState('idgen', 'sfMachineId', 1);
const batchCount = useToolState('idgen', 'batchCount', 1);
const uuidResult = ref('');
const uuidHistory = useToolState<string[]>('idgen', 'uuidHistory', []);

const SNOWFLAKE_EPOCH = BigInt(Date.UTC(2024, 0, 1));
let sfSequence = 0n;
let sfLastTs = -1n;

function genSnowflakeId(): string {
  const machineId = BigInt(Math.max(0, Math.min(1023, sfMachineId.value)));
  let ts = BigInt(Date.now()) - SNOWFLAKE_EPOCH;
  if (ts === sfLastTs) {
    sfSequence = (sfSequence + 1n) & 0xfffn;
    if (sfSequence === 0n) {
      while (ts <= sfLastTs) ts = BigInt(Date.now()) - SNOWFLAKE_EPOCH;
    }
  } else {
    sfSequence = 0n;
  }
  sfLastTs = ts;
  return ((ts << 22n) | (machineId << 12n) | sfSequence).toString();
}

function formatUuid(raw: string): string {
  let u = raw;
  if (uuidNoDash.value) u = u.replace(/-/g, '');
  if (uuidUppercase.value) u = u.toUpperCase();
  return u;
}

function genId() {
  const count = Math.max(1, Math.min(50, batchCount.value));
  const results: string[] = [];
  for (let i = 0; i < count; i++) {
    results.push(uuidType.value === 'v4' ? formatUuid(crypto.randomUUID()) : genSnowflakeId());
  }
  uuidResult.value = results.join('\n');
  uuidHistory.value = [...results, ...uuidHistory.value].slice(0, 20);
}

const sfDecoded = computed(() => {
  if (uuidType.value !== 'snowflake' || !uuidResult.value)
    return { timeStr: '—', machineId: '—', sequence: '—' };
  const firstId = uuidResult.value.split('\n')[0] ?? '';
  try {
    const n = BigInt(firstId);
    return {
      timeStr: new Date(Number(n >> 22n) + Number(SNOWFLAKE_EPOCH)).toLocaleString('zh-CN', {
        hour12: false,
      }),
      machineId: String(Number((n >> 12n) & 0x3ffn)),
      sequence: String(Number(n & 0xfffn)),
    };
  } catch {
    return { timeStr: '—', machineId: '—', sequence: '—' };
  }
});

const pwdLen = useToolState('idgen', 'pwdLen', 16);
const useUpper = useToolState('idgen', 'useUpper', true);
const useLower = useToolState('idgen', 'useLower', true);
const useDigits = useToolState('idgen', 'useDigits', true);
const useSymbols = useToolState('idgen', 'useSymbols', false);
const excludeAmbiguous = useToolState('idgen', 'excludeAmbiguous', false);
const pwdResult = ref('');

const AMBIGUOUS = /[0O1lI|`]/g;

function genPwd() {
  const pools: string[] = [];
  if (useUpper.value) pools.push('ABCDEFGHIJKLMNOPQRSTUVWXYZ');
  if (useLower.value) pools.push('abcdefghijklmnopqrstuvwxyz');
  if (useDigits.value) pools.push('0123456789');
  if (useSymbols.value) pools.push('!@#$%^&*()-_=+[]{};:,.<>?');
  if (!pools.length) return toast.error('至少选择一种字符集');
  let all = pools.join('');
  if (excludeAmbiguous.value) all = all.replace(AMBIGUOUS, '');
  if (!all) return toast.error('排除后无可用字符');
  const len = Math.max(4, Math.min(64, pwdLen.value));
  const buf = new Uint32Array(len);
  crypto.getRandomValues(buf);
  pwdResult.value = Array.from(buf, (b) => all[b % all.length] ?? '').join('');
}

const strengthLevel = computed(() => {
  const len = pwdResult.value.length;
  if (!len) return 'none';
  const variety = [useUpper.value, useLower.value, useDigits.value, useSymbols.value].filter(
    Boolean,
  ).length;
  const score = len * variety;
  if (score >= 60) return 'strong';
  if (score >= 30) return 'medium';
  return 'weak';
});
const strengthPct = computed(
  () => ({ none: 0, weak: 33, medium: 66, strong: 100 })[strengthLevel.value],
);
const strengthLabel = computed(
  () => ({ none: '—', weak: '弱', medium: '中', strong: '强' })[strengthLevel.value],
);

watch(
  tab,
  (t) => {
    if (t === 'uuid' && !uuidResult.value) genId();
    if (t === 'password' && !pwdResult.value) genPwd();
  },
  { immediate: true },
);
</script>

<style scoped>
.tabs {
  display: flex;
  gap: 2px;
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
  transition:
    color var(--xuya-duration-fast),
    border-color var(--xuya-duration-fast);
  cursor: pointer;
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
.uuid-type-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
.seg {
  display: inline-flex;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  padding: 2px;
  gap: 2px;
}
.seg button {
  padding: 5px 14px;
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
.result-row {
  display: flex;
  gap: 8px;
  align-items: stretch;
}
.result-row :deep(.base-input) {
  font-family: var(--xuya-font-mono);
}
.check {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  cursor: pointer;
}
.check input[type='checkbox'] {
  accent-color: var(--xuya-accent);
  width: 14px;
  height: 14px;
}
.sf-input {
  width: 56px;
  padding: 4px 6px;
  font-size: 12px;
  font-family: var(--xuya-font-mono);
  color: var(--xuya-text);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  outline: none;
}
.sf-input:focus {
  border-color: var(--xuya-accent);
}
.sf-decode {
  display: flex;
  gap: 20px;
  padding: 12px 16px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  flex-wrap: wrap;
}
.sf-decode-item {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.sf-label {
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
  font-weight: 600;
}
.sf-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
}
.history {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.history-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
}
.clear-hist {
  font-size: 10px;
  text-transform: none;
  letter-spacing: 0;
  color: var(--xuya-text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
}
.clear-hist:hover {
  color: var(--xuya-danger);
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
  transition: all var(--xuya-duration-fast);
}
.history-item:hover {
  color: var(--xuya-text);
  border-color: var(--xuya-accent);
}
.history-item code {
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  word-break: break-all;
}
.pwd-strength {
  display: flex;
  align-items: center;
  gap: 10px;
}
.strength-label {
  font-size: 12px;
  color: var(--xuya-text-secondary);
  font-weight: 600;
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
  transition:
    width var(--xuya-duration),
    background var(--xuya-duration);
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
  font-size: 12px;
  font-weight: 600;
  min-width: 24px;
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
.control-row {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  font-weight: 600;
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
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 10px;
}
</style>
