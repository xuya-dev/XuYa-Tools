<template>
  <ToolShell
    title="格式互转"
    :icon="ArrowLeftRight"
    description="JSON ↔ YAML / CSV / XML / TOML / Properties 双向转换，实时预览。"
  >
    <div class="conv-toolbar">
      <div class="opt-group">
        <span class="opt-label">源格式</span>
        <select v-model="fromFormat" class="opt-select">
          <option v-for="f in FORMATS" :key="f" :value="f">{{ f.toUpperCase() }}</option>
        </select>
      </div>
      <button class="swap-btn" title="交换" @click="swapFormats">
        <ArrowLeftRight :size="14" />
      </button>
      <div class="opt-group">
        <span class="opt-label">目标</span>
        <select v-model="toFormat" class="opt-select">
          <option v-for="f in FORMATS" :key="f" :value="f">{{ f.toUpperCase() }}</option>
        </select>
      </div>
      <span class="dir-badge">{{ fromFormat.toUpperCase() }} → {{ toFormat.toUpperCase() }}</span>
      <span class="sep"></span>
      <BaseButton variant="ghost" :disabled="!input" @click="copyOutput">
        <Copy :size="13" />
        复制结果
      </BaseButton>
      <BaseButton
        v-if="toFormat === 'json'"
        variant="ghost"
        @click="jsonPretty = !jsonPretty"
      >
        {{ jsonPretty ? '压缩 JSON' : '美化 JSON' }}
      </BaseButton>
      <BaseButton variant="ghost" @click="input = ''">清空</BaseButton>
    </div>

    <div class="conv-grid">
      <div class="conv-col">
        <div class="col-head">
          <span>输入 ({{ fromFormat.toUpperCase() }})</span>
          <span class="stat">{{ input.length }} 字符</span>
        </div>
        <textarea
          v-model="input"
          class="editor"
          :placeholder="inputPlaceholder"
          spellcheck="false"
        ></textarea>
      </div>
      <div class="conv-col">
        <div class="col-head">
          <span>输出 ({{ toFormat.toUpperCase() }})</span>
          <span class="stat" :class="{ err: !!error }">
            {{ error ? '错误' : output ? output.length + ' 字符' : '' }}
          </span>
        </div>
        <pre v-if="error" class="output error-output"><code>{{ error }}</code></pre>
        <CodeView
          v-else
          v-model="output"
          :editable="false"
          :language="codeLang"
          placeholder="转换结果将显示在这里"
        />
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { ArrowLeftRight, Copy } from '@lucide/vue';
import { load as yamlLoad, dump as yamlDump } from 'js-yaml';
import { xml2js as _xml2js, js2xml } from 'xml-js';

const xml2js = (text: string, opts: object) => _xml2js(text, opts) as Record<string, unknown>;
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import CodeView from '@/components/ui/CodeView.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const FORMATS = ['json', 'yaml', 'csv', 'xml', 'toml', 'properties'] as const;
type Format = (typeof FORMATS)[number];

const fromFormat = useToolState<Format>('converter', 'from', 'json');
const toFormat = useToolState<Format>('converter', 'to', 'yaml');
const jsonPretty = useToolState('converter', 'jsonPretty', true);
const input = useToolState('converter', 'input', '');
const output = ref('');
const error = ref('');

const codeLang = computed(() => (toFormat.value === 'json' ? 'json' : ''));
const inputPlaceholder = computed(() => {
  const map: Record<Format, string> = {
    json: '{\n  "name": "XuYa",\n  "version": "1.0"\n}',
    yaml: 'name: XuYa\nversion: "1.0"',
    csv: 'name,version\nXuYa,1.0',
    xml: '<root>\n  <name>XuYa</name>\n  <version>1.0</version>\n</root>',
    toml: 'name = "XuYa"\nversion = "1.0"',
    properties: 'name = XuYa\nversion = 1.0',
  };
  return map[fromFormat.value];
});

function swapFormats() {
  const t = fromFormat.value;
  fromFormat.value = toFormat.value;
  toFormat.value = t;
  if (output.value && !error.value) {
    input.value = output.value;
  }
}

type JsonVal = string | number | boolean | null | JsonVal[] | { [k: string]: JsonVal };

function parseToObj(format: Format, text: string): JsonVal {
  switch (format) {
    case 'json':
      return JSON.parse(text);
    case 'yaml':
      return yamlLoad(text) as JsonVal;
    case 'xml':
      return xmlToJson(xml2js(text, { compact: true }) as Record<string, unknown>);
    case 'csv':
      return csvToJson(text);
    case 'toml':
      return parseToml(text);
    case 'properties':
      return parseProperties(text);
  }
}

function objToFormat(format: Format, obj: JsonVal): string {
  switch (format) {
    case 'json':
      return JSON.stringify(obj, null, jsonPretty.value ? 2 : 0);
    case 'yaml':
      return yamlDump(obj, { indent: 2, lineWidth: 120 });
    case 'xml':
      return js2xml(jsonToXml(obj) as Record<string, unknown>, { compact: true, spaces: 2 });
    case 'csv':
      return jsonToCsv(obj);
    case 'toml':
      return toToml(obj);
    case 'properties':
      return toProperties(obj);
  }
}

function xmlToJson(node: unknown): JsonVal {
  if (typeof node !== 'object' || node === null) return node as JsonVal;
  const n = node as Record<string, unknown>;
  if (n._text !== undefined) return n._text as JsonVal;
  const result: Record<string, JsonVal> = {};
  for (const [k, v] of Object.entries(n)) {
    if (k.startsWith('_')) continue;
    if (Array.isArray(v)) {
      result[k] = v.map((item) => xmlToJson(item));
    } else if (typeof v === 'object' && v !== null) {
      result[k] = xmlToJson(v);
    } else {
      result[k] = v as JsonVal;
    }
  }
  return Object.keys(result).length === 1 && result._root ? result._root : result;
}

function jsonToXml(obj: JsonVal): unknown {
  if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) {
    return { _text: String(obj) };
  }
  return obj;
}

function csvToJson(text: string): JsonVal {
  const lines = text.trim().split('\n');
  if (lines.length < 1) return [];
  const headers = lines[0]!.split(',').map((h) => h.trim().replace(/^["']|["']$/g, ''));
  const rows: Record<string, JsonVal>[] = [];
  for (let i = 1; i < lines.length; i++) {
    const vals = parseCsvLine(lines[i]!);
    const row: Record<string, JsonVal> = {};
    headers.forEach((h, j) => {
      row[h] = (vals[j] ?? '') as JsonVal;
    });
    rows.push(row);
  }
  return rows;
}

function parseCsvLine(line: string): string[] {
  const result: string[] = [];
  let cur = '';
  let inQuote = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i]!;
    if (c === '"') {
      if (inQuote && line[i + 1] === '"') {
        cur += '"';
        i++;
      } else {
        inQuote = !inQuote;
      }
    } else if (c === ',' && !inQuote) {
      result.push(cur);
      cur = '';
    } else {
      cur += c;
    }
  }
  result.push(cur);
  return result;
}

function jsonToCsv(obj: JsonVal): string {
  if (!Array.isArray(obj) || obj.length === 0) {
    if (typeof obj === 'object' && obj !== null) obj = [obj];
    else return '';
  }
  const arr = obj as Record<string, JsonVal>[];
  const headers = [...new Set(arr.flatMap((r) => Object.keys(r)))];
  const escape = (v: unknown) => {
    const s = String(v ?? '');
    return s.includes(',') || s.includes('"') || s.includes('\n')
      ? `"${s.replace(/"/g, '""')}"`
      : s;
  };
  const lines = [headers.join(',')];
  for (const row of arr) {
    lines.push(headers.map((h) => escape(row[h])).join(','));
  }
  return lines.join('\n');
}

function parseToml(text: string): JsonVal {
  const result: Record<string, JsonVal> = {};
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith('[')) continue;
    const eqIdx = trimmed.indexOf('=');
    if (eqIdx < 0) continue;
    const key = trimmed
      .slice(0, eqIdx)
      .trim()
      .replace(/^["']|["']$/g, '');
    let val = trimmed.slice(eqIdx + 1).trim();
    val = val.replace(/^["']|["']$/g, '');
    if (val === 'true') result[key] = true;
    else if (val === 'false') result[key] = false;
    else if (/^-?\d+$/.test(val)) result[key] = parseInt(val, 10);
    else if (/^-?\d+\.\d+$/.test(val)) result[key] = parseFloat(val);
    else result[key] = val;
  }
  return result;
}

function toToml(obj: JsonVal): string {
  if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) return '';
  const lines: string[] = [];
  for (const [k, v] of Object.entries(obj)) {
    if (typeof v === 'string') lines.push(`${k} = "${v}"`);
    else if (typeof v === 'number' || typeof v === 'boolean') lines.push(`${k} = ${v}`);
    else lines.push(`${k} = ${JSON.stringify(v)}`);
  }
  return lines.join('\n');
}

function parseProperties(text: string): JsonVal {
  const result: Record<string, JsonVal> = {};
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const eqIdx = trimmed.indexOf('=');
    if (eqIdx < 0) continue;
    const key = trimmed.slice(0, eqIdx).trim();
    const val = trimmed.slice(eqIdx + 1).trim();
    result[key] = val;
  }
  return result;
}

function toProperties(obj: JsonVal): string {
  if (typeof obj !== 'object' || obj === null) return '';
  const lines: string[] = [];
  const flatten = (o: Record<string, JsonVal>, prefix: string) => {
    for (const [k, v] of Object.entries(o)) {
      const key = prefix ? `${prefix}.${k}` : k;
      if (typeof v === 'object' && v !== null && !Array.isArray(v)) {
        flatten(v as Record<string, JsonVal>, key);
      } else if (Array.isArray(v)) {
        v.forEach((item, i) => lines.push(`${key}[${i}] = ${item}`));
      } else {
        lines.push(`${key} = ${v}`);
      }
    }
  };
  flatten(obj as Record<string, JsonVal>, '');
  return lines.join('\n');
}

function convert() {
  error.value = '';
  output.value = '';
  const text = input.value.trim();
  if (!text) return;
  if (fromFormat.value === toFormat.value) {
    output.value = text;
    return;
  }
  try {
    const obj = parseToObj(fromFormat.value, text);
    output.value = objToFormat(toFormat.value, obj);
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

watch([input, fromFormat, toFormat, jsonPretty], convert, { immediate: true });

async function copyOutput() {
  if (output.value) await copyToClipboard(output.value, '已复制转换结果');
}
</script>

<style scoped>
.conv-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 14px;
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
.opt-select:focus {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.swap-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text-secondary);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.swap-btn:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}
.dir-badge {
  font-family: var(--xuya-font-mono);
  font-size: 11px;
  font-weight: 700;
  color: var(--xuya-accent);
  padding: 3px 8px;
  border-radius: 99px;
  background: var(--xuya-accent-soft);
  border: 1px solid var(--xuya-accent-soft-strong);
  white-space: nowrap;
}
.sep {
  width: 1px;
  height: 20px;
  background: var(--xuya-border);
  margin: 0 2px;
}

.conv-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
}
.conv-col {
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
  min-height: 280px;
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
.editor:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.editor::placeholder {
  color: var(--xuya-text-tertiary);
  white-space: pre-wrap;
}

.error-output {
  flex: 1;
  min-height: 280px;
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
</style>
