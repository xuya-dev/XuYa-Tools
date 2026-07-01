<template>
  <ToolShell
    title="正则表达式测试"
    :icon="Regex"
    description="实时匹配高亮、捕获组提取、替换预览、标志位切换。"
  >
    <div class="regex-top">
      <div class="pattern-row">
        <span class="pat-slash">/</span>
        <input
          v-model="pattern"
          class="pat-input"
          type="text"
          placeholder="输入正则表达式，如 \d+(\.\d+)?"
          spellcheck="false"
        />
        <span class="pat-slash">/{{ flags }}</span>
        <div class="flag-toggles">
          <button
            v-for="f in FLAG_LIST"
            :key="f.ch"
            class="flag-btn"
            :class="{ active: flags.includes(f.ch) }"
            :title="f.desc"
            @click="toggleFlag(f.ch)"
          >
            <span class="flag-ch">{{ f.ch }}</span>{{ f.short }}
          </button>
        </div>
      </div>
      <div v-if="regexError" class="regex-error">{{ regexError }}</div>
    </div>

    <div class="regex-toolbar">
      <div class="seg">
        <button :class="{ active: mode === 'match' }" @click="mode = 'match'">匹配</button>
        <button :class="{ active: mode === 'replace' }" @click="mode = 'replace'">替换</button>
      </div>

      <template v-if="mode === 'replace'">
        <span class="sep"></span>
        <input
          v-model="replacement"
          class="repl-input"
          placeholder="替换为…（支持 $1 $2 $& 等）"
          spellcheck="false"
        />
      </template>

      <span style="flex: 1"></span>

      <span class="tb-stat" :class="{ ok: matchCount > 0, err: !!regexError }">
        {{ mode === 'replace' ? `${matchCount} 处替换` : `${matchCount} 处匹配` }}
      </span>

      <span class="sep"></span>

      <BaseButton variant="ghost" :disabled="!pattern" @click="copyRegex">复制正则</BaseButton>
      <BaseButton variant="ghost" :disabled="!hasResult" @click="copyResult">复制结果</BaseButton>
      <BaseButton variant="ghost" @click="clearText">清空文本</BaseButton>
    </div>

    <div class="io-grid">
      <div class="io-col">
        <div class="col-head">
          <span>测试文本</span>
          <span class="stat">
            {{ testText.length }} 字符 · {{ testText.split('\n').length }} 行
          </span>
        </div>
        <textarea
          v-model="testText"
          class="editor"
          placeholder="在此输入要匹配的文本…"
          spellcheck="false"
        ></textarea>
      </div>
      <div class="io-col">
        <div class="col-head">
          <span>{{ mode === 'match' ? '匹配结果' : '替换结果' }}</span>
          <span class="stat" :class="{ ok: matchCount > 0 }">{{ resultStat }}</span>
        </div>
        <div class="result-view" v-html="outputHtml"></div>
      </div>
    </div>

    <div v-if="matchDetails.length" class="match-details">
      <div class="detail-header">
        <span class="section-label">匹配详情</span>
        <span class="detail-count">{{ matchDetails.length }} 条</span>
      </div>
      <div class="detail-list">
        <div
          v-for="d in matchDetails"
          :key="d.index"
          class="detail-row"
          :title="`点击复制: ${d.full}`"
          @click="copyText(d.full)"
        >
          <span class="d-num">#{{ d.index + 1 }}</span>
          <span class="d-pos">{{ d.start }}–{{ d.end }}</span>
          <span class="d-full">{{ d.full || '(空匹配)' }}</span>
          <template v-if="d.groups.length">
            <span
              v-for="(g, gi) in d.groups"
              :key="gi"
              class="d-group"
              :title="`点击复制 $${gi + 1}`"
              @click.stop="copyText(g)"
            >
              <span class="dg-label">${{ gi + 1 }}</span>
              <span class="dg-val">{{ g || '(空)' }}</span>
            </span>
          </template>
        </div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Regex } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const FLAG_LIST = [
  { ch: 'g', short: '全局', desc: '全局匹配（匹配全部，而非第一个）' },
  { ch: 'i', short: '忽略大小写', desc: '忽略大小写' },
  { ch: 'm', short: '多行', desc: '多行模式（^ $ 匹配每行边界）' },
  { ch: 's', short: '.含换行', desc: '. 匹配换行符' },
  { ch: 'u', short: 'Unicode', desc: 'Unicode 模式' },
  { ch: 'y', short: '粘性', desc: '粘性匹配（从 lastIndex 开始）' },
];

const pattern = useToolState('regex', 'pattern', '\\d+');
const flags = useToolState('regex', 'flags', 'g');
const testText = useToolState(
  'regex',
  'testText',
  '订单号 20240115 共 3 件，金额 99.5 元。电话 13800138000。',
);
const mode = useToolState<'match' | 'replace'>('regex', 'mode', 'match');
const replacement = useToolState('regex', 'replacement', '[$&]');

const flagSet = computed(() => new Set(flags.value.split('')));

function toggleFlag(ch: string) {
  const s = new Set(flags.value);
  if (s.has(ch)) s.delete(ch);
  else s.add(ch);
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

const matchHtml = computed(() => {
  const { re } = compiled.value;
  const text = testText.value;
  if (!re || !text) {
    return escapeHtml(text) || '<span class="ph">在左侧输入文本后查看匹配</span>';
  }
  if (!matches.value.length) return escapeHtml(text);
  let result = '';
  let last = 0;
  for (const m of matches.value) {
    const idx = m.index ?? 0;
    result += escapeHtml(text.slice(last, idx));
    result += `<mark>${escapeHtml(m[0])}</mark>`;
    last = idx + m[0].length;
  }
  result += escapeHtml(text.slice(last));
  return result;
});

const replaceParts = computed<{ text: string; replaced: boolean }[]>(() => {
  const text = testText.value;
  if (!compiled.value.re || !text || !matches.value.length) {
    return [{ text, replaced: false }];
  }
  const individualRe = new RegExp(pattern.value, flags.value.replace('g', ''));
  const parts: { text: string; replaced: boolean }[] = [];
  let last = 0;
  for (const m of matches.value) {
    const idx = m.index ?? 0;
    const matched = m[0];
    if (idx > last) parts.push({ text: text.slice(last, idx), replaced: false });
    const repl = matched.replace(individualRe, replacement.value);
    parts.push({ text: repl, replaced: true });
    last = idx + matched.length;
  }
  if (last < text.length) parts.push({ text: text.slice(last), replaced: false });
  return parts;
});

const replaceResult = computed(() => replaceParts.value.map((p) => p.text).join(''));

const replaceHtml = computed(() => {
  if (!testText.value) return '<span class="ph">替换结果将显示在这里</span>';
  return replaceParts.value
    .map((p) =>
      p.replaced ? `<mark class="replaced">${escapeHtml(p.text)}</mark>` : escapeHtml(p.text),
    )
    .join('');
});

const outputHtml = computed(() => {
  if (regexError.value) return escapeHtml(testText.value || '');
  return mode.value === 'replace' ? replaceHtml.value : matchHtml.value;
});

const resultStat = computed(() => {
  if (regexError.value) return '错误';
  if (!testText.value) return '';
  if (matchCount.value === 0) return '无匹配';
  if (mode.value === 'replace') return `${matchCount.value} 处替换`;
  return `${matchCount.value} 处匹配`;
});

const hasResult = computed(() => {
  if (mode.value === 'replace') return !!replaceResult.value;
  return matches.value.length > 0;
});

interface MatchDetail {
  index: number;
  start: number;
  end: number;
  full: string;
  groups: string[];
}

const matchDetails = computed<MatchDetail[]>(() => {
  return matches.value.map((m, i) => ({
    index: i,
    start: m.index ?? 0,
    end: (m.index ?? 0) + m[0].length,
    full: m[0],
    groups: m.slice(1).filter((v) => v !== undefined) as string[],
  }));
});

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

async function copyRegex() {
  if (!pattern.value) return;
  const code = `/${pattern.value}/${flags.value}`;
  await copyToClipboard(code, `已复制: ${code}`);
}

async function copyResult() {
  if (mode.value === 'replace') {
    await copyToClipboard(replaceResult.value, '已复制替换结果');
  } else {
    const text = matches.value.map((m) => m[0]).join('\n');
    await copyToClipboard(text, `已复制 ${matches.value.length} 个匹配`);
  }
}

async function copyText(v: string) {
  await copyToClipboard(v, `已复制: ${v.length > 40 ? v.slice(0, 40) + '…' : v}`);
}

function clearText() {
  testText.value = '';
}
</script>

<style scoped>
.regex-top {
  margin-bottom: 14px;
}
.pattern-row {
  display: flex;
  align-items: center;
  gap: 0;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  transition:
    border-color var(--xuya-duration-fast) var(--xuya-ease),
    box-shadow var(--xuya-duration-fast) var(--xuya-ease);
}
.pattern-row:focus-within {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.pat-slash {
  padding: 0 6px 0 14px;
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
  font-size: 14px;
}
.pat-input {
  flex: 1;
  border: none;
  background: none;
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 14px;
  padding: 11px 0;
  outline: none;
}
.pat-input::placeholder {
  color: var(--xuya-text-tertiary);
}
.flag-toggles {
  display: flex;
  gap: 2px;
  padding: 0 6px 0 4px;
}
.flag-btn {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  height: 26px;
  padding: 0 8px;
  border-radius: var(--xuya-radius-sm);
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  white-space: nowrap;
}
.flag-ch {
  font-family: var(--xuya-font-mono);
  font-weight: 700;
  font-size: 12px;
}
.flag-btn:hover {
  background: var(--xuya-border);
  color: var(--xuya-text);
}
.flag-btn.active {
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent);
  font-weight: 600;
}
.regex-error {
  margin-top: 8px;
  padding: 8px 12px;
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
  border-radius: var(--xuya-radius-sm);
  font-size: 12.5px;
  font-family: var(--xuya-font-mono);
}

.regex-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 14px;
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
.repl-input {
  flex: 1;
  min-width: 160px;
  padding: 6px 12px;
  font-size: 12.5px;
  font-family: var(--xuya-font-mono);
  color: var(--xuya-text);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  outline: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.repl-input::placeholder {
  color: var(--xuya-text-tertiary);
}
.repl-input:focus {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.tb-stat {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.tb-stat.ok {
  color: var(--xuya-success);
}
.tb-stat.err {
  color: var(--xuya-danger);
}

.io-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
  margin-bottom: 16px;
}
.io-col {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 0;
}
.col-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.stat {
  font-size: 10.5px;
  font-weight: 400;
  opacity: 0.75;
}
.stat.ok {
  color: var(--xuya-success);
  opacity: 1;
}
.editor {
  flex: 1;
  min-height: 220px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.65;
  resize: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
  tab-size: 2;
}
.editor::placeholder {
  color: var(--xuya-text-tertiary);
}
.editor:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.result-view {
  flex: 1;
  min-height: 220px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.65;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
  tab-size: 2;
}
.result-view :deep(mark) {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
  border-radius: var(--xuya-radius-sm);
  padding: 1px 2px;
  font-weight: 600;
}
.result-view :deep(mark.replaced) {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.result-view :deep(.ph) {
  color: var(--xuya-text-tertiary);
}

.match-details {
  flex-shrink: 0;
}
.detail-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}
.detail-count {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  font-weight: 400;
}
.detail-list {
  max-height: 200px;
  overflow: auto;
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  background: var(--xuya-input-bg);
}
.detail-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 5px 12px;
  font-size: 12px;
  font-family: var(--xuya-font-mono);
  line-height: 1.6;
  cursor: pointer;
  border-bottom: 1px solid var(--xuya-border-light);
  transition: background var(--xuya-duration-fast);
}
.detail-row:last-child {
  border-bottom: none;
}
.detail-row:hover {
  background: var(--xuya-accent-soft);
}
.d-num {
  color: var(--xuya-text-tertiary);
  font-weight: 600;
  flex-shrink: 0;
  min-width: 28px;
}
.d-pos {
  color: var(--xuya-info);
  font-size: 10.5px;
  flex-shrink: 0;
  min-width: 56px;
}
.d-full {
  color: var(--xuya-warn);
  font-weight: 500;
  word-break: break-all;
}
.d-group {
  display: inline-flex;
  align-items: baseline;
  gap: 3px;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--xuya-bg-elevated);
  border: 1px solid var(--xuya-border);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
  flex-shrink: 0;
}
.d-group:hover {
  border-color: var(--xuya-accent);
}
.dg-label {
  color: var(--xuya-purple);
  font-weight: 600;
  font-size: 10px;
}
.dg-val {
  color: var(--xuya-text);
  word-break: break-all;
}

.section-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
}
</style>
