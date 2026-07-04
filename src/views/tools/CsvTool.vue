<template>
  <ToolShell
    title="CSV 表格"
    :icon="Table"
    description="CSV 可视化预览编辑,支持排序 / 筛选,一键转 JSON / Markdown / TSV,兼容 RFC 4180。"
  >
    <!-- 选项栏 -->
    <div class="bar">
      <label class="opt">
        分隔符
        <select v-model="delimiter" class="sel">
          <option value=",">逗号 ,</option>
          <option value=";">分号 ;</option>
          <option value="\t">Tab</option>
          <option value="|">竖线 |</option>
        </select>
      </label>
      <label class="opt">
        <input v-model="hasHeader" type="checkbox" />
        首行为表头
      </label>
      <label class="opt">
        <input v-model="trimCell" type="checkbox" />
        去单元格首尾空格
      </label>
      <div class="bar-right">
        <span v-if="parseError" class="err-inline">{{ parseError }}</span>
        <span v-else-if="rows.length" class="stat">{{ rowCount }} 行 × {{ colCount }} 列</span>
      </div>
    </div>

    <div class="grid">
      <!-- 左:CSV 输入 -->
      <div class="col">
        <div class="col-head">
          <span>CSV 输入</span>
          <div class="col-actions">
            <button class="mini-btn" @click="loadSample">示例</button>
            <button class="mini-btn" @click="loadFile">导入文件</button>
            <input
              ref="fileInput"
              type="file"
              accept=".csv,.txt"
              style="display: none"
              @change="onFilePicked"
            />
          </div>
        </div>
        <textarea
          v-model="input"
          class="editor"
          placeholder="粘贴 CSV,如:&#10;name,age,city&#10;张三,18,北京&#10;李四,25,上海"
          spellcheck="false"
        ></textarea>
      </div>

      <!-- 右:表格预览 -->
      <div class="col">
        <div class="col-head">
          <span>表格预览</span>
          <div class="col-actions">
            <input v-model="filterText" class="filter-input" placeholder="筛选…" />
          </div>
        </div>

        <div v-if="filteredRows.length" class="table-wrap">
          <table class="csv-table">
            <thead>
              <tr>
                <th class="row-idx">#</th>
                <th
                  v-for="(h, i) in headers"
                  :key="i"
                  class="th-sortable"
                  :title="`按 ${h} 排序`"
                  @click="toggleSort(i)"
                >
                  <span class="th-text">{{ h || `列${i + 1}` }}</span>
                  <span class="sort-mark">{{ sortMark(i) }}</span>
                </th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, ri) in filteredRows" :key="ri">
                <td class="row-idx">{{ ri + 1 }}</td>
                <td v-for="(cell, ci) in row" :key="ci">{{ cell }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div v-else class="empty">
          <Table :size="36" />
          <p>{{ parseError ? '' : '输入 CSV 后在此预览' }}</p>
        </div>
      </div>
    </div>

    <!-- 转换输出 -->
    <div v-if="rows.length" class="export-bar">
      <div class="seg">
        <button :class="{ active: exportFmt === 'json' }" @click="exportFmt = 'json'">JSON</button>
        <button :class="{ active: exportFmt === 'markdown' }" @click="exportFmt = 'markdown'">
          Markdown
        </button>
        <button :class="{ active: exportFmt === 'tsv' }" @click="exportFmt = 'tsv'">TSV</button>
        <button :class="{ active: exportFmt === 'sql' }" @click="exportFmt = 'sql'">
          SQL INSERT
        </button>
      </div>
      <div class="export-out">
        <pre class="export-code">{{ exportText }}</pre>
        <button class="mini-btn copy-fixed" @click="copyExport">
          <Copy :size="12" />
          复制
        </button>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Table, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const input = useToolState('csvtool', 'input', '');
const delimiter = useToolState<string>('csvtool', 'delimiter', ',');
const hasHeader = useToolState('csvtool', 'hasHeader', true);
const trimCell = useToolState('csvtool', 'trimCell', false);
const filterText = useToolState('csvtool', 'filter', '');
const exportFmt = useToolState<'json' | 'markdown' | 'tsv' | 'sql'>('csvtool', 'exportFmt', 'json');

const fileInput = ref<HTMLInputElement | null>(null);
const rows = ref<string[][]>([]);
const headers = ref<string[]>([]);
const parseError = ref('');
const sortCol = ref(-1);
const sortDir = ref<'asc' | 'desc'>('asc');

const actualDelimiter = computed(() => (delimiter.value === '\\t' ? '\t' : delimiter.value));

const rowCount = computed(() => rows.value.length);
const colCount = computed(() => headers.value.length || (rows.value[0]?.length ?? 0));

// ===== RFC 4180 CSV 解析 =====
function parseCsv(text: string, delim: string): string[][] {
  const out: string[][] = [];
  let row: string[] = [];
  let cell = '';
  let inQuotes = false;
  let i = 0;
  while (i < text.length) {
    const ch = text[i];
    if (inQuotes) {
      if (ch === '"') {
        // 双引号转义
        if (text[i + 1] === '"') {
          cell += '"';
          i += 2;
          continue;
        }
        inQuotes = false;
        i++;
        continue;
      }
      cell += ch;
      i++;
      continue;
    }
    // 非引号状态
    if (ch === '"') {
      inQuotes = true;
      i++;
      continue;
    }
    if (ch === delim) {
      row.push(maybeTrim(cell));
      cell = '';
      i++;
      continue;
    }
    if (ch === '\r') {
      // \r\n 或 \r 换行
      row.push(maybeTrim(cell));
      out.push(row);
      row = [];
      cell = '';
      i++;
      if (text[i] === '\n') i++;
      continue;
    }
    if (ch === '\n') {
      row.push(maybeTrim(cell));
      out.push(row);
      row = [];
      cell = '';
      i++;
      continue;
    }
    cell += ch;
    i++;
  }
  // 末尾残留
  if (cell.length > 0 || row.length > 0) {
    row.push(maybeTrim(cell));
    out.push(row);
  }
  // 过滤完全空行
  return out.filter((r) => r.length > 1 || (r.length === 1 && r[0] !== ''));
}

function maybeTrim(s: string): string {
  return trimCell.value ? s.trim() : s;
}

function run() {
  parseError.value = '';
  rows.value = [];
  headers.value = [];
  sortCol.value = -1;
  const text = input.value;
  if (!text.trim()) return;
  try {
    const parsed = parseCsv(text, actualDelimiter.value);
    if (parsed.length === 0) return;
    if (hasHeader.value) {
      headers.value = parsed[0] ?? [];
      rows.value = parsed.slice(1);
    } else {
      const maxCols = Math.max(...parsed.map((r) => r.length));
      headers.value = Array.from({ length: maxCols }, (_, i) => `列${i + 1}`);
      rows.value = parsed;
    }
  } catch (e) {
    parseError.value = e instanceof Error ? e.message : String(e);
  }
}

// ===== 排序 + 筛选 =====
function toggleSort(col: number) {
  if (sortCol.value === col) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc';
  } else {
    sortCol.value = col;
    sortDir.value = 'asc';
  }
}
function sortMark(col: number): string {
  if (sortCol.value !== col) return '';
  return sortDir.value === 'asc' ? ' ▲' : ' ▼';
}

const filteredRows = computed(() => {
  let r = rows.value;
  // 筛选
  const kw = filterText.value.trim().toLowerCase();
  if (kw) {
    r = r.filter((row) => row.some((c) => c.toLowerCase().includes(kw)));
  }
  // 排序
  if (sortCol.value >= 0) {
    const col = sortCol.value;
    const dir = sortDir.value === 'asc' ? 1 : -1;
    r = [...r].sort((a, b) => {
      const av = a[col] ?? '';
      const bv = b[col] ?? '';
      // 数字优先数值比较
      const an = Number(av);
      const bn = Number(bv);
      if (!isNaN(an) && !isNaN(bn) && av !== '' && bv !== '') {
        return (an - bn) * dir;
      }
      return av.localeCompare(bv, 'zh') * dir;
    });
  }
  return r;
});

// ===== 转换输出 =====
const exportText = computed(() => {
  const data = filteredRows.value;
  const cols = headers.value;
  if (!data.length) return '';
  switch (exportFmt.value) {
    case 'json':
      return JSON.stringify(
        data.map((row) => {
          const obj: Record<string, string> = {};
          cols.forEach((h, i) => {
            obj[h || `col${i}`] = row[i] ?? '';
          });
          return obj;
        }),
        null,
        2,
      );
    case 'markdown': {
      const head = `| ${cols.map((h) => h || '').join(' | ')} |`;
      const sep = `| ${cols.map(() => '---').join(' | ')} |`;
      const body = data.map((row) => `| ${row.map((c) => c ?? '').join(' | ')} |`).join('\n');
      return `${head}\n${sep}\n${body}`;
    }
    case 'tsv':
      return data.map((row) => row.join('\t')).join('\n');
    case 'sql': {
      const table = 'data';
      const colsQuoted = cols.map((c) => `\`${c || 'col'}\``).join(', ');
      const vals = data
        .map(
          (row) =>
            `(${row
              .map((c) => (c === '' ? 'NULL' : `'${String(c).replace(/'/g, "''")}'`))
              .join(', ')})`,
        )
        .join(',\n  ');
      return `INSERT INTO \`${table}\` (${colsQuoted}) VALUES\n  ${vals};`;
    }
  }
  return '';
});

// ===== 操作 =====
function loadSample() {
  input.value = `name,age,city,join_date
张三,28,北京,2023-01-15
李四,35,上海,2022-11-08
王五,22,广州,2024-03-20
赵六,30,北京,2023-07-01
孙七,28,深圳,2024-01-10`;
}

function loadFile() {
  fileInput.value?.click();
}

function onFilePicked(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => {
    input.value = String(reader.result ?? '');
  };
  reader.readAsText(file, 'utf-8');
  // 重置以便重复选同一文件
  (e.target as HTMLInputElement).value = '';
}

async function copyExport() {
  if (exportText.value) await copyToClipboard(exportText.value, '已复制');
}

watch([input, delimiter, hasHeader, trimCell], run, { immediate: true });
</script>

<style scoped>
.bar {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
.opt {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  cursor: pointer;
}
.opt input[type='checkbox'] {
  accent-color: var(--xuya-accent);
}
.sel {
  padding: 4px 8px;
  font-size: 12px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text);
  outline: none;
}
.bar-right {
  margin-left: auto;
}
.stat {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
}
.err-inline {
  font-size: 11px;
  color: var(--xuya-danger);
}

.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
  margin-bottom: 16px;
}
.col {
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
.col-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}
.filter-input {
  width: 120px;
  padding: 3px 8px;
  font-size: 11px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text);
  outline: none;
}
.filter-input:focus {
  border-color: var(--xuya-accent);
}
.mini-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
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

.editor {
  width: 100%;
  min-height: 280px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.6;
  resize: vertical;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.editor:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

.table-wrap {
  flex: 1;
  min-height: 280px;
  overflow: auto;
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.csv-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}
.csv-table thead {
  position: sticky;
  top: 0;
  z-index: 1;
}
.th-sortable {
  padding: 7px 10px;
  text-align: left;
  font-weight: 600;
  color: var(--xuya-text);
  background: var(--xuya-card-bg);
  border-bottom: 1px solid var(--xuya-border);
  cursor: pointer;
  white-space: nowrap;
  user-select: none;
  transition: background var(--xuya-duration-fast);
}
.th-sortable:hover {
  background: var(--xuya-accent-soft);
}
.sort-mark {
  color: var(--xuya-accent);
  font-size: 10px;
}
.csv-table td {
  padding: 6px 10px;
  border-bottom: 1px solid var(--xuya-border);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
}
.csv-table tbody tr:hover {
  background: var(--xuya-accent-soft);
}
.row-idx {
  width: 40px;
  text-align: center;
  color: var(--xuya-text-tertiary) !important;
  font-size: 11px !important;
  background: var(--xuya-card-bg);
}

.empty {
  flex: 1;
  min-height: 280px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--xuya-text-tertiary);
  border: 1px dashed var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.empty p {
  font-size: 13px;
}

.export-bar {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.seg {
  display: inline-flex;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  padding: 2px;
  gap: 2px;
  align-self: flex-start;
}
.seg button {
  padding: 5px 14px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.seg button:hover {
  color: var(--xuya-text);
}
.seg button.active {
  background: var(--xuya-bg-elevated);
  color: var(--xuya-accent);
  font-weight: 600;
}
.export-out {
  position: relative;
}
.export-code {
  margin: 0;
  padding: 12px;
  max-height: 200px;
  overflow: auto;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  line-height: 1.5;
  white-space: pre;
  color: var(--xuya-text);
}
.copy-fixed {
  position: absolute;
  top: 8px;
  right: 8px;
}
</style>
