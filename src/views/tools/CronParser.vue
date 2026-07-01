<template>
  <ToolShell
    title="Cron 表达式解析"
    :icon="ListTree"
    description="解析 Cron 五段表达式，人类可读描述，未来执行时间预览。"
  >
    <!-- 五段编辑器 -->
    <div class="field-grid">
      <div v-for="(f, i) in FIELDS" :key="i" class="field">
        <label class="field-label">{{ f.label }}</label>
        <input
          v-model="parts[i]"
          type="text"
          class="field-input"
          :class="{ invalid: fieldErrors[i] }"
          :placeholder="f.placeholder"
          spellcheck="false"
        />
        <span v-if="fieldErrors[i]" class="field-err">{{ fieldErrors[i] }}</span>
      </div>
    </div>

    <!-- 完整表达式 + 操作 -->
    <div class="expr-row">
      <code class="expr-display">{{ parts.join(' ') }}</code>
      <button class="mini-btn" title="复制表达式" @click="copyExpr">
        <Copy :size="13" />
      </button>
      <button class="mini-btn" title="复制描述" :disabled="!humanDesc" @click="copyDesc">
        <Copy :size="13" />
      </button>
      <button class="mini-btn danger" title="重置" @click="resetParts">重置</button>
    </div>

    <!-- 快捷模板 -->
    <div class="preset-pills">
      <button
        v-for="p in PRESETS"
        :key="p.expr"
        class="preset-pill"
        :class="{ active: parts.join(' ') === p.expr }"
        @click="applyPreset(p.expr)"
      >
        {{ p.name }}
      </button>
    </div>

    <!-- 错误 -->
    <div v-if="parseError" class="error-box">{{ parseError }}</div>

    <template v-else>
      <!-- 人类可读描述 -->
      <div class="desc-card">
        <span class="desc-label">执行规则</span>
        <span class="desc-text">{{ humanDesc }}</span>
      </div>

      <!-- 未来执行时间 -->
      <div class="times-section">
        <div class="times-head">
          <span class="section-label">未来执行时间</span>
          <div class="seg">
            <button :class="{ active: previewCount === 5 }" @click="previewCount = 5">5 次</button>
            <button :class="{ active: previewCount === 10 }" @click="previewCount = 10">
              10 次
            </button>
            <button :class="{ active: previewCount === 20 }" @click="previewCount = 20">
              20 次
            </button>
          </div>
        </div>
        <div class="times-grid">
          <div v-for="(t, i) in futureTimes" :key="i" class="time-item" @click="copyText(t.abs)">
            <span class="time-rel">{{ t.rel }}</span>
            <code class="time-abs">{{ t.abs }}</code>
            <span class="time-dow">{{ t.dow }}</span>
          </div>
          <div v-if="!futureTimes.length" class="ph">无法计算执行时间</div>
        </div>
      </div>
    </template>
  </ToolShell>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { ListTree, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const FIELDS = [
  { label: '分钟', placeholder: '0-59 *' },
  { label: '小时', placeholder: '0-23 *' },
  { label: '日', placeholder: '1-31 *' },
  { label: '月', placeholder: '1-12 *' },
  { label: '星期', placeholder: '0-6 *' },
];

const PRESETS = [
  { name: '每分钟', expr: '* * * * *' },
  { name: '每小时整点', expr: '0 * * * *' },
  { name: '每天 0 点', expr: '0 0 * * *' },
  { name: '每天 9 点', expr: '0 9 * * *' },
  { name: '工作日 9 点', expr: '0 9 * * 1-5' },
  { name: '每周一', expr: '0 0 * * 1' },
  { name: '每月 1 号', expr: '0 0 1 * *' },
  { name: '每 15 分钟', expr: '*/15 * * * *' },
  { name: '每 6 小时', expr: '0 */6 * * *' },
  { name: '每天 0/12 点', expr: '0 0,12 * * *' },
  { name: '每年 1 月 1 日', expr: '0 0 1 1 *' },
  { name: '每季度', expr: '0 0 1 */3 *' },
];

const parts = useToolState<string[]>('cron', 'parts', ['0', '9', '*', '*', '1-5']);
const previewCount = useToolState('cron', 'previewCount', 10);

interface FieldRange {
  min: number;
  max: number;
  name: string;
}
const RANGES: FieldRange[] = [
  { min: 0, max: 59, name: '分钟' },
  { min: 0, max: 23, name: '小时' },
  { min: 1, max: 31, name: '日' },
  { min: 1, max: 12, name: '月' },
  { min: 0, max: 6, name: '星期' },
];

function parseField(field: string, range: FieldRange): number[] {
  if (field === '*') {
    const out: number[] = [];
    for (let i = range.min; i <= range.max; i++) out.push(i);
    return out;
  }
  const out: number[] = [];
  for (const seg of field.split(',')) {
    const stepMatch = seg.match(/^(.*)\/(\d+)$/);
    const step = stepMatch ? parseInt(stepMatch[2]!, 10) : 1;
    const rangePart = stepMatch ? stepMatch[1]! : seg;
    let lo: number, hi: number;
    if (rangePart === '*') {
      lo = range.min;
      hi = range.max;
    } else if (rangePart.includes('-')) {
      const [a, b] = rangePart.split('-');
      lo = parseInt(a ?? '', 10);
      hi = parseInt(b ?? '', 10);
    } else {
      lo = parseInt(rangePart, 10);
      hi = stepMatch ? range.max : lo;
    }
    if (isNaN(lo) || isNaN(hi) || lo < range.min || hi > range.max || lo > hi) {
      throw new Error(`${range.name}: ${field} 超出 ${range.min}-${range.max}`);
    }
    for (let v = lo; v <= hi; v += step) out.push(v);
  }
  return [...new Set(out)].sort((a, b) => a - b);
}

const fieldErrors = computed<(string | null)[]>(() => {
  return RANGES.map((range, i) => {
    const val = parts.value[i]?.trim();
    if (!val) return null;
    try {
      parseField(val, range);
      return null;
    } catch (e) {
      return e instanceof Error ? e.message : String(e);
    }
  });
});

const parsed = computed<{ ok: boolean; error: string; vals: number[][] }>(() => {
  try {
    if (parts.value.some((p) => !p?.trim())) throw new Error('请填写完整的 5 段表达式');
    const vals = parts.value.map((p, i) => parseField((p ?? '').trim(), RANGES[i]!));
    return { ok: true, error: '', vals };
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e), vals: [] };
  }
});
const parseError = computed(() => parsed.value.error);

function isAll(arr: number[] | undefined, range: FieldRange): boolean {
  if (!arr) return false;
  return arr.length === range.max - range.min + 1;
}

const humanDesc = computed(() => {
  if (!parsed.value.ok) return '';
  const [min, hour, dom, mon, dow] = parsed.value.vals;
  const DOW_NAMES = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
  const MON_NAMES = [
    '',
    '1月',
    '2月',
    '3月',
    '4月',
    '5月',
    '6月',
    '7月',
    '8月',
    '9月',
    '10月',
    '11月',
    '12月',
  ];
  const partsDesc: string[] = [];

  const allMin = isAll(min, RANGES[0]!);
  const allHour = isAll(hour, RANGES[1]!);
  const allDom = isAll(dom, RANGES[2]!);
  const allMon = isAll(mon, RANGES[3]!);
  const allDow = isAll(dow, RANGES[4]!);

  if (allMin && allHour) {
    partsDesc.push('每分钟');
  } else if (min!.length === 1 && hour!.length === 1) {
    partsDesc.push(`每天 ${pad(hour![0]!)}:${pad(min![0]!)}`);
  } else if (hour!.length === 1 && allMin) {
    partsDesc.push(`每小时整点后的每分钟（${pad(hour![0]!)} 时）`);
  } else if (allHour) {
    partsDesc.push(`每小时的 ${min!.slice(0, 5).map(pad).join('、')} 分`);
  } else if (min!.length === 1) {
    partsDesc.push(
      `每天 ${hour!
        .slice(0, 4)
        .map((h) => `${pad(h)}:${pad(min![0]!)}`)
        .join('、')}`,
    );
  } else {
    partsDesc.push(
      `${hour!.slice(0, 3).map(pad).join('、')} 时 ${min!.slice(0, 3).map(pad).join('、')} 分`,
    );
  }

  const cons: string[] = [];
  if (!allDow) cons.push(dow!.map((d) => DOW_NAMES[d]!).join('、'));
  if (!allMon) cons.push(mon!.map((m) => MON_NAMES[m]!).join('、'));
  if (!allDom && allDow) cons.push(`每月 ${dom!.join('、')} 号`);
  if (cons.length) partsDesc[0] = cons.join('、') + '的 ' + partsDesc[0];

  return partsDesc.join('，');
});

const futureTimes = computed<{ rel: string; abs: string; dow: string }[]>(() => {
  if (!parsed.value.ok) return [];
  const [min, hour, dom, mon, dow] = parsed.value.vals;
  const setMin = new Set(min!),
    setHour = new Set(hour!),
    setDom = new Set(dom!),
    setMon = new Set(mon!),
    setDow = new Set(dow!);
  const DOW_NAMES = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
  const result: Date[] = [];
  const now = new Date();
  const d = new Date(now.getTime() + 60000);
  d.setSeconds(0, 0);
  let safety = 0;
  const limit = previewCount.value;
  while (result.length < limit && safety < 500000) {
    safety++;
    if (!setMon.has(d.getMonth() + 1)) {
      d.setMonth(d.getMonth() + 1, 1);
      d.setHours(0, 0, 0, 0);
      continue;
    }
    const domOk = setDom.has(d.getDate());
    const dowOk = setDow.has(d.getDay());
    if (!domOk && !dowOk) {
      d.setDate(d.getDate() + 1);
      d.setHours(0, 0, 0, 0);
      continue;
    }
    if (!setHour.has(d.getHours())) {
      d.setHours(d.getHours() + 1, 0, 0, 0);
      continue;
    }
    if (!setMin.has(d.getMinutes())) {
      d.setMinutes(d.getMinutes() + 1, 0, 0);
      continue;
    }
    result.push(new Date(d));
    d.setMinutes(d.getMinutes() + 1, 0, 0);
  }
  return result.map((t) => ({
    rel: relTime(t, now),
    abs: t.toLocaleString('zh-CN', { hour12: false }),
    dow: DOW_NAMES[t.getDay()]!,
  }));
});

function pad(n: number) {
  return String(n).padStart(2, '0');
}
function relTime(t: Date, now: Date) {
  const diff = t.getTime() - now.getTime();
  const mins = Math.round(diff / 60000);
  if (mins < 60) return `${mins} 分钟后`;
  if (mins < 1440) return `${Math.floor(mins / 60)} 小时后`;
  return `${Math.floor(mins / 1440)} 天后`;
}

function applyPreset(expr: string) {
  parts.value = expr.split(' ');
}
function resetParts() {
  parts.value = ['*', '*', '*', '*', '*'];
}
async function copyExpr() {
  await copyToClipboard(parts.value.join(' '), '已复制 Cron 表达式');
}
async function copyDesc() {
  if (humanDesc.value) await copyToClipboard(humanDesc.value, '已复制描述');
}
async function copyText(text: string) {
  await copyToClipboard(text, '已复制时间');
}
</script>

<style scoped>
.field-grid {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 10px;
  margin-bottom: 14px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.field-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.field-input {
  padding: 9px 10px;
  font-family: var(--xuya-font-mono);
  font-size: 14px;
  text-align: center;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.field-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.field-input.invalid {
  border-color: var(--xuya-danger);
}
.field-err {
  font-size: 10px;
  color: var(--xuya-danger);
}

.expr-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  margin-bottom: 14px;
}
.expr-display {
  flex: 1;
  font-family: var(--xuya-font-mono);
  font-size: 15px;
  font-weight: 700;
  color: var(--xuya-accent);
}
.mini-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text-secondary);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-bg-elevated);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
  flex-shrink: 0;
}
.mini-btn:hover:not(:disabled) {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}
.mini-btn:disabled {
  opacity: 0.4;
}
.mini-btn.danger:hover {
  color: var(--xuya-danger);
  border-color: var(--xuya-danger);
}

.preset-pills {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}
.preset-pill {
  padding: 4px 12px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: 99px;
  cursor: pointer;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  white-space: nowrap;
}
.preset-pill:hover {
  border-color: var(--xuya-accent);
  color: var(--xuya-text);
}
.preset-pill.active {
  background: var(--xuya-accent-soft);
  border-color: var(--xuya-accent);
  color: var(--xuya-accent);
  font-weight: 600;
}

.error-box {
  padding: 12px 16px;
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
  border-radius: var(--xuya-radius);
  font-size: 13px;
}

.desc-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  background: var(--xuya-accent-soft);
  border: 1px solid var(--xuya-accent-soft-strong);
  border-radius: var(--xuya-radius);
  margin-bottom: 16px;
}
.desc-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
  flex-shrink: 0;
}
.desc-text {
  font-size: 14px;
  font-weight: 500;
  color: var(--xuya-text);
}

.times-section {
  margin-bottom: 8px;
}
.times-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}
.section-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
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

.times-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
}
.time-item {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 10px 12px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.time-item:hover {
  border-color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
}
.time-rel {
  font-size: 11px;
  color: var(--xuya-accent);
  font-weight: 600;
}
.time-abs {
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  color: var(--xuya-text);
}
.time-dow {
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
}
.ph {
  color: var(--xuya-text-tertiary);
  font-size: 13px;
  padding: 12px;
}

@media (max-width: 560px) {
  .field-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
  .times-head {
    flex-wrap: wrap;
    gap: 8px;
  }
}
</style>
