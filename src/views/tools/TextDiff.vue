<template>
  <ToolShell
    title="文本差异对比"
    :icon="GitCompare"
    description="双栏输入，行级 + 字符级 diff 高亮，统计增删改，支持忽略空格与大小写。"
  >
    <div class="diff-toolbar">
      <div class="seg">
        <button :class="{ active: mode === 'split' }" @click="mode = 'split'">分栏对比</button>
        <button :class="{ active: mode === 'unified' }" @click="mode = 'unified'">合并视图</button>
      </div>

      <span class="sep"></span>

      <label class="toggle" title="隐藏相同行，仅展示差异">
        <input v-model="onlyDiff" type="checkbox" />
        仅看差异
      </label>
      <label class="toggle" title="比较时移除所有空白字符">
        <input v-model="ignoreWs" type="checkbox" />
        忽略空格
      </label>
      <label class="toggle" title="比较时不区分大小写">
        <input v-model="ignoreCase" type="checkbox" />
        忽略大小写
      </label>

      <span class="sep"></span>

      <div v-if="hasInput" class="diff-stats">
        <span class="ds add">+{{ stat.add }}</span>
        <span class="ds del">−{{ stat.del }}</span>
        <span class="ds eq">={{ stat.eq }}</span>
        <span class="ds sim">{{ similarity }}% 相似</span>
      </div>

      <span style="flex: 1"></span>

      <BaseButton variant="ghost" :disabled="!hasInput" @click="swap">交换</BaseButton>
      <BaseButton variant="ghost" :disabled="!lines.length" @click="copyResult">
        复制结果
      </BaseButton>
      <BaseButton variant="ghost" @click="clear">清空</BaseButton>
    </div>

    <div class="input-grid">
      <div class="input-col">
        <div class="col-head">
          <span>原始文本</span>
          <span class="stat">{{ leftLines.length }} 行 · {{ leftText.length }} 字符</span>
        </div>
        <textarea
          v-model="leftText"
          class="editor"
          placeholder="粘贴原始文本…"
          spellcheck="false"
        ></textarea>
      </div>
      <div class="input-col">
        <div class="col-head">
          <span>修改后文本</span>
          <span class="stat">{{ rightLines.length }} 行 · {{ rightText.length }} 字符</span>
        </div>
        <textarea
          v-model="rightText"
          class="editor"
          placeholder="粘贴修改后文本…"
          spellcheck="false"
        ></textarea>
      </div>
    </div>

    <div class="diff-result">
      <div class="result-head">
        <span class="section-label">差异结果</span>
        <span v-if="hasInput && !hasDiff" class="same-badge">✓ 完全相同</span>
      </div>

      <div v-if="!hasInput" class="ph">在上方输入两段文本后自动开始对比</div>

      <div v-else-if="!hasDiff" class="ph ph-ok">
        {{ anyNorm ? '忽略空格 / 大小写后完全相同' : '两段文本完全相同' }}
      </div>

      <div v-else-if="mode === 'split'" class="split-view">
        <div class="split-col">
          <div class="split-head">原始</div>
          <div v-for="(ln, i) in displayLines" :key="'l' + i" class="diff-line" :class="ln.type">
            <span class="ln-num">{{ ln.lNum || '' }}</span>
            <span class="ln-sign">{{ ln.type === 'del' ? '−' : '' }}</span>
            <span class="ln-content">
              <template v-if="ln.lSegs">
                <span
                  v-for="(seg, si) in ln.lSegs"
                  :key="si"
                  class="seg"
                  :class="{ 'seg-del': seg.hl }"
                >
                  {{ seg.val }}
                </span>
              </template>
              <template v-else>{{ ln.lContent }}</template>
            </span>
          </div>
        </div>
        <div class="split-col">
          <div class="split-head">修改后</div>
          <div v-for="(ln, i) in displayLines" :key="'r' + i" class="diff-line" :class="ln.type">
            <span class="ln-num">{{ ln.rNum || '' }}</span>
            <span class="ln-sign">{{ ln.type === 'add' ? '+' : '' }}</span>
            <span class="ln-content">
              <template v-if="ln.rSegs">
                <span
                  v-for="(seg, si) in ln.rSegs"
                  :key="si"
                  class="seg"
                  :class="{ 'seg-add': seg.hl }"
                >
                  {{ seg.val }}
                </span>
              </template>
              <template v-else>{{ ln.rContent }}</template>
            </span>
          </div>
        </div>
      </div>

      <div v-else class="unified-view">
        <div v-for="(ln, i) in displayLines" :key="i" class="diff-line" :class="ln.type">
          <span class="ln-num">{{ ln.type === 'add' ? ln.rNum : ln.lNum }}</span>
          <span class="ln-sign">{{ ln.type === 'add' ? '+' : ln.type === 'del' ? '−' : ' ' }}</span>
          <span class="ln-content">{{ ln.type === 'del' ? ln.lContent : ln.rContent }}</span>
        </div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { GitCompare } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const leftText = useToolState('diff', 'left', '');
const rightText = useToolState('diff', 'right', '');
const mode = useToolState<'split' | 'unified'>('diff', 'mode', 'split');
const onlyDiff = useToolState('diff', 'onlyDiff', true);
const ignoreWs = useToolState('diff', 'ignoreWs', false);
const ignoreCase = useToolState('diff', 'ignoreCase', false);

const leftLines = computed(() => leftText.value.split('\n'));
const rightLines = computed(() => rightText.value.split('\n'));
const hasInput = computed(() => !!leftText.value || !!rightText.value);
const anyNorm = computed(() => ignoreWs.value || ignoreCase.value);

interface InlineSeg {
  val: string;
  hl: boolean;
}

interface DiffLine {
  type: 'eq' | 'add' | 'del';
  lNum: number;
  rNum: number;
  lContent: string;
  rContent: string;
  lSegs?: InlineSeg[];
  rSegs?: InlineSeg[];
}

function charDiff(a: string, b: string): { val: string; type: 'eq' | 'del' | 'add' }[] {
  const aa = [...a];
  const bb = [...b];
  const n = aa.length;
  const m = bb.length;
  if (n === 0) return [{ val: b, type: 'add' }];
  if (m === 0) return [{ val: a, type: 'del' }];

  const dp: number[][] = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i]![j] =
        aa[i] === bb[j] ? dp[i + 1]![j + 1]! + 1 : Math.max(dp[i + 1]![j]!, dp[i]![j + 1]!);
    }
  }

  const result: { val: string; type: 'eq' | 'del' | 'add' }[] = [];
  const push = (val: string, type: 'eq' | 'del' | 'add') => {
    const last = result[result.length - 1];
    if (last && last.type === type) last.val += val;
    else result.push({ val, type });
  };

  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (aa[i] === bb[j]) {
      push(aa[i]!, 'eq');
      i++;
      j++;
    } else if (dp[i + 1]![j]! >= dp[i]![j + 1]!) {
      push(aa[i]!, 'del');
      i++;
    } else {
      push(bb[j]!, 'add');
      j++;
    }
  }
  while (i < n) push(aa[i++]!, 'del');
  while (j < m) push(bb[j++]!, 'add');
  return result;
}

function computeInline(dels: DiffLine[], adds: DiffLine[]): void {
  const pairCount = Math.min(dels.length, adds.length);
  for (let k = 0; k < pairCount; k++) {
    const d = dels[k]!;
    const a = adds[k]!;
    if (d.lContent.length > 500 || a.rContent.length > 500) continue;
    const segs = charDiff(d.lContent, a.rContent);
    d.lSegs = segs
      .filter((s) => s.type !== 'add')
      .map((s) => ({ val: s.val, hl: s.type === 'del' }));
    a.rSegs = segs
      .filter((s) => s.type !== 'del')
      .map((s) => ({ val: s.val, hl: s.type === 'add' }));
  }
}

const lines = computed<DiffLine[]>(() => {
  const ic = ignoreCase.value;
  const iw = ignoreWs.value;
  const norm = (line: string): string => {
    let s = line;
    if (ic) s = s.toLowerCase();
    if (iw) s = s.replace(/\s/g, '');
    return s;
  };

  const aRaw = leftLines.value;
  const bRaw = rightLines.value;
  const a = aRaw.map(norm);
  const b = bRaw.map(norm);
  const n = a.length;
  const m = b.length;

  const dp: number[][] = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i]![j] = a[i] === b[j] ? dp[i + 1]![j + 1]! + 1 : Math.max(dp[i + 1]![j]!, dp[i]![j + 1]!);
    }
  }

  const out: DiffLine[] = [];
  let i = 0;
  let j = 0;
  let lNum = 0;
  let rNum = 0;
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      lNum++;
      rNum++;
      out.push({ type: 'eq', lNum, rNum, lContent: aRaw[i]!, rContent: bRaw[j]! });
      i++;
      j++;
    } else if (dp[i + 1]![j]! >= dp[i]![j + 1]!) {
      lNum++;
      out.push({ type: 'del', lNum, rNum: 0, lContent: aRaw[i]!, rContent: '' });
      i++;
    } else {
      rNum++;
      out.push({ type: 'add', lNum: 0, rNum, lContent: '', rContent: bRaw[j]! });
      j++;
    }
  }
  while (i < n) {
    lNum++;
    out.push({ type: 'del', lNum, rNum: 0, lContent: aRaw[i]!, rContent: '' });
    i++;
  }
  while (j < m) {
    rNum++;
    out.push({ type: 'add', lNum: 0, rNum, lContent: '', rContent: bRaw[j]! });
    j++;
  }

  let k = 0;
  while (k < out.length) {
    if (out[k]!.type === 'del') {
      const delStart = k;
      while (k < out.length && out[k]!.type === 'del') k++;
      const addStart = k;
      while (k < out.length && out[k]!.type === 'add') k++;
      computeInline(out.slice(delStart, addStart), out.slice(addStart, k));
    } else {
      k++;
    }
  }

  return out;
});

const hasDiff = computed(() => lines.value.some((l) => l.type !== 'eq'));

const displayLines = computed(() => {
  if (!onlyDiff.value) return lines.value;
  return lines.value.filter((ln) => ln.type !== 'eq');
});

const stat = computed(() => {
  let add = 0;
  let del = 0;
  let eq = 0;
  for (const l of lines.value) {
    if (l.type === 'add') add++;
    else if (l.type === 'del') del++;
    else eq++;
  }
  return { add, del, eq };
});

const similarity = computed(() => {
  const total = lines.value.length;
  if (total === 0) return 100;
  return Math.round((stat.value.eq / total) * 100);
});

function swap() {
  const t = leftText.value;
  leftText.value = rightText.value;
  rightText.value = t;
}

function clear() {
  leftText.value = '';
  rightText.value = '';
}

async function copyResult() {
  const text = lines.value
    .map((l) => {
      if (l.type === 'add') return `+ ${l.rContent}`;
      if (l.type === 'del') return `- ${l.lContent}`;
      return `  ${l.lContent}`;
    })
    .join('\n');
  await copyToClipboard(text, '已复制差异结果');
}
</script>

<style scoped>
.diff-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 14px;
  flex-wrap: wrap;
}
.sep {
  width: 1px;
  height: 20px;
  background: var(--xuya-border);
  margin: 0 2px;
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
.toggle {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  cursor: pointer;
  user-select: none;
  transition: color var(--xuya-duration-fast);
}
.toggle:hover {
  color: var(--xuya-text);
}
.toggle input {
  accent-color: var(--xuya-accent);
  width: 14px;
  height: 14px;
  cursor: pointer;
}
.diff-stats {
  display: flex;
  gap: 10px;
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  font-weight: 600;
}
.ds.add {
  color: var(--xuya-success);
}
.ds.del {
  color: var(--xuya-danger);
}
.ds.eq {
  color: var(--xuya-text-tertiary);
}
.ds.sim {
  color: var(--xuya-info);
  padding-left: 6px;
  border-left: 1px solid var(--xuya-border);
}

.input-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  margin-bottom: 16px;
}
.input-col {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.col-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.col-head .stat {
  font-size: 10.5px;
  font-weight: 400;
  opacity: 0.75;
}
.editor {
  height: 150px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.65;
  resize: vertical;
  tab-size: 2;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.editor::placeholder {
  color: var(--xuya-text-tertiary);
}
.editor:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

.diff-result {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.result-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}
.same-badge {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-success);
}
.section-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
}
.ph {
  color: var(--xuya-text-tertiary);
  font-size: 13px;
  padding: 24px;
  text-align: center;
}
.ph-ok {
  color: var(--xuya-success);
  font-weight: 600;
}

.split-view {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.split-col {
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  overflow: auto;
}
.split-head {
  position: sticky;
  top: 0;
  padding: 5px 12px;
  font-size: 10.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  color: var(--xuya-text-tertiary);
  background: var(--xuya-bg-subtle);
  border-bottom: 1px solid var(--xuya-border);
  z-index: 1;
}
.unified-view {
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  overflow: auto;
  flex: 1;
  min-height: 200px;
}
.diff-line {
  display: flex;
  gap: 6px;
  padding: 2px 10px;
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  line-height: 1.7;
  min-height: 22px;
}
.diff-line.eq {
  color: var(--xuya-text);
}
.diff-line.add {
  background: var(--xuya-success-soft);
}
.diff-line.del {
  background: var(--xuya-danger-soft);
}
.ln-num {
  width: 34px;
  text-align: right;
  color: var(--xuya-text-tertiary);
  flex-shrink: 0;
  user-select: none;
  font-size: 10.5px;
}
.ln-sign {
  width: 14px;
  flex-shrink: 0;
  user-select: none;
  font-weight: 700;
  text-align: center;
}
.diff-line.add .ln-sign {
  color: var(--xuya-success);
}
.diff-line.del .ln-sign {
  color: var(--xuya-danger);
}
.ln-content {
  white-space: pre-wrap;
  word-break: break-all;
}
.seg {
  border-radius: 2px;
}
.seg-del {
  background: rgba(207, 34, 46, 0.18);
  text-decoration: line-through;
  text-decoration-color: rgba(207, 34, 46, 0.4);
}
.seg-add {
  background: rgba(26, 127, 55, 0.18);
  font-weight: 600;
}
</style>
