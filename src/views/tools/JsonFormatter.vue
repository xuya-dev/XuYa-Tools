<template>
  <ToolShell
    title="JSON 格式化"
    :icon="Braces"
    description="美化 / 压缩 / 转义 / 树形浏览 / 双 JSON 对比，支持语法高亮、缩进控制、键排序与 Unicode 转义。"
  >
    <div class="json-toolbar">
      <div class="tb-group">
        <BaseButton variant="primary" @click="format">美化</BaseButton>
        <BaseButton @click="minify">压缩</BaseButton>
        <BaseButton @click="validate">验证</BaseButton>
        <span class="sep"></span>
        <BaseButton @click="escape">转义</BaseButton>
        <BaseButton @click="unescape">反转义</BaseButton>
      </div>

      <div class="tb-group">
        <span class="tb-label">缩进</span>
        <div class="seg">
          <button
            v-for="opt in indentOptions"
            :key="opt.value"
            :class="{ active: indentSize === opt.value }"
            @click="indentSize = opt.value"
          >
            {{ opt.label }}
          </button>
        </div>
      </div>

      <span class="sep"></span>

      <div class="seg">
        <button :class="{ active: viewMode === 'text' }" @click="viewMode = 'text'">文本</button>
        <button :class="{ active: viewMode === 'tree' }" @click="viewMode = 'tree'">树形</button>
        <button :class="{ active: viewMode === 'diff' }" @click="viewMode = 'diff'">对比</button>
      </div>

      <span class="sep"></span>

      <label class="toggle" title="按键名字母排序">
        <input v-model="sortKeysOpt" type="checkbox" />
        排序
      </label>
      <label class="toggle" title="开启高亮（只读），关闭后输出框可直接编辑">
        <input v-model="syntaxHighlight" type="checkbox" />
        高亮
      </label>
      <label class="toggle" title="非 ASCII 字符转 \uXXXX">
        <input v-model="unicodeEscape" type="checkbox" />
        Unicode
      </label>
      <label class="toggle" title="粘贴时自动美化">
        <input v-model="autoFormat" type="checkbox" />
        自动
      </label>

      <span style="flex: 1"></span>

      <BaseButton variant="ghost" @click="triggerFileInput">
        <Upload :size="13" />
        导入
      </BaseButton>
      <BaseButton variant="ghost" :disabled="!output" @click="exportFile">
        <Download :size="13" />
        导出
      </BaseButton>
      <input
        ref="fileInput"
        type="file"
        accept=".json,.txt,application/json,text/plain"
        hidden
        @change="onFileSelected"
      />

      <span class="sep"></span>

      <BaseButton variant="ghost" :disabled="!output && !errorMsg" @click="copyOutput">
        <Copy :size="13" />
        复制
      </BaseButton>
      <BaseButton variant="ghost" @click="clearAll">清空</BaseButton>
    </div>

    <div class="editor-grid" :class="{ 'is-diff': viewMode === 'diff' }">
      <div class="editor-col">
        <div class="col-head">
          <span>{{ viewMode === 'diff' ? 'JSON A（原始）' : '输入' }}</span>
          <span class="head-right">
            <span v-if="parseBadge" class="parse-badge" :class="parseBadge.cls">
              {{ parseBadge.text }}
            </span>
            <span class="stat">{{ inputStats }}</span>
          </span>
        </div>
        <textarea
          v-model="input"
          class="editor"
          :placeholder="viewMode === 'diff' ? 'JSON A…' : '在此粘贴 JSON…'"
          spellcheck="false"
          @keydown="onKeydown"
          @paste="onPaste"
        ></textarea>
      </div>

      <div v-if="viewMode === 'diff'" class="editor-col">
        <div class="col-head">
          <span>JSON B（对比）</span>
          <span class="head-right">
            <span v-if="diffParseBadge" class="parse-badge" :class="diffParseBadge.cls">
              {{ diffParseBadge.text }}
            </span>
            <span class="stat">{{ diffInputStats }}</span>
          </span>
        </div>
        <textarea
          v-model="diffInput"
          class="editor"
          placeholder="JSON B…"
          spellcheck="false"
        ></textarea>
      </div>

      <div v-else class="editor-col">
        <div class="col-head">
          <span class="output-title">
            输出
            <template v-if="viewMode === 'tree' && flatNodes.length">
              <button class="mini-btn" @click="expandAll">全展开</button>
              <button class="mini-btn" @click="collapseAll">全折叠</button>
            </template>
          </span>
          <span class="stat" :class="{ err: !!errorMsg }">{{ outputStats }}</span>
        </div>

        <pre v-if="errorMsg" class="output error-output"><code>{{ errorMsg }}</code></pre>

        <CodeView
          v-else-if="viewMode === 'text'"
          v-model="output"
          :editable="!syntaxHighlight"
          placeholder="点击「美化」查看格式化结果"
        />

        <div v-else class="output tree-view">
          <div v-if="!flatNodes.length" class="tree-placeholder">{{ phTree }}</div>
          <div v-else class="tree-body">
            <div
              v-for="node in visibleNodes"
              :key="node.path"
              class="tree-row"
              :style="{ '--depth': node.depth }"
              :title="node.path === '$' ? '$ (根)' : node.path"
              @click="onNodeClick(node)"
            >
              <span
                v-if="node.hasChildren"
                class="toggle-arrow"
                :class="{ expanded: expandedPaths.has(node.path) }"
              >
                <ChevronRight :size="13" />
              </span>
              <span v-else class="toggle-spacer"></span>

              <span
                v-if="node.depth > 0"
                class="node-key"
                :class="{ 'is-index': /^\d+$/.test(node.displayKey) }"
              >
                {{ node.displayKey }}
                <span class="colon">:</span>
              </span>

              <span class="node-value" :class="'val-' + node.type">
                <template v-if="node.type === 'object'">
                  {{ node.childCount ? '{ ' + node.childCount + ' 项 }' : '{ }' }}
                </template>
                <template v-else-if="node.type === 'array'">
                  {{ node.childCount ? '[ ' + node.childCount + ' 项 ]' : '[ ]' }}
                </template>
                <template v-else>{{ node.value }}</template>
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="viewMode === 'diff'" class="diff-panel">
      <div class="diff-header">
        <span class="diff-title">差异结果</span>
        <div class="diff-stats">
          <span class="ds add">+{{ diffStats.added }} 新增</span>
          <span class="ds del">-{{ diffStats.removed }} 删除</span>
          <span class="ds chg">~{{ diffStats.changed }} 修改</span>
          <span class="ds total">{{ diffStats.total }} 处差异</span>
        </div>
        <span style="flex: 1"></span>
        <button class="mini-btn" @click="swapDiff">交换 A / B</button>
      </div>
      <div v-if="!bothValid" class="diff-empty">请在左右两侧各输入有效的 JSON</div>
      <div v-else-if="diffResults.length === 0" class="diff-empty diff-same">
        ✓ 两个 JSON 完全相同
      </div>
      <div v-else class="diff-list">
        <div
          v-for="(d, i) in diffResults"
          :key="i"
          class="diff-entry"
          :class="'diff-' + d.type"
          @click="copyDiffPath(d.path)"
        >
          <span class="diff-badge">{{ diffBadgeText(d.type) }}</span>
          <span class="diff-path">{{ diffDisplayPath(d.path) }}</span>
          <span class="diff-values">
            <template v-if="d.type === 'added'">
              <span class="val-new" :class="'vc-' + d.newType">{{ d.newValue }}</span>
            </template>
            <template v-else-if="d.type === 'removed'">
              <span class="val-old" :class="'vc-' + d.oldType">{{ d.oldValue }}</span>
            </template>
            <template v-else>
              <span class="val-old" :class="'vc-' + d.oldType">{{ d.oldValue }}</span>
              <span class="diff-arrow">→</span>
              <span class="val-new" :class="'vc-' + d.newType">{{ d.newValue }}</span>
            </template>
          </span>
        </div>
      </div>
    </div>

    <div v-if="viewMode !== 'diff'" class="status-bar">
      <template v-if="jsonStats">
        <span class="status-item">
          <span class="status-label">类型</span>
          <span class="status-val type-badge" :class="'badge-' + rootType">
            {{ rootTypeLabel }}
          </span>
        </span>
        <span class="status-item">
          <span class="status-label">深度</span>
          <span class="status-val">{{ jsonStats.depth }}</span>
        </span>
        <span class="status-item">
          <span class="status-label">对象</span>
          <span class="status-val">{{ jsonStats.objects }}</span>
        </span>
        <span class="status-item">
          <span class="status-label">数组</span>
          <span class="status-val">{{ jsonStats.arrays }}</span>
        </span>
        <span class="status-item">
          <span class="status-label">键</span>
          <span class="status-val">{{ jsonStats.keys }}</span>
        </span>
        <span class="status-item">
          <span class="status-label">叶子</span>
          <span class="status-val">{{ jsonStats.values }}</span>
        </span>
        <span class="status-item">
          <span class="status-label">大小</span>
          <span class="status-val">{{ dataSizeLabel }}</span>
        </span>
      </template>
      <span style="flex: 1"></span>
      <span v-if="isValid" class="valid-badge">✓ 有效 JSON</span>
      <span v-else-if="parseBadge?.cls === 'err'" class="invalid-badge">✗ JSON 解析失败</span>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Braces, Copy, ChevronRight, Upload, Download } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import CodeView from '@/components/ui/CodeView.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const toast = useToast();

const input = useToolState('json', 'input', '');
const diffInput = useToolState('json', 'diffInput', '');
const output = ref('');
const errorMsg = ref('');
const fileInput = ref<HTMLInputElement | null>(null);

const indentSize = useToolState<string>('json', 'indent', '2');
const viewMode = useToolState<'text' | 'tree' | 'diff'>('json', 'view', 'text');
const sortKeysOpt = useToolState('json', 'sortKeys', false);
const syntaxHighlight = useToolState('json', 'highlight', true);
const unicodeEscape = useToolState('json', 'unicodeEsc', false);
const autoFormat = useToolState('json', 'autoFormat', false);

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
  () => [indentSize.value, sortKeysOpt.value, unicodeEscape.value] as const,
  () => {
    if (!autoFormat.value || !input.value.trim()) return;
    format();
  },
);

const indentOptions = [
  { label: '2', value: '2' },
  { label: '4', value: '4' },
  { label: '8', value: '8' },
  { label: 'Tab', value: 'tab' },
];

type JsonType = 'object' | 'array' | 'string' | 'number' | 'boolean' | 'null';

const indentStr = computed<string | number>(() =>
  indentSize.value === 'tab' ? '\t' : parseInt(indentSize.value, 10),
);

const phTree = '输入有效 JSON 后自动显示树形结构';

const inputStats = computed(() => statsOf(input.value));
const diffInputStats = computed(() => statsOf(diffInput.value));
const outputStats = computed(() => {
  if (errorMsg.value) return '解析失败';
  if (!output.value && viewMode.value === 'tree') {
    return flatNodes.value.length ? flatNodes.value.length + ' 节点' : '';
  }
  return statsOf(output.value);
});

function statsOf(s: string) {
  if (!s) return '';
  const lines = s.split('\n').length;
  return `${s.length} 字符 · ${lines} 行`;
}

const parsedResult = computed<{ data: unknown; valid: boolean } | null>(() => {
  if (!input.value.trim()) return null;
  try {
    return { data: JSON.parse(input.value) as unknown, valid: true };
  } catch {
    return { data: undefined, valid: false };
  }
});

const diffParsedResult = computed<{ data: unknown; valid: boolean } | null>(() => {
  if (!diffInput.value.trim()) return null;
  try {
    return { data: JSON.parse(diffInput.value) as unknown, valid: true };
  } catch {
    return { data: undefined, valid: false };
  }
});

const isValid = computed(() => parsedResult.value?.valid ?? false);
const bothValid = computed(
  () => (parsedResult.value?.valid ?? false) && (diffParsedResult.value?.valid ?? false),
);

const parseBadge = computed<{ text: string; cls: string } | null>(() => {
  if (!parsedResult.value) return null;
  return parsedResult.value.valid ? { text: 'JSON', cls: 'ok' } : { text: '错误', cls: 'err' };
});

const diffParseBadge = computed<{ text: string; cls: string } | null>(() => {
  if (!diffParsedResult.value) return null;
  return diffParsedResult.value.valid ? { text: 'JSON', cls: 'ok' } : { text: '错误', cls: 'err' };
});

const rootType = computed<JsonType | null>(() => {
  if (!parsedResult.value?.valid) return null;
  return getJsonType(parsedResult.value.data);
});

const rootTypeLabel = computed(() => {
  if (!rootType.value) return '—';
  const map: Record<JsonType, string> = {
    object: 'Object',
    array: 'Array',
    string: 'String',
    number: 'Number',
    boolean: 'Boolean',
    null: 'Null',
  };
  return map[rootType.value];
});

const dataSizeLabel = computed(() => {
  const raw = output.value || input.value;
  if (!raw) return '0 B';
  return formatBytes(new Blob([raw]).size);
});

function formatBytes(n: number) {
  if (n < 1024) return n + ' B';
  if (n < 1024 * 1024) return (n / 1024).toFixed(1) + ' KB';
  return (n / (1024 * 1024)).toFixed(2) + ' MB';
}

interface JsonAnalysis {
  depth: number;
  objects: number;
  arrays: number;
  keys: number;
  values: number;
}

const jsonStats = computed<JsonAnalysis | null>(() => {
  if (!parsedResult.value?.valid) return null;
  return analyzeJson(parsedResult.value.data);
});

function analyzeJson(data: unknown): JsonAnalysis {
  const r = { depth: 0, objects: 0, arrays: 0, keys: 0, values: 0 };
  function walk(val: unknown, d: number) {
    r.depth = Math.max(r.depth, d);
    if (val === null || typeof val !== 'object') {
      r.values++;
      return;
    }
    if (Array.isArray(val)) {
      r.arrays++;
      for (const v of val) walk(v, d + 1);
    } else {
      r.objects++;
      const ks = Object.keys(val);
      r.keys += ks.length;
      for (const k of ks) walk((val as Record<string, unknown>)[k], d + 1);
    }
  }
  walk(data, 0);
  return r;
}

function getJsonType(val: unknown): JsonType {
  if (val === null) return 'null';
  if (Array.isArray(val)) return 'array';
  const t = typeof val;
  if (t === 'object') return 'object';
  if (t === 'string') return 'string';
  if (t === 'number') return 'number';
  if (t === 'boolean') return 'boolean';
  return 'null';
}

function sortKeysDeep(obj: unknown): unknown {
  if (obj === null || typeof obj !== 'object') return obj;
  if (Array.isArray(obj)) return obj.map(sortKeysDeep);
  const sorted: Record<string, unknown> = {};
  for (const key of Object.keys(obj as Record<string, unknown>).sort()) {
    sorted[key] = sortKeysDeep((obj as Record<string, unknown>)[key]);
  }
  return sorted;
}

function unicodeEscapeStr(s: string): string {
  return s.replace(
    /[\u007f-\uffff]/g,
    (c) => '\\u' + ('0000' + c.charCodeAt(0).toString(16)).slice(-4),
  );
}

interface TreeNode {
  path: string;
  parentPath: string;
  depth: number;
  displayKey: string;
  type: JsonType;
  value: string;
  childCount: number;
  hasChildren: boolean;
}

function buildFlatTree(data: unknown, sortKeys: boolean): TreeNode[] {
  const nodes: TreeNode[] = [];
  function walk(val: unknown, path: string, parentPath: string, depth: number, displayKey: string) {
    const type = getJsonType(val);
    if (type === 'object') {
      const obj = val as Record<string, unknown>;
      const keys = sortKeys ? Object.keys(obj).sort() : Object.keys(obj);
      nodes.push({
        path,
        parentPath,
        depth,
        displayKey,
        type,
        value: '',
        childCount: keys.length,
        hasChildren: keys.length > 0,
      });
      for (const k of keys) walk(obj[k], `${path}.${k}`, path, depth + 1, k);
    } else if (type === 'array') {
      const arr = val as unknown[];
      nodes.push({
        path,
        parentPath,
        depth,
        displayKey,
        type,
        value: '',
        childCount: arr.length,
        hasChildren: arr.length > 0,
      });
      arr.forEach((item, i) => walk(item, `${path}[${i}]`, path, depth + 1, String(i)));
    } else {
      let displayVal: string;
      if (type === 'string') displayVal = `"${val}"`;
      else if (type === 'null') displayVal = 'null';
      else displayVal = String(val);
      nodes.push({
        path,
        parentPath,
        depth,
        displayKey,
        type,
        value: displayVal,
        childCount: 0,
        hasChildren: false,
      });
    }
  }
  walk(data, '$', '', 0, '$');
  return nodes;
}

const flatNodes = computed<TreeNode[]>(() => {
  if (!parsedResult.value?.valid) return [];
  return buildFlatTree(parsedResult.value.data, sortKeysOpt.value);
});

const expandedPaths = ref<Set<string>>(new Set(['$']));

function toggleNode(path: string) {
  const next = new Set(expandedPaths.value);
  if (next.has(path)) next.delete(path);
  else next.add(path);
  expandedPaths.value = next;
}

function expandAll() {
  const all = new Set<string>();
  for (const n of flatNodes.value) {
    if (n.hasChildren) all.add(n.path);
  }
  expandedPaths.value = all;
}

function collapseAll() {
  expandedPaths.value = new Set(['$']);
}

const visibleNodes = computed<TreeNode[]>(() => {
  const result: TreeNode[] = [];
  const hiddenParents = new Set<string>();
  for (const node of flatNodes.value) {
    if (hiddenParents.has(node.parentPath)) {
      hiddenParents.add(node.path);
      continue;
    }
    result.push(node);
    if (node.hasChildren && !expandedPaths.value.has(node.path)) {
      hiddenParents.add(node.path);
    }
  }
  return result;
});

function onNodeClick(node: TreeNode) {
  if (node.hasChildren) {
    toggleNode(node.path);
  } else {
    let val = node.value;
    if (node.type === 'string') val = val.slice(1, -1);
    copyToClipboard(val, `已复制值: ${val.length > 40 ? val.slice(0, 40) + '…' : val}`);
  }
}

interface DiffEntry {
  path: string;
  type: 'added' | 'removed' | 'changed' | 'type-changed';
  oldValue: string;
  newValue: string;
  oldType: JsonType;
  newType: JsonType;
}

function formatDiffVal(val: unknown): string {
  if (val === undefined) return '';
  if (val === null) return 'null';
  if (typeof val === 'string') return `"${val}"`;
  if (typeof val === 'object') {
    const json = JSON.stringify(val);
    return json && json.length > 60 ? json.slice(0, 60) + '…' : json || '';
  }
  return String(val);
}

function deepDiff(a: unknown, b: unknown, path: string, results: DiffEntry[]): void {
  if (a === b) return;
  const ta = getJsonType(a);
  const tb = getJsonType(b);

  if (ta !== tb) {
    results.push({
      path,
      type: 'type-changed',
      oldValue: formatDiffVal(a),
      newValue: formatDiffVal(b),
      oldType: ta,
      newType: tb,
    });
    return;
  }

  if (ta === 'object') {
    const oa = a as Record<string, unknown>;
    const ob = b as Record<string, unknown>;
    const allKeys = new Set([...Object.keys(oa), ...Object.keys(ob)]);
    for (const key of allKeys) {
      const inA = key in oa;
      const inB = key in ob;
      const childPath = `${path}.${key}`;
      if (inA && !inB) {
        results.push({
          path: childPath,
          type: 'removed',
          oldValue: formatDiffVal(oa[key]),
          newValue: '',
          oldType: getJsonType(oa[key]),
          newType: 'null',
        });
      } else if (!inA && inB) {
        results.push({
          path: childPath,
          type: 'added',
          oldValue: '',
          newValue: formatDiffVal(ob[key]),
          oldType: 'null',
          newType: getJsonType(ob[key]),
        });
      } else {
        deepDiff(oa[key], ob[key], childPath, results);
      }
    }
    return;
  }

  if (ta === 'array') {
    const aa = a as unknown[];
    const ab = b as unknown[];
    const maxLen = Math.max(aa.length, ab.length);
    for (let i = 0; i < maxLen; i++) {
      const childPath = `${path}[${i}]`;
      if (i >= aa.length) {
        results.push({
          path: childPath,
          type: 'added',
          oldValue: '',
          newValue: formatDiffVal(ab[i]),
          oldType: 'null',
          newType: getJsonType(ab[i]),
        });
      } else if (i >= ab.length) {
        results.push({
          path: childPath,
          type: 'removed',
          oldValue: formatDiffVal(aa[i]),
          newValue: '',
          oldType: getJsonType(aa[i]),
          newType: 'null',
        });
      } else {
        deepDiff(aa[i], ab[i], childPath, results);
      }
    }
    return;
  }

  results.push({
    path,
    type: 'changed',
    oldValue: formatDiffVal(a),
    newValue: formatDiffVal(b),
    oldType: ta,
    newType: tb,
  });
}

const diffResults = computed<DiffEntry[]>(() => {
  if (!bothValid.value) return [];
  const results: DiffEntry[] = [];
  deepDiff(parsedResult.value!.data, diffParsedResult.value!.data, '$', results);
  return results;
});

const diffStats = computed(() => {
  let added = 0,
    removed = 0,
    changed = 0;
  for (const d of diffResults.value) {
    if (d.type === 'added') added++;
    else if (d.type === 'removed') removed++;
    else changed++;
  }
  return { added, removed, changed, total: diffResults.value.length };
});

function diffBadgeText(type: DiffEntry['type']): string {
  return { added: '+', removed: '-', changed: '~', 'type-changed': 'T' }[type];
}

function diffDisplayPath(path: string): string {
  if (path === '$') return '(根)';
  return path.replace(/^\$\./, '');
}

async function copyDiffPath(path: string) {
  await copyToClipboard(path, `已复制路径: ${path}`);
}

function swapDiff() {
  const t = input.value;
  input.value = diffInput.value;
  diffInput.value = t;
}

function format() {
  errorMsg.value = '';
  if (!input.value.trim()) return toast.error('请先输入 JSON');
  try {
    let parsed: unknown = JSON.parse(input.value);
    if (sortKeysOpt.value) parsed = sortKeysDeep(parsed);
    let result = JSON.stringify(parsed, null, indentStr.value);
    if (unicodeEscape.value) result = unicodeEscapeStr(result);
    output.value = result;
    toast.success('格式化成功');
  } catch (e) {
    errorMsg.value = formatError(e, input.value);
  }
}

function minify() {
  errorMsg.value = '';
  if (!input.value.trim()) return toast.error('请先输入 JSON');
  try {
    let parsed: unknown = JSON.parse(input.value);
    if (sortKeysOpt.value) parsed = sortKeysDeep(parsed);
    let result = JSON.stringify(parsed);
    if (unicodeEscape.value) result = unicodeEscapeStr(result);
    output.value = result;
    toast.success('压缩成功');
  } catch (e) {
    errorMsg.value = formatError(e, input.value);
  }
}

function validate() {
  errorMsg.value = '';
  if (!input.value.trim()) return toast.error('请先输入 JSON');
  try {
    const parsed = JSON.parse(input.value);
    toast.success(`有效 JSON — ${rootTypeLabel.value || getJsonType(parsed)}`);
  } catch (e) {
    errorMsg.value = formatError(e, input.value);
  }
}

function escape() {
  errorMsg.value = '';
  if (!input.value) return toast.error('请先输入内容');
  output.value = JSON.stringify(input.value);
  toast.success('转义成功');
}

function unescape() {
  errorMsg.value = '';
  if (!input.value.trim()) return toast.error('请先输入内容');
  try {
    const restored = JSON.parse(input.value);
    if (typeof restored === 'string') {
      output.value = restored;
      toast.success('反转义成功');
    } else {
      output.value = JSON.stringify(restored, null, 2);
      toast.success('已解析为 JSON');
    }
  } catch (e) {
    errorMsg.value = formatError(e, input.value);
  }
}

function formatError(e: unknown, src: string): string {
  const msg = String(e);
  const posMatch = msg.match(/position\s+(\d+)/i);
  if (posMatch) {
    const pos = parseInt(posMatch[1] ?? '0', 10);
    const before = src.slice(0, pos);
    const line = before.split('\n').length;
    const col = pos - before.lastIndexOf('\n');
    const snippet = src.slice(Math.max(0, pos - 20), Math.min(src.length, pos + 20));
    return `❌ 第 ${line} 行, 第 ${col} 列附近:\n…${snippet}…\n\n${msg}`;
  }
  return `❌ ${msg}`;
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
  await copyToClipboard(text);
}

function clearAll() {
  input.value = '';
  diffInput.value = '';
  output.value = '';
  errorMsg.value = '';
  expandedPaths.value = new Set(['$']);
}

function triggerFileInput() {
  fileInput.value?.click();
}

function onFileSelected(e: Event) {
  const target = e.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;
  if (file.size > 5 * 1024 * 1024) {
    toast.error('文件过大 (>5MB)');
    target.value = '';
    return;
  }
  const reader = new FileReader();
  reader.onload = () => {
    input.value = String(reader.result);
    output.value = '';
    errorMsg.value = '';
    if (autoFormat.value) format();
    toast.success(`已导入 ${file.name}`);
  };
  reader.onerror = () => toast.error('文件读取失败');
  reader.readAsText(file);
  target.value = '';
}

function exportFile() {
  const content = output.value || input.value;
  if (!content.trim()) return;
  const blob = new Blob([content], { type: 'application/json;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'formatted.json';
  a.click();
  URL.revokeObjectURL(url);
  toast.success('已导出 formatted.json');
}
</script>

<style>
@import url('https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;500;600&display=swap');
</style>

<style scoped>
.json-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 14px;
}
.tb-group {
  display: flex;
  align-items: center;
  gap: 8px;
}
.tb-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
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
  padding: 4px 12px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-radius: var(--xuya-radius-sm);
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
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

.mini-btn {
  padding: 1px 8px;
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
  background: transparent;
  border: 1px solid var(--xuya-border);
  border-radius: 4px;
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
  margin-left: 4px;
}
.mini-btn:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}

.editor-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
}
.editor-grid.is-diff {
  flex: 0 0 auto;
  margin-bottom: 14px;
}
.editor-grid.is-diff .editor {
  height: 200px;
  min-height: 200px;
}
.editor-col {
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
.head-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
.output-title {
  display: flex;
  align-items: center;
  gap: 2px;
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

.parse-badge {
  padding: 1px 8px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 600;
  line-height: 1.5;
}
.parse-badge.ok {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.parse-badge.err {
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
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
  letter-spacing: 0.1px;
  font-feature-settings:
    'calt' 1,
    'liga' 1;
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

.output {
  flex: 1;
  min-height: 260px;
  margin: 0;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-output-bg);
  overflow: auto;
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.65;
  letter-spacing: 0.1px;
  font-feature-settings:
    'calt' 1,
    'liga' 1;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--xuya-output-text);
  transition: border-color var(--xuya-duration-fast);
  tab-size: 2;
}
.output code {
  color: inherit;
}
.error-output {
  border-color: var(--xuya-danger);
}
.error-output code {
  color: var(--xuya-danger);
  white-space: pre-wrap;
}

.tree-view {
  display: flex;
  flex-direction: column;
  padding: 6px 8px;
}
.tree-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--xuya-text-tertiary);
  font-size: 13px;
}
.tree-body {
  overflow: auto;
  flex: 1;
}
.tree-row {
  display: flex;
  align-items: flex-start;
  gap: 4px;
  padding: 2px 6px;
  border-radius: 4px;
  cursor: pointer;
  line-height: 1.6;
  font-size: 12.5px;
  padding-left: calc(8px + var(--depth, 0) * 18px);
  transition: background var(--xuya-duration-fast);
}
.tree-row:hover {
  background: var(--xuya-accent-soft);
}
.toggle-arrow {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 18px;
  flex-shrink: 0;
  color: var(--xuya-text-tertiary);
  transition: transform var(--xuya-duration-fast) var(--xuya-ease);
  transform: rotate(0deg);
}
.toggle-arrow.expanded {
  transform: rotate(90deg);
}
.toggle-spacer {
  display: inline-block;
  width: 14px;
  flex-shrink: 0;
}
.node-key {
  color: var(--xuya-syn-key);
  font-weight: 500;
  flex-shrink: 0;
}
.node-key.is-index {
  color: var(--xuya-text-tertiary);
  font-weight: 400;
}
.colon {
  margin-right: 2px;
  color: var(--xuya-text-tertiary);
}
.node-value {
  word-break: break-all;
}
.val-object {
  color: var(--xuya-info);
  font-weight: 500;
}
.val-array {
  color: var(--xuya-purple);
  font-weight: 500;
}
.val-string {
  color: var(--xuya-syn-str);
}
.val-number {
  color: var(--xuya-syn-num);
}
.val-boolean {
  color: var(--xuya-syn-bool);
  font-weight: 500;
}
.val-null {
  color: var(--xuya-syn-null);
  font-style: italic;
}

.diff-panel {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-card-bg);
  overflow: hidden;
}
.diff-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--xuya-border-light);
  background: var(--xuya-bg-subtle);
  flex-shrink: 0;
}
.diff-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--xuya-text);
}
.diff-stats {
  display: flex;
  gap: 10px;
  font-size: 11.5px;
  font-weight: 600;
  font-family: var(--xuya-font-mono);
}
.ds.add {
  color: var(--xuya-success);
}
.ds.del {
  color: var(--xuya-danger);
}
.ds.chg {
  color: var(--xuya-warn);
}
.ds.total {
  color: var(--xuya-text-secondary);
  padding-left: 6px;
  border-left: 1px solid var(--xuya-border);
}

.diff-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--xuya-text-tertiary);
  font-size: 13px;
}
.diff-empty.diff-same {
  color: var(--xuya-success);
  font-weight: 600;
}

.diff-list {
  flex: 1;
  overflow: auto;
  padding: 4px 0;
}
.diff-entry {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 14px;
  font-size: 12px;
  line-height: 1.6;
  cursor: pointer;
  border-bottom: 1px solid var(--xuya-border-light);
  transition: background var(--xuya-duration-fast);
}
.diff-entry:hover {
  background: var(--xuya-accent-soft);
}
.diff-entry:last-child {
  border-bottom: none;
}

.diff-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 700;
  font-family: var(--xuya-font-mono);
}
.diff-added .diff-badge {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.diff-removed .diff-badge {
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
}
.diff-changed .diff-badge {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}
.diff-type-changed .diff-badge {
  background: var(--xuya-purple-soft);
  color: var(--xuya-purple);
}

.diff-path {
  flex-shrink: 0;
  min-width: 120px;
  max-width: 280px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--xuya-syn-key);
  font-family: var(--xuya-font-mono);
  font-weight: 500;
  font-size: 11.5px;
}
.diff-values {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  flex-wrap: wrap;
  font-family: var(--xuya-font-mono);
  font-size: 11.5px;
}
.val-old {
  color: var(--xuya-danger);
  text-decoration: line-through;
  opacity: 0.8;
  word-break: break-all;
}
.val-new {
  color: var(--xuya-success);
  word-break: break-all;
}
.vc-string {
  font-style: normal;
}
.vc-null {
  font-style: italic;
}
.vc-boolean,
.vc-number {
  font-weight: 600;
}
.diff-arrow {
  color: var(--xuya-text-tertiary);
  flex-shrink: 0;
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
.type-badge {
  padding: 1px 8px;
  border-radius: 10px;
  font-size: 10.5px;
}
.badge-object {
  background: var(--xuya-info-soft);
  color: var(--xuya-info);
}
.badge-array {
  background: var(--xuya-purple-soft);
  color: var(--xuya-purple);
}
.badge-string {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.badge-number {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}
.badge-boolean {
  background: var(--xuya-purple-soft);
  color: var(--xuya-purple);
}
.badge-null {
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
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
