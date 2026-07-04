<template>
  <ToolShell
    title="JSON 结构化对比"
    :icon="GitCompare"
    description="按字段路径结构化对比两个 JSON,识别新增 / 删除 / 修改,忽略数组顺序可选。"
  >
    <!-- 选项 -->
    <div class="opt-bar">
      <label class="opt">
        <input v-model="ignoreArrayOrder" type="checkbox" />
        <span>忽略数组顺序(按值排序后再比)</span>
      </label>
      <label class="opt">
        <input v-model="ignoreCase" type="checkbox" />
        <span>字符串忽略大小写</span>
      </label>
      <span v-if="diffs.length" class="opt-summary">
        共 {{ diffs.length }} 处差异 ·
        <span class="add">+{{ addCount }}</span>
        <span class="del">-{{ delCount }}</span>
        <span class="mod">~{{ modCount }}</span>
      </span>
    </div>

    <div class="grid">
      <!-- 左:JSON A -->
      <div class="col">
        <div class="col-head">
          <span>JSON A (基准)</span>
          <button class="mini-btn" @click="swap">⇅ 交换 A/B</button>
        </div>
        <textarea
          v-model="left"
          class="editor"
          placeholder='粘贴 JSON A,如:&#10;{ "name": "张三", "age": 18 }'
          spellcheck="false"
        ></textarea>
      </div>
      <!-- 右:JSON B -->
      <div class="col">
        <div class="col-head">
          <span>JSON B (对比)</span>
        </div>
        <textarea
          v-model="right"
          class="editor"
          placeholder='粘贴 JSON B,如:&#10;{ "name": "李四", "age": 18, "city": "北京" }'
          spellcheck="false"
        ></textarea>
      </div>
    </div>

    <!-- 错误 -->
    <div v-if="errorMsg" class="err-box">
      <AlertCircle :size="14" />
      {{ errorMsg }}
    </div>

    <!-- 差异列表 -->
    <div v-if="diffs.length && !errorMsg" class="diff-list">
      <div class="diff-head">
        <span>差异详情(按字段路径)</span>
      </div>
      <div v-for="(d, i) in diffs" :key="i" class="diff-row" :class="d.kind">
        <span class="diff-path">{{ d.path || '(根)' }}</span>
        <span class="diff-kind" :class="d.kind">{{ kindLabel(d.kind) }}</span>
        <span class="diff-val">
          <template v-if="d.kind === 'modify'">
            <code class="val-old">{{ formatVal(d.oldVal) }}</code>
            <span class="arrow">→</span>
            <code class="val-new">{{ formatVal(d.newVal) }}</code>
          </template>
          <template v-else-if="d.kind === 'add'">
            <code class="val-new">{{ formatVal(d.val) }}</code>
          </template>
          <template v-else>
            <code class="val-old">{{ formatVal(d.val) }}</code>
          </template>
        </span>
      </div>
    </div>

    <!-- 一致 -->
    <div v-else-if="parsed && !errorMsg" class="identical">
      <CircleCheck :size="16" />
      两个 JSON 完全一致
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { GitCompare, AlertCircle, CircleCheck } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { useToolState } from '@/composables/useToolState';

const left = useToolState('jsondiff', 'left', '');
const right = useToolState('jsondiff', 'right', '');
const ignoreArrayOrder = useToolState('jsondiff', 'ignoreArrayOrder', false);
const ignoreCase = useToolState('jsondiff', 'ignoreCase', false);

type DiffKind = 'add' | 'del' | 'modify';
interface Diff {
  path: string;
  kind: DiffKind;
  val?: unknown; // add/del 时
  oldVal?: unknown; // modify 时
  newVal?: unknown;
}

const diffs = ref<Diff[]>([]);
const errorMsg = ref('');
const parsed = ref(false);

const addCount = computed(() => diffs.value.filter((d) => d.kind === 'add').length);
const delCount = computed(() => diffs.value.filter((d) => d.kind === 'del').length);
const modCount = computed(() => diffs.value.filter((d) => d.kind === 'modify').length);

function kindLabel(k: DiffKind): string {
  return k === 'add' ? '新增' : k === 'del' ? '删除' : '修改';
}

function formatVal(v: unknown): string {
  if (v === undefined) return 'undefined';
  if (typeof v === 'object') return JSON.stringify(v);
  return String(v);
}

function swap() {
  const t = left.value;
  left.value = right.value;
  right.value = t;
}

// ===== 结构化对比核心 =====
function normalize(v: unknown): unknown {
  if (ignoreArrayOrder.value && Array.isArray(v)) {
    // 对数组元素递归 normalize 后,按序列化字符串排序
    const norm = v.map(normalize);
    return norm.sort((a, b) => {
      const sa = JSON.stringify(a);
      const sb = JSON.stringify(b);
      return sa < sb ? -1 : sa > sb ? 1 : 0;
    });
  }
  return v;
}

function valuesEqual(a: unknown, b: unknown): boolean {
  // 类型不同 → 不等
  if (typeof a !== typeof b) {
    // 但 null/undefined 宽松处理
    if ((a == null) !== (b == null)) return false;
    if (a == null && b == null) return true;
    return false;
  }
  if (typeof a === 'string' && typeof b === 'string' && ignoreCase.value) {
    return a.toLowerCase() === b.toLowerCase();
  }
  if (a === b) return true;
  // 深对象/数组:用 JSON 字符串比(已 normalize 过)
  if (typeof a === 'object' && typeof b === 'object') {
    return JSON.stringify(a) === JSON.stringify(b);
  }
  return false;
}

/** 对比两个值,把差异推入 out。path 是当前字段路径。 */
function compare(a: unknown, b: unknown, path: string, out: Diff[]) {
  // 都是数组
  if (Array.isArray(a) && Array.isArray(b)) {
    const an = normalize(a) as unknown[];
    const bn = normalize(b) as unknown[];
    const max = Math.max(an.length, bn.length);
    for (let i = 0; i < max; i++) {
      const p = `${path}[${i}]`;
      if (i >= an.length) {
        out.push({ path: p, kind: 'add', val: bn[i] });
      } else if (i >= bn.length) {
        out.push({ path: p, kind: 'del', val: an[i] });
      } else {
        compare(an[i], bn[i], p, out);
      }
    }
    return;
  }
  // 都是普通对象
  if (isObject(a) && isObject(b)) {
    const ao = a as Record<string, unknown>;
    const bo = b as Record<string, unknown>;
    const keys = new Set([...Object.keys(ao), ...Object.keys(bo)]);
    for (const k of keys) {
      const p = path ? `${path}.${k}` : k;
      if (!(k in ao)) {
        out.push({ path: p, kind: 'add', val: bo[k] });
      } else if (!(k in bo)) {
        out.push({ path: p, kind: 'del', val: ao[k] });
      } else {
        compare(ao[k], bo[k], p, out);
      }
    }
    return;
  }
  // 叶子值
  if (!valuesEqual(a, b)) {
    out.push({ path, kind: 'modify', oldVal: a, newVal: b });
  }
}

function isObject(v: unknown): v is Record<string, unknown> {
  return v !== null && typeof v === 'object' && !Array.isArray(v);
}

function run() {
  errorMsg.value = '';
  diffs.value = [];
  parsed.value = false;
  const la = left.value.trim();
  const rb = right.value.trim();
  if (!la || !rb) return;
  let a: unknown;
  let b: unknown;
  try {
    a = JSON.parse(la);
    b = JSON.parse(rb);
  } catch (e) {
    errorMsg.value = `JSON 解析失败:${e instanceof Error ? e.message : String(e)}`;
    return;
  }
  parsed.value = true;
  const an = normalize(a);
  const bn = normalize(b);
  const out: Diff[] = [];
  compare(an, bn, '', out);
  diffs.value = out;
}

watch([left, right, ignoreArrayOrder, ignoreCase], run, { immediate: true });
</script>

<style scoped>
.opt-bar {
  display: flex;
  align-items: center;
  gap: 18px;
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
.opt-summary {
  margin-left: auto;
  font-size: 12px;
  color: var(--xuya-text-tertiary);
}
.opt-summary .add {
  color: var(--xuya-success);
  margin-left: 8px;
}
.opt-summary .del {
  color: var(--xuya-danger);
  margin-left: 8px;
}
.opt-summary .mod {
  color: var(--xuya-warn);
  margin-left: 8px;
}

.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  margin-bottom: 16px;
}
.col {
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
  min-height: 180px;
  max-height: 260px;
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

.err-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
  border-radius: var(--xuya-radius);
  font-size: 13px;
}

.diff-list {
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  overflow: hidden;
}
.diff-head {
  padding: 10px 14px;
  background: var(--xuya-card-bg);
  border-bottom: 1px solid var(--xuya-border);
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.diff-row {
  display: grid;
  grid-template-columns: minmax(160px, 1fr) 60px 2fr;
  gap: 12px;
  align-items: center;
  padding: 8px 14px;
  border-bottom: 1px solid var(--xuya-border);
  font-size: 12px;
}
.diff-row:last-child {
  border-bottom: none;
}
.diff-row.add {
  background: rgba(16, 185, 129, 0.05);
}
.diff-row.del {
  background: rgba(239, 68, 68, 0.05);
}
.diff-row.modify {
  background: rgba(245, 158, 11, 0.05);
}
.diff-path {
  font-family: var(--xuya-font-mono);
  font-weight: 600;
  color: var(--xuya-text);
  word-break: break-all;
}
.diff-kind {
  font-size: 11px;
  font-weight: 600;
  text-align: center;
  padding: 2px 6px;
  border-radius: var(--xuya-radius-sm);
}
.diff-kind.add {
  color: var(--xuya-success);
  background: rgba(16, 185, 129, 0.12);
}
.diff-kind.del {
  color: var(--xuya-danger);
  background: rgba(239, 68, 68, 0.12);
}
.diff-kind.modify {
  color: var(--xuya-warn);
  background: rgba(245, 158, 11, 0.12);
}
.diff-val {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.diff-val code {
  font-family: var(--xuya-font-mono);
  font-size: 11.5px;
  padding: 2px 6px;
  border-radius: 3px;
  word-break: break-all;
}
.val-old {
  color: var(--xuya-danger);
  background: rgba(239, 68, 68, 0.08);
  text-decoration: line-through;
  opacity: 0.85;
}
.val-new {
  color: var(--xuya-success);
  background: rgba(16, 185, 129, 0.08);
}
.arrow {
  color: var(--xuya-text-tertiary);
  font-size: 11px;
}

.identical {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px;
  color: var(--xuya-success);
  background: rgba(16, 185, 129, 0.06);
  border-radius: var(--xuya-radius);
  font-size: 13px;
  font-weight: 600;
}
</style>
