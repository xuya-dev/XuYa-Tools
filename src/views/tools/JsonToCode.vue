<template>
  <ToolShell
    title="JSON 转类型"
    :icon="Braces"
    description="粘贴 JSON,一键生成 Go / Rust / TypeScript / Java / Python 类型定义。"
  >
    <!-- 语言选择 -->
    <div class="lang-bar">
      <div class="seg">
        <button
          v-for="l in langs"
          :key="l.id"
          :class="{ active: lang === l.id }"
          @click="lang = l.id"
        >
          {{ l.label }}
        </button>
      </div>
      <label class="opt">
        <input v-model="rootNameEditable" type="checkbox" />
        <span>根类型名</span>
        <input v-if="rootNameEditable" v-model="rootName" class="name-input" />
      </label>
    </div>

    <div class="grid">
      <!-- 输入 -->
      <div class="col">
        <div class="col-head">
          <span>JSON 输入</span>
          <button class="mini-btn" @click="sampleJson">示例</button>
        </div>
        <textarea
          v-model="input"
          class="editor"
          placeholder='粘贴 JSON,如:&#10;{&#10;  "name": "张三",&#10;  "age": 18,&#10;  "tags": ["a", "b"]&#10;}'
          spellcheck="false"
        ></textarea>
      </div>

      <!-- 输出 -->
      <div class="col">
        <div class="col-head">
          <span>{{ currentLangLabel }} 代码</span>
          <div class="col-actions">
            <button class="mini-btn" :disabled="!output" @click="doCopy">
              <Copy :size="12" />
              复制
            </button>
          </div>
        </div>
        <pre
          class="output"
          :class="{ err: !!errorMsg }"
        ><code>{{ errorMsg || output || '—' }}</code></pre>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Braces, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

type Lang = 'go' | 'rust' | 'typescript' | 'java' | 'python';
const langs: { id: Lang; label: string }[] = [
  { id: 'typescript', label: 'TypeScript' },
  { id: 'go', label: 'Go' },
  { id: 'rust', label: 'Rust' },
  { id: 'java', label: 'Java' },
  { id: 'python', label: 'Python' },
];

const lang = useToolState<Lang>('jsontocode', 'lang', 'typescript');
const input = useToolState('jsontocode', 'input', '');
const rootName = useToolState('jsontocode', 'rootName', 'Root');
const rootNameEditable = useToolState('jsontocode', 'rootNameEditable', false);
const output = ref('');
const errorMsg = ref('');

const currentLangLabel = computed(() => langs.find((l) => l.id === lang.value)?.label ?? '');

// ===== JSON 类型推断树 =====
type Node =
  | { kind: 'object'; fields: [string, Node][] }
  | { kind: 'array'; item: Node | null }
  | { kind: 'value'; ts: string }; // ts: 推断的 TS 类型

function inferType(v: unknown): Node {
  if (v === null) return { kind: 'value', ts: 'null' };
  if (Array.isArray(v)) {
    // 合并数组所有元素的类型,取并集;若为空数组,item=null
    if (v.length === 0) return { kind: 'array', item: null };
    const items = v.map(inferType);
    return { kind: 'array', item: mergeNodes(items) };
  }
  const t = typeof v;
  if (t === 'string') {
    // 数字串 / 布尔串不强猜,保持 string
    return { kind: 'value', ts: 'string' };
  }
  if (t === 'number')
    return { kind: 'value', ts: Number.isInteger(v as number) ? 'number' : 'number' };
  if (t === 'boolean') return { kind: 'value', ts: 'boolean' };
  if (t === 'object' && v) {
    const fields: [string, Node][] = Object.entries(v as Record<string, unknown>).map(
      ([k, val]) => [k, inferType(val)],
    );
    return { kind: 'object', fields };
  }
  return { kind: 'value', ts: 'any' };
}

/** 合并多个同位置节点(数组合并用),对象字段取并集,基本类型去重。 */
function mergeNodes(nodes: Node[]): Node {
  if (nodes.length === 0) return { kind: 'value', ts: 'any' };
  if (nodes.length === 1) return nodes[0]!;
  // 全是对象 → 合并字段
  if (nodes.every((n) => n.kind === 'object')) {
    const map = new Map<string, Node[]>();
    for (const n of nodes as Extract<Node, { kind: 'object' }>[]) {
      for (const [k, child] of n.fields) {
        if (!map.has(k)) map.set(k, []);
        map.get(k)!.push(child);
      }
    }
    const fields: [string, Node][] = [];
    for (const [k, arr] of map) {
      // 字段在某元素缺失 → 可选;同名字段合并
      const present = arr.length;
      const total = nodes.length;
      const merged = mergeNodes(arr);
      if (present < total) {
        fields.push([k, { kind: 'value', ts: optionalTs(merged) }]);
      } else {
        fields.push([k, merged]);
      }
    }
    return { kind: 'object', fields };
  }
  // 全是数组 → item 合并
  if (nodes.every((n) => n.kind === 'array')) {
    const arrs = nodes as Extract<Node, { kind: 'array' }>[];
    const items = arrs.map((a) => a.item).filter((x): x is Node => x !== null);
    return { kind: 'array', item: items.length ? mergeNodes(items) : null };
  }
  // 混合/值 → 取并集
  const types = new Set<string>();
  for (const n of nodes) types.add(nodeToTs(n));
  return { kind: 'value', ts: Array.from(types).join(' | ') };
}

function optionalTs(n: Node): string {
  const ts = nodeToTs(n);
  return ts.endsWith(' | null') || ts.includes('undefined') ? ts : `${ts} | null`;
}

function nodeToTs(n: Node): string {
  switch (n.kind) {
    case 'value':
      return n.ts;
    case 'array':
      return n.item ? `${nodeToTs(n.item)}[]` : 'unknown[]';
    case 'object':
      return 'object';
  }
}

// ===== 代码生成 =====
function genGo(root: Node, rootName: string): string {
  const structs: string[] = [];
  const used = new Set<string>();
  function emit(name: string, n: Node) {
    if (n.kind !== 'object') return;
    if (used.has(name)) return;
    used.add(name);
    const lines = [`type ${name} struct {`];
    for (const [k, child] of n.fields) {
      lines.push(`    ${pascal(k)} ${goType(child, pascal(k))} \`json:"${k}"\``);
    }
    lines.push('}');
    structs.push(lines.join('\n'));
    // 递归子对象
    for (const [k, child] of n.fields) {
      if (child.kind === 'object') emit(pascal(k), child);
      if (child.kind === 'array' && child.item?.kind === 'object') emit(pascal(k), child.item);
    }
  }
  function goType(n: Node, fieldName: string): string {
    switch (n.kind) {
      case 'value': {
        const ts = n.ts;
        if (ts.includes('number')) return 'float64';
        if (ts.includes('boolean')) return 'bool';
        if (ts.includes('string')) return 'string';
        return 'interface{}';
      }
      case 'array':
        return n.item ? `[]${goType(n.item, fieldName)}` : '[]interface{}';
      case 'object':
        return fieldName;
    }
    // unreachable
  }
  emit(rootName, root);
  return structs.join('\n\n') + '\n';
}

function genTypescript(root: Node, rootName: string): string {
  const blocks: string[] = [];
  const used = new Set<string>();
  function emit(name: string, n: Node) {
    if (n.kind !== 'object') return;
    if (used.has(name)) return;
    used.add(name);
    const lines = [`export interface ${name} {`];
    for (const [k, child] of n.fields) {
      const opt = isOptional(child);
      lines.push(`  ${opt ? `${k}?` : k}: ${tsType(child, name + pascal(k))};`);
    }
    lines.push('}');
    blocks.push(lines.join('\n'));
    for (const [k, child] of n.fields) {
      const subName = name + pascal(k);
      if (child.kind === 'object') emit(subName, child);
      if (child.kind === 'array' && child.item?.kind === 'object') emit(subName, child.item);
    }
  }
  function tsType(n: Node, objName: string): string {
    switch (n.kind) {
      case 'value':
        return n.ts === 'null' ? 'null' : n.ts;
      case 'array':
        return n.item ? `${tsType(n.item, objName)}[]` : 'unknown[]';
      case 'object':
        return objName;
    }
  }
  emit(rootName, root);
  return blocks.join('\n\n') + '\n';
}

function genRust(root: Node, rootName: string): string {
  const blocks: string[] = [];
  const used = new Set<string>();
  function emit(name: string, n: Node) {
    if (n.kind !== 'object') return;
    if (used.has(name)) return;
    used.add(name);
    const lines = [
      `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]`,
      `pub struct ${name} {`,
    ];
    for (const [k, child] of n.fields) {
      const opt = isOptional(child);
      const ty = rustType(child, name + pascal(k));
      lines.push(`    pub ${k}: ${opt ? `Option<${ty}>` : ty},`);
    }
    lines.push('}');
    blocks.push(lines.join('\n'));
    for (const [k, child] of n.fields) {
      const subName = name + pascal(k);
      if (child.kind === 'object') emit(subName, child);
      if (child.kind === 'array' && child.item?.kind === 'object') emit(subName, child.item);
    }
  }
  function rustType(n: Node, objName: string): string {
    switch (n.kind) {
      case 'value': {
        const ts = n.ts;
        if (ts.includes('number')) return 'f64';
        if (ts.includes('boolean')) return 'bool';
        if (ts.includes('string')) return 'String';
        return 'serde_json::Value';
      }
      case 'array':
        return n.item ? `Vec<${rustType(n.item, objName)}>` : 'Vec<serde_json::Value>';
      case 'object':
        return objName;
    }
  }
  emit(rootName, root);
  return blocks.join('\n\n') + '\n';
}

function genJava(root: Node, rootName: string): string {
  const blocks: string[] = [];
  const used = new Set<string>();
  function emit(name: string, n: Node) {
    if (n.kind !== 'object') return;
    if (used.has(name)) return;
    used.add(name);
    const lines = [`public class ${name} {`];
    for (const [k, child] of n.fields) {
      lines.push(`    private ${javaType(child, name + pascal(k))} ${k};`);
    }
    lines.push('}');
    blocks.push(lines.join('\n'));
    for (const [k, child] of n.fields) {
      const subName = name + pascal(k);
      if (child.kind === 'object') emit(subName, child);
      if (child.kind === 'array' && child.item?.kind === 'object') emit(subName, child.item);
    }
  }
  function javaType(n: Node, objName: string): string {
    switch (n.kind) {
      case 'value': {
        const ts = n.ts;
        if (ts.includes('number')) return 'Double';
        if (ts.includes('boolean')) return 'Boolean';
        if (ts.includes('string')) return 'String';
        return 'Object';
      }
      case 'array':
        return n.item ? `List<${javaType(n.item, objName)}>` : 'List<Object>';
      case 'object':
        return objName;
    }
  }
  emit(rootName, root);
  return `import java.util.List;\n\n` + blocks.join('\n\n') + '\n';
}

function genPython(root: Node, rootName: string): string {
  const blocks: string[] = [];
  const used = new Set<string>();
  function emit(name: string, n: Node) {
    if (n.kind !== 'object') return;
    if (used.has(name)) return;
    used.add(name);
    const lines = [`class ${name}:`, '    """自动生成,pydantic 风格。"""'];
    if (n.fields.length === 0) lines.push('    pass');
    else {
      for (const [k, child] of n.fields) {
        lines.push(`    ${k}: ${pyType(child, name + pascal(k))}`);
      }
    }
    blocks.push(lines.join('\n'));
    for (const [k, child] of n.fields) {
      const subName = name + pascal(k);
      if (child.kind === 'object') emit(subName, child);
      if (child.kind === 'array' && child.item?.kind === 'object') emit(subName, child.item);
    }
  }
  function pyType(n: Node, objName: string): string {
    switch (n.kind) {
      case 'value': {
        const ts = n.ts;
        if (ts.includes('number')) return 'float';
        if (ts.includes('boolean')) return 'bool';
        if (ts.includes('string')) return 'str';
        return 'Any';
      }
      case 'array':
        return n.item ? `list[${pyType(n.item, objName)}]` : 'list';
      case 'object':
        return `'${objName}'`;
    }
  }
  emit(rootName, root);
  return 'from typing import Any\n\n' + blocks.join('\n\n\n') + '\n';
}

// ===== 辅助 =====
function isOptional(n: Node): boolean {
  if (n.kind === 'value') return n.ts.includes('null') || n.ts.includes('undefined');
  return false;
}
function pascal(s: string): string {
  return (
    s
      .replace(/[^a-zA-Z0-9]+/g, '_')
      .split('_')
      .filter(Boolean)
      .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
      .join('') || 'Field'
  );
}

function run() {
  errorMsg.value = '';
  output.value = '';
  const text = input.value.trim();
  if (!text) return;
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (e) {
    errorMsg.value = `❌ JSON 解析失败:${e instanceof Error ? e.message : String(e)}`;
    return;
  }
  // 根是数组 → 取首个元素作为根对象推断
  let rootVal = parsed;
  let rootLabel = rootName.value || 'Root';
  if (Array.isArray(parsed)) {
    rootVal = parsed.length ? mergeNodes(parsed.map(inferType)) : { kind: 'object', fields: [] };
    rootLabel = rootName.value || 'RootItem';
  }
  const root = inferType(rootVal);
  try {
    switch (lang.value) {
      case 'go':
        output.value = genGo(root, rootLabel);
        break;
      case 'typescript':
        output.value = genTypescript(root, rootLabel);
        break;
      case 'rust':
        output.value = genRust(root, rootLabel);
        break;
      case 'java':
        output.value = genJava(root, rootLabel);
        break;
      case 'python':
        output.value = genPython(root, rootLabel);
        break;
    }
  } catch (e) {
    errorMsg.value = `❌ 生成失败:${e instanceof Error ? e.message : String(e)}`;
  }
}

function sampleJson() {
  input.value = JSON.stringify(
    {
      id: 123,
      name: '张三',
      active: true,
      score: 95.5,
      tags: ['a', 'b'],
      address: { city: '北京', zip: '100000' },
      friends: [{ id: 2, name: '李四' }],
    },
    null,
    2,
  );
}

async function doCopy() {
  if (output.value) await copyToClipboard(output.value, '已复制');
}

watch([input, lang, rootName, rootNameEditable], run, { immediate: true });
</script>

<style scoped>
.lang-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
.seg {
  display: inline-flex;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  padding: 2px;
  gap: 2px;
}
.seg button {
  padding: 5px 14px;
  font-size: 12px;
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
.opt {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
}
.opt input[type='checkbox'] {
  accent-color: var(--xuya-accent);
}
.name-input {
  width: 100px;
  padding: 4px 8px;
  font-size: 12px;
  font-family: var(--xuya-font-mono);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text);
  outline: none;
}
.name-input:focus {
  border-color: var(--xuya-accent);
}

.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
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
  gap: 6px;
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
.mini-btn:hover:not(:disabled) {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}
.mini-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.editor {
  flex: 1;
  min-height: 320px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.6;
  resize: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.editor:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.output {
  flex: 1;
  min-height: 320px;
  margin: 0;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  overflow: auto;
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.6;
  white-space: pre;
  color: var(--xuya-text);
}
.output.err code {
  color: var(--xuya-danger);
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
