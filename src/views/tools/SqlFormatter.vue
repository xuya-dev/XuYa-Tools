<template>
  <ToolShell
    title="SQL 格式化"
    :icon="Database"
    description="美化 / 压缩 SQL，支持多种方言、关键字大小写控制与语法高亮。"
  >
    <div class="sql-toolbar">
      <BaseButton variant="primary" @click="format">美化</BaseButton>
      <BaseButton @click="minify">压缩</BaseButton>

      <span class="sep"></span>

      <div class="opt-group">
        <span class="opt-label">方言</span>
        <select v-model="dialect" class="opt-select">
          <option v-for="d in dialectOptions" :key="d.value" :value="d.value">{{ d.label }}</option>
        </select>
      </div>

      <span class="sep"></span>

      <div class="opt-group">
        <span class="opt-label">关键字</span>
        <select v-model="keywordCase" class="opt-select">
          <option value="upper">大写</option>
          <option value="lower">小写</option>
          <option value="preserve">保持</option>
        </select>
      </div>

      <div class="opt-group">
        <span class="opt-label">缩进</span>
        <div class="seg">
          <button :class="{ active: tabWidth === 2 }" @click="tabWidth = 2">2</button>
          <button :class="{ active: tabWidth === 4 }" @click="tabWidth = 4">4</button>
        </div>
      </div>

      <span class="sep"></span>

      <label class="toggle" title="输出语法高亮，关闭后可编辑">
        <input v-model="highlight" type="checkbox" />
        高亮
      </label>
      <label class="toggle" title="粘贴时自动美化">
        <input v-model="autoFormat" type="checkbox" />
        自动
      </label>

      <span style="flex: 1"></span>

      <BaseButton variant="ghost" :disabled="!output && !errorMsg" @click="copyOutput">
        <Copy :size="13" />
        复制
      </BaseButton>
      <BaseButton variant="ghost" @click="clearAll">清空</BaseButton>
    </div>

    <div class="sql-grid">
      <div class="sql-col">
        <div class="col-head">
          <span>输入 SQL</span>
          <span class="stat">{{ input.length }} 字符 · {{ input.split('\n').length }} 行</span>
        </div>
        <textarea
          v-model="input"
          class="editor"
          placeholder="SELECT * FROM users WHERE id = 1"
          spellcheck="false"
          @keydown="onKeydown"
          @paste="onPaste"
        ></textarea>
      </div>
      <div class="sql-col">
        <div class="col-head">
          <span>输出</span>
          <span class="stat" :class="{ err: !!errorMsg }">
            {{
              errorMsg
                ? '错误'
                : output
                  ? `${output.length} 字符 · ${output.split('\n').length} 行`
                  : ''
            }}
          </span>
        </div>

        <pre v-if="errorMsg" class="output error-output"><code>{{ errorMsg }}</code></pre>

        <CodeView
          v-else
          v-model="output"
          language="sql"
          :editable="!highlight"
          placeholder="点击「美化」查看格式化结果"
        />
      </div>
    </div>

    <div class="status-bar">
      <span class="status-item">
        <span class="status-label">方言</span>
        <span class="status-val">{{ dialectLabel }}</span>
      </span>
      <span class="status-item">
        <span class="status-label">关键字</span>
        <span class="status-val">{{ keywordCaseLabel }}</span>
      </span>
      <span v-if="output" class="status-item">
        <span class="status-label">输出</span>
        <span class="status-val">{{ output.length }} 字符</span>
      </span>
      <span style="flex: 1"></span>
      <span v-if="errorMsg" class="invalid-badge">✗ 格式化失败</span>
      <span v-else-if="output" class="valid-badge">✓ 格式化完成</span>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Database, Copy } from '@lucide/vue';
import { format as formatSqlLib } from 'sql-formatter';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import CodeView from '@/components/ui/CodeView.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const input = useToolState('sql', 'input', '');
const output = ref('');
const errorMsg = ref('');

const dialect = useToolState<string>('sql', 'dialect', 'mysql');
const keywordCase = useToolState<'upper' | 'lower' | 'preserve'>('sql', 'keywordCase', 'upper');
const tabWidth = useToolState<number>('sql', 'tabWidth', 2);
const highlight = useToolState('sql', 'highlight', true);
const autoFormat = useToolState('sql', 'autoFormat', false);

let fmtTimer: ReturnType<typeof setTimeout> | null = null;
watch(
  () => input.value,
  (val) => {
    if (!autoFormat.value || !val.trim()) return;
    if (fmtTimer) clearTimeout(fmtTimer);
    fmtTimer = setTimeout(() => format(), 300);
  },
);
watch(
  () => [dialect.value, keywordCase.value, tabWidth.value] as const,
  () => {
    if (!autoFormat.value || !input.value.trim()) return;
    format();
  },
);

const dialectOptions = [
  { value: 'mysql', label: 'MySQL' },
  { value: 'postgresql', label: 'PostgreSQL' },
  { value: 'sqlite', label: 'SQLite' },
  { value: 'transactsql', label: 'SQL Server' },
  { value: 'plsql', label: 'Oracle' },
  { value: 'sql', label: '标准 SQL' },
  { value: 'mariadb', label: 'MariaDB' },
  { value: 'bigquery', label: 'BigQuery' },
  { value: 'snowflake', label: 'Snowflake' },
  { value: 'redshift', label: 'Redshift' },
];

const dialectLabel = computed(
  () => dialectOptions.find((d) => d.value === dialect.value)?.label ?? dialect.value,
);
const keywordCaseLabel = computed(
  () => ({ upper: '大写', lower: '小写', preserve: '保持' })[keywordCase.value],
);

function format() {
  errorMsg.value = '';
  if (!input.value.trim()) return;
  try {
    output.value = formatSqlLib(input.value, {
      language: dialect.value as never,
      keywordCase: keywordCase.value,
      tabWidth: tabWidth.value,
    });
  } catch (e) {
    errorMsg.value = '格式化失败: ' + (e instanceof Error ? e.message : String(e));
  }
}

function minify() {
  errorMsg.value = '';
  if (!input.value.trim()) return;
  output.value = input.value.replace(/\s+/g, ' ').trim();
}

function onKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
    e.preventDefault();
    format();
  }
}

function onPaste(e: ClipboardEvent) {
  if (!autoFormat.value) return;
  const text = e.clipboardData?.getData('text/plain') ?? '';
  if (!text.trim()) return;
  setTimeout(() => format(), 0);
}

async function copyOutput() {
  const text = errorMsg.value || output.value;
  if (!text) return;
  await copyToClipboard(text, '已复制 SQL');
}

function clearAll() {
  input.value = '';
  output.value = '';
  errorMsg.value = '';
}
</script>

<style scoped>
.sql-toolbar {
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
.opt-group {
  display: flex;
  align-items: center;
  gap: 6px;
}
.opt-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.opt-select {
  padding: 5px 10px;
  font-size: 12px;
  color: var(--xuya-text);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  outline: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.opt-select:hover {
  border-color: var(--xuya-border-strong);
}
.opt-select:focus {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
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
}
.toggle input {
  accent-color: var(--xuya-accent);
  width: 14px;
  height: 14px;
  cursor: pointer;
}

.sql-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
}
.sql-col {
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
.stat.err {
  color: var(--xuya-danger);
  opacity: 1;
}

.editor {
  flex: 1;
  min-height: 260px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.65;
  resize: none;
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

.error-output {
  flex: 1;
  min-height: 260px;
  margin: 0;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-danger);
  background: var(--xuya-input-bg);
  overflow: auto;
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.65;
  white-space: pre-wrap;
  word-break: break-all;
}
.error-output code {
  color: var(--xuya-danger);
}

.status-bar {
  display: flex;
  align-items: center;
  gap: 18px;
  margin-top: 12px;
  padding: 8px 14px;
  border-radius: var(--xuya-radius);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border-light);
  flex-wrap: wrap;
}
.status-item {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 11.5px;
}
.status-label {
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.3px;
  font-size: 10px;
}
.status-val {
  color: var(--xuya-text);
  font-weight: 600;
  font-family: var(--xuya-font-mono);
}
.valid-badge {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--xuya-success);
}
.invalid-badge {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--xuya-danger);
}
</style>
