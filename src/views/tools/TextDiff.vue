<template>
  <ToolShell title="文本差异对比" :icon="GitCompare" description="双栏输入,行级 diff 高亮,统计增删改。">
    <div class="diff-toolbar">
      <div class="seg">
        <button :class="{ active: mode === 'split' }" @click="mode = 'split'">分栏对比</button>
        <button :class="{ active: mode === 'unified' }" @click="mode = 'unified'">合并视图</button>
      </div>
      <span class="sep"></span>
      <div v-if="lines.length" class="diff-stats">
        <span class="stat add">+{{ stat.add }}</span>
        <span class="stat del">-{{ stat.del }}</span>
        <span class="stat eq">={{ stat.eq }}</span>
      </div>
      <span style="flex:1"></span>
      <BaseButton variant="ghost" @click="swap"><ArrowLeftRight :size="14" /> 交换</BaseButton>
      <BaseButton variant="ghost" @click="clear">清空</BaseButton>
    </div>

    <div class="input-grid">
      <div class="input-col">
        <div class="col-head"><span>原始文本</span><span class="stat">{{ leftText.split('\n').length }} 行</span></div>
        <textarea v-model="leftText" class="editor" placeholder="粘贴原始文本..." spellcheck="false"></textarea>
      </div>
      <div class="input-col">
        <div class="col-head"><span>修改后文本</span><span class="stat">{{ rightText.split('\n').length }} 行</span></div>
        <textarea v-model="rightText" class="editor" placeholder="粘贴修改后文本..." spellcheck="false"></textarea>
      </div>
    </div>

    <div v-if="leftText || rightText" class="diff-result">
      <div class="section-label">差异结果</div>
      <div v-if="!lines.length" class="ph">两段文本完全相同</div>
      <div v-else-if="mode === 'split'" class="split-view">
        <div class="split-col left">
          <div v-for="(ln, i) in lines" :key="'l'+i" class="diff-line" :class="ln.type">
            <span class="ln-num">{{ ln.lNum || '' }}</span>
            <span class="ln-content">{{ ln.lContent }}</span>
          </div>
        </div>
        <div class="split-col right">
          <div v-for="(ln, i) in lines" :key="'r'+i" class="diff-line" :class="ln.type">
            <span class="ln-num">{{ ln.rNum || '' }}</span>
            <span class="ln-content">{{ ln.rContent }}</span>
          </div>
        </div>
      </div>
      <div v-else class="unified-view">
        <div v-for="(ln, i) in lines.filter(l => l.type !== 'eq' || showEqual)" :key="i" class="diff-line" :class="ln.type">
          <span class="ln-sign">{{ ln.type === 'add' ? '+' : ln.type === 'del' ? '-' : ' ' }}</span>
          <span class="ln-content">{{ ln.type === 'del' ? ln.lContent : ln.rContent }}</span>
        </div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { GitCompare, ArrowLeftRight } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';

const mode = ref<'split' | 'unified'>('split');
const showEqual = ref(false);
const leftText = ref('function greet(name) {\n  return "Hello, " + name;\n}\n\nconst x = 1;');
const rightText = ref('function greet(name) {\n  return `Hello, ${name}!`;\n}\n\nconst x = 2;\nconst y = 3;');

interface DiffLine { type: 'eq' | 'add' | 'del'; lNum: number; rNum: number; lContent: string; rContent: string; }

/** LCS 行级 diff */
const lines = computed<DiffLine[]>(() => {
  const a = leftText.value.split('\n');
  const b = rightText.value.split('\n');
  const n = a.length, m = b.length;
  // dp[i][j] = LCS 长度
  const dp: number[][] = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i]![j] = a[i] === b[j] ? dp[i + 1]![j + 1]! + 1 : Math.max(dp[i + 1]![j]!, dp[i]![j + 1]!);
    }
  }
  const out: DiffLine[] = [];
  let i = 0, j = 0, lNum = 0, rNum = 0;
  while (i < n && j < m) {
    if (a[i] === b[j]) { lNum++; rNum++; out.push({ type: 'eq', lNum, rNum, lContent: a[i]!, rContent: b[j]! }); i++; j++; }
    else if (dp[i + 1]![j]! >= dp[i]![j + 1]!) { lNum++; out.push({ type: 'del', lNum, rNum: 0, lContent: a[i]!, rContent: '' }); i++; }
    else { rNum++; out.push({ type: 'add', lNum: 0, rNum, lContent: '', rContent: b[j]! }); j++; }
  }
  while (i < n) { lNum++; out.push({ type: 'del', lNum, rNum: 0, lContent: a[i]!, rContent: '' }); i++; }
  while (j < m) { rNum++; out.push({ type: 'add', lNum: 0, rNum, lContent: '', rContent: b[j]! }); j++; }
  return out;
});

const stat = computed(() => {
  let add = 0, del = 0, eq = 0;
  for (const l of lines.value) { if (l.type === 'add') add++; else if (l.type === 'del') del++; else eq++; }
  return { add, del, eq };
});

function swap() { const t = leftText.value; leftText.value = rightText.value; rightText.value = t; }
function clear() { leftText.value = ''; rightText.value = ''; }
</script>

<style scoped>
.diff-toolbar { display: flex; align-items: center; gap: 10px; margin-bottom: 14px; flex-wrap: wrap; }
.seg { display: inline-flex; background: var(--xuya-input-bg); border-radius: var(--xuya-radius-sm); padding: 2px; gap: 2px; }
.seg button { padding: 5px 14px; font-size: 12px; color: var(--xuya-text-secondary); background: transparent; border: none; border-radius: var(--xuya-radius-sm); transition: .1s; }
.seg button.active { background: var(--xuya-bg-elevated); color: var(--xuya-accent); font-weight: 600; box-shadow: var(--xuya-shadow); }
.sep { width: 1px; height: 20px; background: var(--xuya-border); }
.diff-stats { display: flex; gap: 8px; font-family: var(--xuya-font-mono); font-size: 12px; font-weight: 600; }
.stat.add { color: var(--xuya-success); } .stat.del { color: var(--xuya-danger); } .stat.eq { color: var(--xuya-text-tertiary); }

.input-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-bottom: 16px; }
.input-col { display: flex; flex-direction: column; gap: 6px; }
.col-head { display: flex; justify-content: space-between; font-size: 12px; font-weight: 600; color: var(--xuya-text-secondary); }
.col-head .stat { font-size: 10.5px; font-weight: 400; opacity: .75; }
.editor { height: 140px; padding: 10px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); font-family: var(--xuya-font-mono); font-size: 12px; line-height: 1.5; resize: vertical; transition: border-color .12s, box-shadow .12s; }
.editor:focus { outline: none; border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }

.diff-result { flex: 1; min-height: 0; }
.section-label { font-size: 11px; text-transform: uppercase; letter-spacing: .5px; color: var(--xuya-text-tertiary); margin-bottom: 8px; }
.ph { color: var(--xuya-text-tertiary); font-size: 13px; padding: 20px; text-align: center; }

.split-view { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; overflow: auto; max-height: 360px; }
.split-col { background: var(--xuya-input-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius); overflow: auto; }
.unified-view { background: var(--xuya-input-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius); overflow: auto; max-height: 360px; }
.diff-line { display: flex; gap: 8px; padding: 2px 10px; font-family: var(--xuya-font-mono); font-size: 12px; line-height: 1.6; min-height: 22px; }
.diff-line.eq { color: var(--xuya-text); }
.diff-line.add { background: var(--xuya-success-soft); color: var(--xuya-success); }
.diff-line.del { background: var(--xuya-danger-soft); color: var(--xuya-danger); }
.ln-num { width: 32px; text-align: right; color: var(--xuya-text-tertiary); flex-shrink: 0; user-select: none; }
.ln-sign { width: 16px; flex-shrink: 0; user-select: none; font-weight: 700; }
.ln-content { white-space: pre-wrap; word-break: break-all; }
</style>
