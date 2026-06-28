<template>
  <ToolShell title="正则表达式测试" :icon="Regex" description="实时匹配高亮、捕获组提取、标志位切换、常用速查。">
    <div class="regex-top">
      <div class="pattern-row">
        <span class="pat-slash">/</span>
        <input
          v-model="pattern"
          class="pat-input"
          type="text"
          placeholder="输入正则表达式, 如 \\d+(\\.\\d+)?"
          spellcheck="false"
        />
        <span class="pat-slash">/{{ flags.replace(/[gimsuy]/g, '') }}</span>
        <div class="flag-toggles">
          <button
            v-for="f in FLAG_LIST"
            :key="f.ch"
            class="flag-btn"
            :class="{ active: flags.includes(f.ch) }"
            :title="f.desc"
            @click="toggleFlag(f.ch)"
          >{{ f.ch }}</button>
        </div>
      </div>
      <div v-if="regexError" class="regex-error">⚠️ {{ regexError }}</div>
    </div>

    <div class="io-grid">
      <div class="io-col">
        <div class="col-head"><span>测试文本</span><span class="stat">{{ testText.length }} 字符</span></div>
        <textarea
          v-model="testText"
          class="editor"
          placeholder="在此输入要匹配的文本..."
          spellcheck="false"
        ></textarea>
      </div>
      <div class="io-col">
        <div class="col-head"><span>匹配结果</span><span class="stat" :class="{ ok: matchCount, err: regexError }">{{ matchCount }} 处匹配</span></div>
        <div class="result-view" v-html="highlighted"></div>
      </div>
    </div>

    <div class="captures" v-if="captureGroups.length">
      <div class="section-label">捕获组 ({{ captureGroups.length }} 组)</div>
      <div class="capture-grid">
        <div v-for="(g, gi) in captureGroups" :key="gi" class="capture-card">
          <div class="cap-head">第 {{ gi }} 组 · {{ g.length }} 个</div>
          <div v-for="(val, vi) in g" :key="vi" class="cap-val mono" @click="copy(val)">
            {{ val || '(空)' }}
          </div>
        </div>
      </div>
    </div>

    <details class="cheatsheet">
      <summary>常用正则速查</summary>
      <div class="cs-grid">
        <button v-for="c in CHEATS" :key="c.name" class="cs-item" @click="apply(c)">
          <span class="cs-name">{{ c.name }}</span>
          <code class="mono">{{ c.pattern }}</code>
        </button>
      </div>
    </details>
  </ToolShell>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Regex } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const FLAG_LIST = [
  { ch: 'g', desc: '全局匹配' },
  { ch: 'i', desc: '忽略大小写' },
  { ch: 'm', desc: '多行模式' },
  { ch: 's', desc: '. 匹配换行' },
  { ch: 'u', desc: 'Unicode' },
  { ch: 'y', desc: '粘性匹配' },
];
const CHEATS = [
  { name: '邮箱', pattern: '[\\w.+-]+@[\\w-]+\\.[\\w.-]+' },
  { name: 'URL', pattern: 'https?://[^\\s]+' },
  { name: '手机号', pattern: '1[3-9]\\d{9}' },
  { name: 'IPv4', pattern: '\\b(?:\\d{1,3}\\.){3}\\d{1,3}\\b' },
  { name: '数字', pattern: '\\d+(\\.\\d+)?' },
  { name: '中文字符', pattern: '[\\u4e00-\\u9fa5]+' },
  { name: '身份证', pattern: '\\d{17}[\\dXx]' },
  { name: '十六进制', pattern: '#?[0-9a-fA-F]{3,8}' },
];

const pattern = useToolState('regex', 'pattern', '\\d+');
const flags = useToolState('regex', 'flags', 'g');
const testText = useToolState('regex', 'testText', '订单号 20240115 共 3 件,金额 99.5 元。电话 13800138000。');

const flagSet = computed(() => new Set(flags.value.split('')));

function toggleFlag(ch: string) {
  const s = new Set(flags.value);
  if (s.has(ch)) s.delete(ch);
  else s.add(ch);
  // 保留顺序 g i m s u y
  flags.value = ['g', 'i', 'm', 's', 'u', 'y'].filter((f) => s.has(f)).join('');
}

const compiled = computed<{ re: RegExp | null; error: string }>(() => {
  if (!pattern.value) return { re: null, error: '' };
  try {
    return { re: new RegExp(pattern.value, flags.value || undefined), error: '' };
  } catch (e) {
    return { re: null, error: e instanceof Error ? e.message : String(e) };
  }
});
const regexError = computed(() => compiled.value.error);

const matches = computed<RegExpMatchArray[]>(() => {
  const { re } = compiled.value;
  if (!re || !testText.value) return [];
  const arr: RegExpMatchArray[] = [];
  if (flagSet.value.has('g')) {
    for (const m of testText.value.matchAll(re)) arr.push(m);
  } else {
    const m = testText.value.match(re);
    if (m) arr.push(m);
  }
  return arr;
});
const matchCount = computed(() => matches.value.length);

/** 高亮匹配位置 */
const highlighted = computed(() => {
  const { re } = compiled.value;
  const text = testText.value;
  if (!re || !text) return escapeHtml(text || '') || '<span class="ph">在左侧输入文本后查看匹配</span>';
  if (!matches.value.length) return escapeHtml(text);
  // 用 matchAll 重建带高亮
  let result = '';
  let last = 0;
  const sorted = [...matches.value].sort((a, b) => (a.index ?? 0) - (b.index ?? 0));
  for (const m of sorted) {
    const idx = m.index ?? 0;
    result += escapeHtml(text.slice(last, idx));
    result += `<mark>${escapeHtml(m[0])}</mark>`;
    last = idx + m[0].length;
  }
  result += escapeHtml(text.slice(last));
  return result;
});

/** 捕获组: 每组一个数组 */
const captureGroups = computed<string[][]>(() => {
  const groups: string[][] = [];
  const maxGroups = matches.value.reduce((mx, m) => Math.max(mx, m.length - 1), 0);
  for (let gi = 1; gi <= maxGroups; gi++) {
    const vals: string[] = [];
    for (const m of matches.value) {
      if (m[gi] !== undefined) vals.push(m[gi]!);
    }
    if (vals.length) groups.push(vals);
  }
  return groups;
});

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
function apply(c: { name: string; pattern: string }) {
  pattern.value = c.pattern;
}
async function copy(v: string) {
  if (v) await copyToClipboard(v, '已复制捕获值');
}
</script>

<style scoped>
.regex-top { margin-bottom: 16px; }
.pattern-row {
  display: flex; align-items: center; gap: 0;
  background: var(--xuya-input-bg); border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius); transition: border-color .12s, box-shadow .12s;
}
.pattern-row:focus-within { border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }
.pat-slash { padding: 0 6px 0 14px; color: var(--xuya-text-tertiary); font-family: var(--xuya-font-mono); font-size: 14px; }
.pat-input { flex: 1; border: none; background: none; color: var(--xuya-text); font-family: var(--xuya-font-mono); font-size: 14px; padding: 11px 0; outline: none; }
.pat-input::placeholder { color: var(--xuya-text-tertiary); }
.flag-toggles { display: flex; gap: 2px; padding: 0 6px 0 4px; }
.flag-btn { width: 26px; height: 26px; border-radius: 6px; font-size: 12px; font-family: var(--xuya-font-mono); color: var(--xuya-text-tertiary); transition: .1s; }
.flag-btn:hover { background: var(--xuya-border); color: var(--xuya-text); }
.flag-btn.active { background: var(--xuya-accent-soft); color: var(--xuya-accent); font-weight: 700; }
.regex-error { margin-top: 8px; padding: 8px 12px; background: var(--xuya-danger-soft); color: var(--xuya-danger); border-radius: var(--xuya-radius-sm); font-size: 12.5px; font-family: var(--xuya-font-mono); }

.io-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; flex: 1; min-height: 0; margin-bottom: 16px; }
.io-col { display: flex; flex-direction: column; gap: 6px; min-height: 0; }
.col-head { display: flex; justify-content: space-between; font-size: 12px; font-weight: 600; color: var(--xuya-text-secondary); }
.stat { font-size: 10.5px; font-weight: 400; opacity: .75; }
.stat.ok { color: var(--xuya-success); opacity: 1; }
.stat.err { color: var(--xuya-danger); opacity: 1; }
.editor { flex: 1; min-height: 220px; padding: 12px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); font-family: var(--xuya-font-mono); font-size: 12.5px; line-height: 1.6; resize: none; transition: border-color .12s, box-shadow .12s; }
.editor:focus { outline: none; border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }
.result-view { flex: 1; min-height: 220px; padding: 12px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); font-family: var(--xuya-font-mono); font-size: 12.5px; line-height: 1.6; overflow: auto; white-space: pre-wrap; word-break: break-all; }
.result-view :deep(mark) { background: var(--xuya-warn-soft); color: var(--xuya-warn); border-radius: 3px; padding: 1px 2px; font-weight: 600; }
.result-view :deep(.ph) { color: var(--xuya-text-tertiary); }

.captures { margin-bottom: 16px; }
.capture-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 10px; }
.capture-card { background: var(--xuya-input-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-sm); padding: 10px 12px; }
.cap-head { font-size: 11px; color: var(--xuya-text-tertiary); margin-bottom: 6px; }
.cap-val { font-size: 12px; color: var(--xuya-text); padding: 3px 0; cursor: pointer; border-bottom: 1px dashed var(--xuya-border); }
.cap-val:hover { color: var(--xuya-accent); }
.cap-val:last-child { border-bottom: none; }

.cheatsheet { background: var(--xuya-card-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius); padding: 12px 16px; }
.cheatsheet summary { cursor: pointer; font-size: 13px; font-weight: 600; color: var(--xuya-text-secondary); }
.cs-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 8px; margin-top: 12px; }
.cs-item { display: flex; flex-direction: column; gap: 4px; text-align: left; padding: 8px 10px; background: var(--xuya-input-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-sm); transition: .1s; }
.cs-item:hover { border-color: var(--xuya-accent); }
.cs-name { font-size: 12px; color: var(--xuya-text); font-weight: 500; }
.cs-item code { font-size: 10.5px; color: var(--xuya-accent); word-break: break-all; }
.section-label { font-size: 11px; text-transform: uppercase; letter-spacing: .5px; color: var(--xuya-text-tertiary); margin-bottom: 8px; }
</style>
