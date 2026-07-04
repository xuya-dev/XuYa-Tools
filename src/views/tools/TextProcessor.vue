<template>
  <ToolShell
    title="文本处理"
    :icon="Type"
    description="大小写转换、去重、排序、提取、统计，一键处理文本。"
  >
    <div class="text-toolbar">
      <div class="seg">
        <button
          v-for="m in MODES"
          :key="m.id"
          :class="{ active: mode === m.id }"
          @click="mode = m.id"
        >
          {{ m.label }}
        </button>
      </div>
      <span class="sep"></span>
      <BaseButton variant="ghost" :disabled="!output" @click="copyOutput">
        <Copy :size="13" />
        复制结果
      </BaseButton>
      <BaseButton variant="ghost" :disabled="!input" @click="input = ''">清空</BaseButton>
    </div>

    <!-- 查找替换面板 -->
    <div v-if="mode === 'find_replace'" class="extra-panel">
      <input v-model="findText" class="extra-input" placeholder="查找内容…" spellcheck="false" />
      <input v-model="replaceText" class="extra-input" placeholder="替换为…" spellcheck="false" />
      <label class="toggle-label">
        <input v-model="useRegex" type="checkbox" class="toggle-check" />
        正则
      </label>
      <BaseButton variant="ghost" :disabled="!output" @click="applyReplace">应用替换</BaseButton>
    </div>

    <!-- 过滤行面板 -->
    <div v-if="mode === 'filter_lines'" class="extra-panel">
      <input
        v-model="filterPattern"
        class="extra-input"
        placeholder="正则表达式 (如 ^ERROR)…"
        spellcheck="false"
      />
      <div class="seg seg-sm">
        <button :class="{ active: filterMode === 'keep' }" @click="filterMode = 'keep'">
          保留匹配
        </button>
        <button :class="{ active: filterMode === 'remove' }" @click="filterMode = 'remove'">
          移除匹配
        </button>
      </div>
    </div>

    <div class="text-grid">
      <div class="text-col">
        <div class="col-head">
          <span>输入</span>
          <span class="stat">{{ inputStats }}</span>
        </div>
        <textarea
          v-model="input"
          class="editor"
          placeholder="在此输入文本…"
          spellcheck="false"
        ></textarea>
      </div>
      <div class="text-col">
        <div class="col-head">
          <span>输出</span>
          <span class="stat">{{ outputStats }}</span>
        </div>
        <pre class="output" @click="copyOutput"><code>{{ output || '—' }}</code></pre>
      </div>
    </div>

    <!-- 统计信息 -->
    <div v-if="input" class="stats-grid">
      <div class="stat-card">
        <span class="stat-val">{{ charCount }}</span>
        <span class="stat-lbl">字符</span>
      </div>
      <div class="stat-card">
        <span class="stat-val">{{ charCountNoSpace }}</span>
        <span class="stat-lbl">不含空格</span>
      </div>
      <div class="stat-card">
        <span class="stat-val">{{ lineCount }}</span>
        <span class="stat-lbl">行数</span>
      </div>
      <div class="stat-card">
        <span class="stat-val">{{ wordCount }}</span>
        <span class="stat-lbl">单词</span>
      </div>
      <div class="stat-card">
        <span class="stat-val">{{ byteCount }}</span>
        <span class="stat-lbl">字节</span>
      </div>
      <div class="stat-card">
        <span class="stat-val">{{ readTime }}</span>
        <span class="stat-lbl">阅读时间</span>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Type, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

type Mode =
  | 'upper'
  | 'lower'
  | 'title'
  | 'capitalize'
  | 'camel'
  | 'snake'
  | 'kebab'
  | 'const_case'
  | 'sort_asc'
  | 'sort_desc'
  | 'sort_natural'
  | 'dedup_lines'
  | 'dedup_words'
  | 'trim_lines'
  | 'remove_empty'
  | 'remove_nums'
  | 'remove_punct'
  | 'reverse_lines'
  | 'reverse_text'
  | 'extract_urls'
  | 'extract_emails'
  | 'extract_numbers'
  | 'add_prefix'
  | 'add_suffix'
  | 'add_line_nums'
  | 'find_replace'
  | 'filter_lines'
  | 'count_lines';

const MODES: { id: Mode; label: string }[] = [
  { id: 'upper', label: '大写' },
  { id: 'lower', label: '小写' },
  { id: 'title', label: '标题' },
  { id: 'capitalize', label: '首字母' },
  { id: 'camel', label: '驼峰' },
  { id: 'snake', label: '下划线' },
  { id: 'kebab', label: '短横线' },
  { id: 'const_case', label: '常量' },
  { id: 'sort_asc', label: '升序' },
  { id: 'sort_desc', label: '降序' },
  { id: 'dedup_lines', label: '去重行' },
  { id: 'trim_lines', label: '去首尾空白' },
  { id: 'remove_empty', label: '删空行' },
  { id: 'remove_nums', label: '删数字' },
  { id: 'remove_punct', label: '删标点' },
  { id: 'reverse_lines', label: '反序行' },
  { id: 'reverse_text', label: '反转字' },
  { id: 'extract_urls', label: '提取URL' },
  { id: 'extract_emails', label: '提取邮箱' },
  { id: 'extract_numbers', label: '提取数字' },
  { id: 'add_prefix', label: '加前缀' },
  { id: 'add_suffix', label: '加后缀' },
  { id: 'add_line_nums', label: '行号' },
  { id: 'find_replace', label: '查找替换' },
  { id: 'filter_lines', label: '过滤行' },
  { id: 'count_lines', label: '行频统计' },
];

const input = useToolState('textproc', 'input', '');
const mode = useToolState<Mode>('textproc', 'mode', 'upper');
const output = ref('');

const findText = useToolState('textproc', 'find', '');
const replaceText = useToolState('textproc', 'replace', '');
const useRegex = useToolState('textproc', 'useRegex', false);
const filterPattern = useToolState('textproc', 'filterPattern', '');
const filterMode = useToolState<'keep' | 'remove'>('textproc', 'filterMode', 'keep');

function process() {
  const text = input.value;
  if (!text) {
    output.value = '';
    return;
  }
  const m = mode.value;
  switch (m) {
    case 'upper':
      output.value = text.toUpperCase();
      break;
    case 'lower':
      output.value = text.toLowerCase();
      break;
    case 'title':
      output.value = text.replace(/\b\w/g, (c) => c.toUpperCase());
      break;
    case 'capitalize':
      output.value = text.charAt(0).toUpperCase() + text.slice(1);
      break;
    case 'camel':
      output.value = text
        .replace(/[^a-zA-Z0-9]+(.)/g, (_, c) => c.toUpperCase())
        .replace(/^[A-Z]/, (c) => c.toLowerCase());
      break;
    case 'snake':
      output.value = text
        .replace(/([a-z])([A-Z])/g, '$1_$2')
        .replace(/[\s.-]+/g, '_')
        .replace(/_+/g, '_')
        .toLowerCase();
      break;
    case 'kebab':
      output.value = text
        .replace(/([a-z])([A-Z])/g, '$1-$2')
        .replace(/[\s_.]+/g, '-')
        .replace(/-+/g, '-')
        .toLowerCase();
      break;
    case 'const_case':
      output.value = text
        .replace(/([a-z])([A-Z])/g, '$1_$2')
        .replace(/[\s.-]+/g, '_')
        .replace(/_+/g, '_')
        .toUpperCase();
      break;
    case 'sort_asc':
      output.value = text.split('\n').sort().join('\n');
      break;
    case 'sort_desc':
      output.value = text.split('\n').sort().reverse().join('\n');
      break;
    case 'sort_natural':
      output.value = text
        .split('\n')
        .sort((a, b) => a.localeCompare(b, 'zh-CN', { numeric: true }))
        .join('\n');
      break;
    case 'dedup_lines': {
      const seen = new Set<string>();
      output.value = text
        .split('\n')
        .filter((l) => {
          if (seen.has(l)) return false;
          seen.add(l);
          return true;
        })
        .join('\n');
      break;
    }
    case 'dedup_words': {
      const seen = new Set<string>();
      output.value = text
        .split(/[\s,;]+/)
        .filter((w) => {
          if (!w || seen.has(w)) return false;
          seen.add(w);
          return true;
        })
        .join(' ');
      break;
    }
    case 'trim_lines':
      output.value = text
        .split('\n')
        .map((l) => l.trim())
        .join('\n');
      break;
    case 'remove_empty':
      output.value = text
        .split('\n')
        .filter((l) => l.trim())
        .join('\n');
      break;
    case 'remove_nums':
      output.value = text.replace(/[0-9]/g, '');
      break;
    case 'remove_punct':
      output.value = text.replace(/[^\w\s\u4e00-\u9fa5]/g, '');
      break;
    case 'reverse_lines':
      output.value = text.split('\n').reverse().join('\n');
      break;
    case 'reverse_text':
      output.value = [...text].reverse().join('');
      break;
    case 'extract_urls':
      output.value = (text.match(/https?:\/\/[^\s<>"']+/g) ?? []).join('\n');
      break;
    case 'extract_emails':
      output.value = (text.match(/[\w.+-]+@[\w-]+\.[\w.-]+/g) ?? []).join('\n');
      break;
    case 'extract_numbers':
      output.value = (text.match(/-?\d+\.?\d*/g) ?? []).join('\n');
      break;
    case 'add_prefix':
      output.value = text
        .split('\n')
        .map((l) => `> ${l}`)
        .join('\n');
      break;
    case 'add_suffix':
      output.value = text
        .split('\n')
        .map((l) => `${l} …`)
        .join('\n');
      break;
    case 'add_line_nums':
      output.value = text
        .split('\n')
        .map((l, i) => `${String(i + 1).padStart(4, ' ')}  ${l}`)
        .join('\n');
      break;
    case 'find_replace': {
      if (!findText.value) {
        output.value = text;
        break;
      }
      try {
        if (useRegex.value) {
          output.value = text.replace(new RegExp(findText.value, 'g'), replaceText.value);
        } else {
          output.value = text.split(findText.value).join(replaceText.value);
        }
      } catch {
        output.value = text;
      }
      break;
    }
    case 'filter_lines': {
      const pattern = filterPattern.value;
      if (!pattern) {
        output.value = text;
        break;
      }
      let regex: RegExp;
      try {
        regex = new RegExp(pattern);
      } catch {
        output.value = text;
        break;
      }
      const keep = filterMode.value === 'keep';
      output.value = text
        .split('\n')
        .filter((l) => regex.test(l) === keep)
        .join('\n');
      break;
    }
    case 'count_lines': {
      const freq = new Map<string, number>();
      for (const line of text.split('\n')) {
        freq.set(line, (freq.get(line) ?? 0) + 1);
      }
      const sorted = [...freq.entries()].sort((a, b) => b[1] - a[1]).slice(0, 10);
      output.value = sorted
        .map(([line, count]) => `${String(count).padStart(4, ' ')}\u00d7  ${line}`)
        .join('\n');
      break;
    }
  }
}

watch([input, mode, findText, replaceText, useRegex, filterPattern, filterMode], process, {
  immediate: true,
});

const charCount = computed(() => input.value.length);
const charCountNoSpace = computed(() => input.value.replace(/\s/g, '').length);
const lineCount = computed(() => (input.value ? input.value.split('\n').length : 0));
const byteCount = computed(() => new TextEncoder().encode(input.value).length);
const wordCount = computed(() => {
  const cjk = (input.value.match(/[\u4e00-\u9fa5]/g) ?? []).length;
  const en = input.value
    .replace(/[\u4e00-\u9fa5]/g, ' ')
    .trim()
    .split(/\s+/)
    .filter(Boolean).length;
  return cjk + en;
});
const readTime = computed(() => {
  const words = wordCount.value;
  const mins = Math.ceil(words / 300);
  return mins < 1 ? '<1分钟' : `${mins}分钟`;
});

const inputStats = computed(() => `${charCount.value} 字符 · ${lineCount.value} 行`);
const outputStats = computed(() => {
  if (!output.value) return '';
  return `${output.value.length} 字符`;
});

async function copyOutput() {
  if (output.value) await copyToClipboard(output.value, '已复制结果');
}

function applyReplace() {
  if (output.value) input.value = output.value;
}
</script>

<style scoped>
.text-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 14px;
}
.seg {
  display: inline-flex;
  flex-wrap: wrap;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  padding: 2px;
  gap: 2px;
}
.seg button {
  padding: 4px 10px;
  font-size: 11.5px;
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
.sep {
  width: 1px;
  height: 20px;
  background: var(--xuya-border);
  margin: 0 2px;
}

.extra-panel {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 14px;
  padding: 10px 12px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.extra-input {
  flex: 1;
  min-width: 120px;
  padding: 6px 10px;
  font-size: 12.5px;
  font-family: var(--xuya-font-mono);
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-card-bg);
  color: var(--xuya-text);
  outline: none;
  transition: border-color var(--xuya-duration-fast);
}
.extra-input:focus {
  border-color: var(--xuya-accent);
}
.toggle-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--xuya-text-secondary);
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
}
.toggle-check {
  accent-color: var(--xuya-accent);
  cursor: pointer;
}
.seg-sm button {
  font-size: 11px;
  padding: 4px 10px;
}

.text-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
  margin-bottom: 16px;
}
.text-col {
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

.editor {
  flex: 1;
  min-height: 240px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  line-height: 1.65;
  resize: none;
  tab-size: 2;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.editor:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.editor::placeholder {
  color: var(--xuya-text-tertiary);
}

.output {
  flex: 1;
  min-height: 240px;
  margin: 0;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  overflow: auto;
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  line-height: 1.65;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--xuya-text);
  cursor: pointer;
  transition: border-color var(--xuya-duration-fast);
}
.output:hover {
  border-color: var(--xuya-accent);
}
.output code {
  color: inherit;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: 8px;
}
.stat-card {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 10px 12px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border-light);
  border-radius: var(--xuya-radius-sm);
  text-align: center;
}
.stat-val {
  font-size: 18px;
  font-weight: 700;
  color: var(--xuya-accent);
  font-family: var(--xuya-font-mono);
}
.stat-lbl {
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
}
</style>
