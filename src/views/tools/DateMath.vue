<template>
  <ToolShell
    title="时间偏移计算"
    :icon="CalendarClock"
    description="计算「N 天后是几号」「两日期相差多久」,支持工作日计算与多种输出格式。"
  >
    <div class="grid">
      <!-- 日期偏移 -->
      <div class="card">
        <div class="card-head">
          <span>日期偏移</span>
          <div class="seg">
            <button :class="{ active: addMode === 'add' }" @click="addMode = 'add'">加</button>
            <button :class="{ active: addMode === 'sub' }" @click="addMode = 'sub'">减</button>
          </div>
        </div>
        <div class="add-body">
          <input v-model="addBase" type="datetime-local" step="1" class="dt-input" />
          <div class="amount-row">
            <input v-model.number="amount" type="number" class="num-input" min="0" />
            <div class="seg unit-seg">
              <button
                v-for="u in units"
                :key="u.id"
                :class="{ active: unit === u.id }"
                @click="unit = u.id"
              >
                {{ u.label }}
              </button>
            </div>
          </div>
          <label class="opt">
            <input v-model="workdayOnly" type="checkbox" />
            <span>仅按工作日计算(跳过周末,不含法定节假日)</span>
          </label>
        </div>
        <div class="result-block">
          <div class="result-label">结果</div>
          <code class="result-val">{{ addResult || '—' }}</code>
          <button v-if="addResult" class="mini-btn" @click="copyText(addResult)">
            <Copy :size="11" />
          </button>
        </div>
      </div>

      <!-- 日期差 -->
      <div class="card">
        <div class="card-head">
          <span>日期差</span>
          <button class="mini-btn" @click="swapDiff">⇅ 交换</button>
        </div>
        <div class="diff-body">
          <div class="diff-row">
            <span class="diff-tag">起始</span>
            <input v-model="diffStart" type="datetime-local" step="1" class="dt-input" />
          </div>
          <div class="diff-row">
            <span class="diff-tag">结束</span>
            <input v-model="diffEnd" type="datetime-local" step="1" class="dt-input" />
          </div>
          <label class="opt">
            <input v-model="countWorkday" type="checkbox" />
            <span>统计工作日天数(跳过周末)</span>
          </label>
        </div>
        <div class="result-list">
          <div v-for="r in diffResults" :key="r.label" class="result-row">
            <span class="result-label">{{ r.label }}</span>
            <code class="result-val">{{ r.value || '—' }}</code>
            <button v-if="r.value" class="mini-btn" @click="copyText(r.value)">
              <Copy :size="11" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { CalendarClock, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

// ===== 日期偏移 =====
type Unit = 'day' | 'week' | 'month' | 'year' | 'hour' | 'minute';
const units: { id: Unit; label: string }[] = [
  { id: 'minute', label: '分' },
  { id: 'hour', label: '时' },
  { id: 'day', label: '天' },
  { id: 'week', label: '周' },
  { id: 'month', label: '月' },
  { id: 'year', label: '年' },
];

const addBase = useToolState('datemath', 'addBase', toLocalInput(new Date()));
const amount = useToolState('datemath', 'amount', 7);
const unit = useToolState<Unit>('datemath', 'unit', 'day');
const addMode = useToolState<'add' | 'sub'>('datemath', 'addMode', 'add');
const workdayOnly = useToolState('datemath', 'workdayOnly', false);

const addResult = computed(() => {
  const base = parseLocal(addBase.value);
  if (!base) return '';
  const n = addMode.value === 'add' ? amount.value : -amount.value;
  if (workdayOnly.value && (unit.value === 'day' || unit.value === 'week')) {
    const days = unit.value === 'week' ? n * 7 : n;
    const result = addWorkdays(base, days);
    return formatLocal(result);
  }
  const d = new Date(base);
  try {
    switch (unit.value) {
      case 'minute':
        d.setMinutes(d.getMinutes() + n);
        break;
      case 'hour':
        d.setHours(d.getHours() + n);
        break;
      case 'day':
        d.setDate(d.getDate() + n);
        break;
      case 'week':
        d.setDate(d.getDate() + n * 7);
        break;
      case 'month':
        d.setMonth(d.getMonth() + n);
        break;
      case 'year':
        d.setFullYear(d.getFullYear() + n);
        break;
    }
  } catch {
    return '';
  }
  return formatLocal(d);
});

// ===== 日期差 =====
const diffStart = useToolState('datemath', 'diffStart', toLocalInput(daysAgo(7)));
const diffEnd = useToolState('datemath', 'diffEnd', toLocalInput(new Date()));
const countWorkday = useToolState('datemath', 'countWorkday', false);

const diffResults = computed(() => {
  const s = parseLocal(diffStart.value);
  const e = parseLocal(diffEnd.value);
  if (!s || !e) return [{ label: '相差', value: '' }];
  const ms = e.getTime() - s.getTime();
  const totalMin = Math.round(ms / 60000);
  const totalHour = Math.round(ms / 3600000);
  const totalDay = Math.round(ms / 86400000);
  const out: { label: string; value: string }[] = [
    { label: '总天数', value: String(totalDay) },
    { label: '总小时', value: String(totalHour) },
    { label: '总分钟', value: String(totalMin) },
    { label: '人类可读', value: humanize(ms) },
  ];
  if (countWorkday.value) {
    out.push({ label: '工作日', value: String(workdaysBetween(s, e)) + ' 天' });
  }
  return out;
});

function swapDiff() {
  const t = diffStart.value;
  diffStart.value = diffEnd.value;
  diffEnd.value = t;
}

// ===== 工作日计算 =====
function addWorkdays(start: Date, workdays: number): Date {
  // workdays 可正可负;0 返回当天
  const d = new Date(start);
  const step = workdays > 0 ? 1 : workdays < 0 ? -1 : 0;
  let remaining = Math.abs(workdays);
  while (remaining > 0) {
    d.setDate(d.getDate() + step);
    const dow = d.getDay();
    if (dow !== 0 && dow !== 6) remaining--;
  }
  return d;
}

function workdaysBetween(s: Date, e: Date): number {
  // 含起不算终,或按天数方向;统计 [min, max) 内的工作日
  let start = new Date(s);
  let end = new Date(e);
  let sign = 1;
  if (start > end) {
    [start, end] = [end, start];
    sign = -1;
  }
  // 规整到当天 00:00
  start.setHours(0, 0, 0, 0);
  end.setHours(0, 0, 0, 0);
  let count = 0;
  const cur = new Date(start);
  while (cur < end) {
    const dow = cur.getDay();
    if (dow !== 0 && dow !== 6) count++;
    cur.setDate(cur.getDate() + 1);
  }
  return sign * count;
}

// ===== 辅助 =====
function humanize(ms: number): string {
  const abs = Math.abs(ms);
  const days = Math.floor(abs / 86400000);
  const hours = Math.floor((abs % 86400000) / 3600000);
  const mins = Math.floor((abs % 3600000) / 60000);
  const parts: string[] = [];
  if (days) parts.push(`${days} 天`);
  if (hours) parts.push(`${hours} 小时`);
  if (mins) parts.push(`${mins} 分钟`);
  if (!parts.length) parts.push('0 分钟');
  return (ms < 0 ? '-' : '') + parts.join(' ');
}

/** 把 <input type="datetime-local"> 的本地字符串解析为 Date。 */
function parseLocal(s: string): Date | null {
  if (!s) return null;
  // 格式 YYYY-MM-DDTHH:mm[:ss]
  const m = /^(\d{4})-(\d{2})-(\d{2})(?:T(\d{2}):(\d{2})(?::(\d{2}))?)?$/.exec(s);
  if (!m) return null;
  const [, y, mo, d, h = '0', mi = '0', se = '0'] = m;
  return new Date(Number(y), Number(mo) - 1, Number(d), Number(h), Number(mi), Number(se));
}

/** Date → <input type="datetime-local"> 的本地字符串(秒精度)。 */
function toLocalInput(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(
    d.getMinutes(),
  )}:${pad(d.getSeconds())}`;
}

function formatLocal(d: Date): string {
  return d.toLocaleString('zh-CN', { hour12: false }) + ` · 周${'日一二三四五六'[d.getDay()]}`;
}

function daysAgo(n: number): Date {
  const d = new Date();
  d.setDate(d.getDate() - n);
  return d;
}

async function copyText(text: string) {
  if (text) await copyToClipboard(text, '已复制');
}
</script>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
}
.card {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.card-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
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
  padding: 4px 12px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-radius: var(--xuya-radius-sm);
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  cursor: pointer;
  white-space: nowrap;
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

.add-body,
.diff-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.dt-input {
  width: 100%;
  padding: 8px 10px;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  outline: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.dt-input:focus {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.amount-row {
  display: flex;
  gap: 10px;
  align-items: center;
}
.num-input {
  width: 100px;
  padding: 8px 10px;
  font-family: var(--xuya-font-mono);
  font-size: 14px;
  font-weight: 600;
  text-align: center;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text);
  outline: none;
}
.num-input:focus {
  border-color: var(--xuya-accent);
}
.unit-seg {
  flex: 1;
}
.unit-seg button {
  flex: 1;
  text-align: center;
}
.opt {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  cursor: pointer;
}
.opt input[type='checkbox'] {
  accent-color: var(--xuya-accent);
}

.diff-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.diff-tag {
  flex-shrink: 0;
  width: 40px;
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}

.result-block {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
}
.result-block .result-val {
  flex: 1;
  font-size: 14px;
  font-weight: 600;
  color: var(--xuya-accent);
}
.result-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.result-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
}
.result-label {
  flex-shrink: 0;
  width: 64px;
  font-size: 11px;
  color: var(--xuya-text-tertiary);
}
.result-val {
  flex: 1;
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  color: var(--xuya-text);
  word-break: break-all;
}
.mini-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 7px;
  font-size: 11px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  transition: all var(--xuya-duration-fast);
  cursor: pointer;
}
.mini-btn:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}
</style>
