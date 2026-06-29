<template>
  <ToolShell title="Cron 表达式解析" :icon="ListTree" description="解析 Cron 五段表达式,人类可读描述,未来执行时间预览。">
    <div class="cron-input-wrap">
      <div class="field-grid">
        <div v-for="(f, i) in FIELDS" :key="i" class="field">
          <label>{{ f.label }}</label>
          <input v-model="parts[i]" type="text" class="field-input mono" :placeholder="f.placeholder" spellcheck="false" @input="onInput" />
        </div>
      </div>
      <div class="raw-row">
        <span class="raw-label">完整表达式</span>
        <code class="raw-expr mono">{{ parts.join(' ') }}</code>
        <button class="mini-btn" @click="copyExpr"><Copy :size="13" /></button>
      </div>
    </div>

    <div v-if="parseError" class="error-box">⚠️ {{ parseError }}</div>

    <div v-if="!parseError" class="result-section">
      <div class="section-label">人类可读描述</div>
      <div class="desc-box">{{ humanDesc }}</div>
    </div>

    <div v-if="!parseError" class="result-section">
      <div class="section-label">未来执行时间 (预览 {{ futureTimes.length }} 次)</div>
      <div class="times-grid">
        <div v-for="(t, i) in futureTimes" :key="i" class="time-item">
          <span class="time-rel">{{ t.rel }}</span>
          <code class="time-abs mono">{{ t.abs }}</code>
        </div>
        <div v-if="!futureTimes.length" class="ph">无法计算未来执行时间</div>
      </div>
    </div>

    <details class="presets">
      <summary>常用 Cron 速查</summary>
      <div class="preset-grid">
        <button v-for="p in PRESETS" :key="p.expr" class="preset-item" @click="applyPreset(p.expr)">
          <span class="p-name">{{ p.name }}</span>
          <code class="p-expr mono">{{ p.expr }}</code>
        </button>
      </div>
    </details>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { ListTree, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';

const FIELDS = [
  { label: '分钟', placeholder: '0-59' },
  { label: '小时', placeholder: '0-23' },
  { label: '日', placeholder: '1-31' },
  { label: '月', placeholder: '1-12' },
  { label: '星期', placeholder: '0-6' },
];
const PRESETS = [
  { name: '每分钟', expr: '* * * * *' },
  { name: '每小时整点', expr: '0 * * * *' },
  { name: '每天 0 点', expr: '0 0 * * *' },
  { name: '每天 9 点', expr: '0 9 * * *' },
  { name: '每周一 9 点', expr: '0 9 * * 1' },
  { name: '每月 1 号 0 点', expr: '0 0 1 * *' },
  { name: '工作日 9 点', expr: '0 9 * * 1-5' },
  { name: '每 15 分钟', expr: '*/15 * * * *' },
  { name: '每 6 小时', expr: '0 */6 * * *' },
  { name: '每天 0/12 点', expr: '0 0,12 * * *' },
];

const parts = ref(['0', '9', '*', '*', '1-5']);

function onInput() { /* 响应式自动更新 */ }

interface FieldRange { min: number; max: number; name: string; }
const RANGES: FieldRange[] = [
  { min: 0, max: 59, name: '分钟' },
  { min: 0, max: 23, name: '小时' },
  { min: 1, max: 31, name: '日' },
  { min: 1, max: 12, name: '月' },
  { min: 0, max: 6, name: '星期' },
];

/** 解析单个字段为候选值集合, 失败抛错 */
function parseField(field: string, range: FieldRange): number[] {
  if (field === '*') {
    const out: number[] = [];
    for (let i = range.min; i <= range.max; i++) out.push(i);
    return out;
  }
  const out: number[] = [];
  for (const seg of field.split(',')) {
    // step: a-b/c 或 */c
    const stepMatch = seg.match(/^(.*)\/(\d+)$/);
    const step = stepMatch ? parseInt(stepMatch[2]!, 10) : 1;
    let rangePart = stepMatch ? stepMatch[1]! : seg;
    let lo: number, hi: number;
    if (rangePart === '*') { lo = range.min; hi = range.max; }
    else if (rangePart.includes('-')) {
      const [a, b] = rangePart.split('-');
      lo = parseInt(a ?? '', 10); hi = parseInt(b ?? '', 10);
    } else {
      lo = parseInt(rangePart, 10);
      hi = stepMatch ? range.max : lo; // 单值带 step 表示 lo..max/step
    }
    if (isNaN(lo) || isNaN(hi) || lo < range.min || hi > range.max) {
      throw new Error(`${range.name}字段 "${field}" 超出范围 ${range.min}-${range.max}`);
    }
    for (let v = lo; v <= hi; v += step) out.push(v);
  }
  return [...new Set(out)].sort((a, b) => a - b);
}

const parsed = computed<{ ok: boolean; error: string; vals: number[][] }>(() => {
  try {
    if (parts.value.some((p) => !p.trim())) throw new Error('请填写完整的 5 段表达式');
    const vals = parts.value.map((p, i) => parseField(p.trim(), RANGES[i]!));
    return { ok: true, error: '', vals };
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e), vals: [] };
  }
});
const parseError = computed(() => parsed.value.error);

const humanDesc = computed(() => {
  if (!parsed.value.ok) return '';
  const [min, hour, dom, mon, dow] = parsed.value.vals;
  const ALL = (arr: number[] | undefined) => arr && arr.length >= 7 && arr[arr.length - 1]! - arr[0]! === arr.length - 1;
  const partsDesc: string[] = [];

  // 时间部分
  if (ALL(hour) && ALL(min)) {
    if (min!.length === 60 && hour!.length === 24) partsDesc.push('每分钟');
    else if (min!.length === 60) partsDesc.push(`每小时的每分钟`);
    else if (min!.length === 1 && hour!.length === 1) partsDesc.push(`每天 ${pad(hour![0]!)}:${pad(min![0]!)}`);
    else if (min!.length === 1) partsDesc.push(`每小时的 ${pad(min![0]!)} 分`);
    else partsDesc.push(`在 ${min!.slice(0, 4).map(pad).join(', ')}${min!.length > 4 ? '…' : ''} 分`);
  } else if (hour!.length === 1) {
    partsDesc.push(`每天 ${pad(hour![0]!)}:${ALL(min) ? '每分钟' : min!.slice(0, 3).map(pad).join(',')}`);
  } else {
    partsDesc.push(`${min!.slice(0, 3).map(pad).join(', ')}分 ${hour!.slice(0, 3).join(', ')}时`);
  }

  // 日期/星期
  const DOW_NAMES = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
  if (!ALL(dom) || !ALL(mon) || !ALL(dow)) {
    const cons: string[] = [];
    if (!ALL(mon)) cons.push(`${mon!.map(monthName).join('/')}`);
    if (!ALL(dow)) cons.push(dow!.map((d) => DOW_NAMES[d]!).join('/'));
    else if (!ALL(dom)) cons.push(`每月 ${dom!.join(',')} 号`);
    partsDesc.push(cons.length ? `仅在 ${cons.join(' 和 ')}` : '');
  }
  return partsDesc.filter(Boolean).join(', ');
});

const futureTimes = computed<{ rel: string; abs: string }[]>(() => {
  if (!parsed.value.ok) return [];
  const [min, hour, dom, mon, dow] = parsed.value.vals;
  const setMin = new Set(min!), setHour = new Set(hour!), setDom = new Set(dom!), setMon = new Set(mon!), setDow = new Set(dow!);
  const result: Date[] = [];
  const now = new Date();
  const d = new Date(now.getTime() + 60000); // 从下一分钟开始
  d.setSeconds(0, 0);
  let safety = 0;
  while (result.length < 8 && safety < 500000) {
    safety++;
    if (!setMon.has(d.getMonth() + 1)) { d.setMonth(d.getMonth() + 1, 1); d.setHours(0, 0, 0, 0); continue; }
    if (!setDom.has(d.getDate()) && !setDow.has(d.getDay())) { d.setDate(d.getDate() + 1); d.setHours(0, 0, 0, 0); continue; }
    if (!setHour.has(d.getHours())) { d.setHours(d.getHours() + 1, 0, 0, 0); continue; }
    if (!setMin.has(d.getMinutes())) { d.setMinutes(d.getMinutes() + 1, 0, 0); continue; }
    // 周和日只要满足其一(标准 cron 行为),这里采用 "日 或 星期"
    const domOk = setDom.has(d.getDate());
    const dowOk = setDow.has(d.getDay());
    if (!domOk && !dowOk) { d.setMinutes(d.getMinutes() + 1, 0, 0); continue; }
    result.push(new Date(d));
    d.setMinutes(d.getMinutes() + 1, 0, 0);
  }
  return result.map((t) => ({
    rel: relTime(t, now),
    abs: t.toLocaleString('zh-CN', { hour12: false }),
  }));
});

function pad(n: number) { return String(n).padStart(2, '0'); }
function monthName(m: number) { return `${m}月`; }
function relTime(t: Date, now: Date) {
  const diff = t.getTime() - now.getTime();
  const mins = Math.round(diff / 60000);
  if (mins < 60) return `${mins} 分钟后`;
  if (mins < 1440) return `${Math.round(mins / 60)} 小时后`;
  return `${Math.round(mins / 1440)} 天后`;
}

function applyPreset(expr: string) { parts.value = expr.split(' '); }
async function copyExpr() { await copyToClipboard(parts.value.join(' '), '已复制 Cron 表达式'); }
</script>

<style scoped>
.cron-input-wrap { margin-bottom: 16px; }
.field-grid { display: grid; grid-template-columns: repeat(5, 1fr); gap: 10px; margin-bottom: 12px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { font-size: 11px; color: var(--xuya-text-secondary); font-weight: 600; }
.field-input { padding: 8px 10px; font-size: 13px; border-radius: var(--xuya-radius-sm); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); text-align: center; transition: border-color var(--xuya-duration-fast); }
.field-input:focus { outline: none; border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }
.raw-row { display: flex; align-items: center; gap: 10px; padding: 10px 14px; background: var(--xuya-input-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius); }
.raw-label { font-size: 11px; color: var(--xuya-text-tertiary); }
.raw-expr { flex: 1; font-size: 14px; font-weight: 700; color: var(--xuya-accent); }
.mini-btn { width: 30px; height: 30px; display: flex; align-items: center; justify-content: center; border-radius: var(--xuya-radius-sm); color: var(--xuya-text-secondary); border: 1px solid var(--xuya-border); background: var(--xuya-bg-elevated); transition: var(--xuya-duration-fast); }
.mini-btn:hover { color: var(--xuya-text); border-color: var(--xuya-accent); }

.error-box { padding: 12px 16px; background: var(--xuya-danger-soft); color: var(--xuya-danger); border-radius: var(--xuya-radius); font-size: 13px; margin-bottom: 16px; }
.result-section { margin-bottom: 16px; }
.section-label { font-size: 11px; text-transform: uppercase; letter-spacing: .5px; color: var(--xuya-text-tertiary); margin-bottom: 8px; }
.desc-box { padding: 14px 16px; background: var(--xuya-accent-soft); color: var(--xuya-text); border-radius: var(--xuya-radius); font-size: 14px; font-weight: 500; }
.times-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 8px; }
.time-item { display: flex; flex-direction: column; gap: 2px; padding: 10px 12px; background: var(--xuya-input-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-sm); }
.time-rel { font-size: 11px; color: var(--xuya-accent); font-weight: 600; }
.time-abs { font-size: 12px; color: var(--xuya-text); }
.ph { color: var(--xuya-text-tertiary); font-size: 13px; padding: 12px; }

.presets { background: var(--xuya-card-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius); padding: 12px 16px; }
.presets summary { cursor: pointer; font-size: 13px; font-weight: 600; color: var(--xuya-text-secondary); }
.preset-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 8px; margin-top: 12px; }
.preset-item { display: flex; flex-direction: column; gap: 4px; text-align: left; padding: 8px 10px; background: var(--xuya-input-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-sm); transition: var(--xuya-duration-fast); }
.preset-item:hover { border-color: var(--xuya-accent); }
.p-name { font-size: 12px; color: var(--xuya-text); font-weight: 500; }
.p-expr { font-size: 10.5px; color: var(--xuya-accent); }
</style>
