<template>
  <ToolShell
    title="编码转换"
    :icon="Binary"
    description="Base64 / URL / HTML 实体 / Hex / Binary / Unicode 双向实时转换。"
  >
    <div class="mode-tabs">
      <button
        v-for="m in modes"
        :key="m.id"
        class="mode-tab"
        :class="{ active: mode === m.id }"
        @click="mode = m.id"
      >
        {{ m.label }}
      </button>
    </div>

    <div class="enc-toolbar">
      <div class="seg">
        <button :class="{ active: !reversed }" @click="reversed = false">编码 ▸</button>
        <button :class="{ active: reversed }" @click="reversed = true">◂ 解码</button>
      </div>

      <span class="sep"></span>

      <span class="tb-stat">{{ directionLabel }} · {{ modeLabel }}</span>

      <span style="flex: 1"></span>

      <BaseButton variant="ghost" :disabled="!output" @click="doCopy">
        <Copy :size="13" />
        复制
      </BaseButton>
      <BaseButton variant="ghost" :disabled="!output" @click="swapToInput">→ 输入</BaseButton>
      <BaseButton variant="ghost" @click="clearAll">清空</BaseButton>
    </div>

    <div class="enc-grid">
      <div class="enc-col">
        <div class="col-head">
          <span>输入</span>
          <span class="stat">{{ input.length }} 字符 · {{ inputBytes }} 字节</span>
        </div>
        <textarea
          v-model="input"
          class="editor"
          :placeholder="`输入要${directionLabel}的文本…`"
          spellcheck="false"
        ></textarea>
      </div>

      <div class="enc-col">
        <div class="col-head">
          <span>结果</span>
          <span class="stat" :class="{ err: !!errorMsg }">
            {{ errorMsg ? '错误' : output ? `${output.length} 字符 · ${outputBytes} 字节` : '' }}
          </span>
        </div>
        <pre v-if="errorMsg" class="output error-output"><code>{{ errorMsg }}</code></pre>
        <pre v-else class="output"><code>{{ output || '—' }}</code></pre>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Binary, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

type Mode = 'base64' | 'url' | 'html' | 'hex' | 'binary' | 'unicode';
const modes: { id: Mode; label: string }[] = [
  { id: 'base64', label: 'Base64' },
  { id: 'url', label: 'URL' },
  { id: 'html', label: 'HTML 实体' },
  { id: 'hex', label: 'Hex' },
  { id: 'binary', label: 'Binary' },
  { id: 'unicode', label: 'Unicode' },
];

const mode = useToolState<Mode>('encoding', 'mode', 'base64');
const input = useToolState('encoding', 'input', '');
const output = ref('');
const errorMsg = ref('');
const reversed = useToolState('encoding', 'reversed', false);

const modeLabel = computed(() => modes.find((m) => m.id === mode.value)?.label ?? '');
const directionLabel = computed(() => (reversed.value ? '解码' : '编码'));

const inputBytes = computed(() => new TextEncoder().encode(input.value).length);
const outputBytes = computed(() => new TextEncoder().encode(output.value).length);

function utf8ToBytes(str: string): Uint8Array {
  return new TextEncoder().encode(str);
}
function bytesToUtf8(bytes: Uint8Array): string {
  return new TextDecoder().decode(bytes);
}
function bytesToBase64(bytes: Uint8Array): string {
  let bin = '';
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin);
}
function base64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const arr = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) arr[i] = bin.charCodeAt(i);
  return arr;
}

function hexToBytes(hex: string): Uint8Array {
  if (hex.length % 2 !== 0) throw new Error('Hex 长度必须为偶数');
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(hex.substr(i * 2, 2), 16);
  return out;
}
function binaryToBytes(bin: string): Uint8Array {
  if (bin.length % 8 !== 0) throw new Error('二进制长度必须为 8 的倍数');
  const out = new Uint8Array(bin.length / 8);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(bin.substr(i * 8, 8), 2);
  return out;
}

function encodeHtmlEntities(s: string): string {
  const map: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;',
  };
  return s.replace(/[&<>"']/g, (c) => map[c] ?? c);
}
function decodeHtmlEntities(s: string): string {
  const txt = document.createElement('textarea');
  txt.innerHTML = s;
  return txt.value;
}

function encodeUnicode(str: string): string {
  return Array.from(str)
    .map((char) => {
      const code = char.codePointAt(0)!;
      if (code < 128) return char;
      if (code <= 0xffff) return '\\u' + code.toString(16).padStart(4, '0').toUpperCase();
      return '\\u{' + code.toString(16).toUpperCase() + '}';
    })
    .join('');
}
function decodeUnicode(str: string): string {
  return str.replace(/\\u\{([0-9a-fA-F]+)\}|\\u([0-9a-fA-F]{4})/g, (_, braced, hex) => {
    const code = parseInt(braced || hex, 16);
    return String.fromCodePoint(code);
  });
}

function run() {
  errorMsg.value = '';
  output.value = '';
  const text = input.value;
  if (!text) return;
  try {
    switch (mode.value) {
      case 'base64':
        output.value = reversed.value
          ? bytesToUtf8(base64ToBytes(text.trim()))
          : bytesToBase64(utf8ToBytes(text));
        break;
      case 'url':
        output.value = reversed.value ? decodeURIComponent(text) : encodeURIComponent(text);
        break;
      case 'html':
        output.value = reversed.value ? decodeHtmlEntities(text) : encodeHtmlEntities(text);
        break;
      case 'hex':
        if (reversed.value) {
          const clean = text.replace(/0x/gi, '').replace(/\s+/g, '');
          output.value = bytesToUtf8(hexToBytes(clean));
        } else {
          output.value = Array.from(utf8ToBytes(text))
            .map((b) => b.toString(16).padStart(2, '0'))
            .join(' ');
        }
        break;
      case 'binary':
        if (reversed.value) {
          const clean = text.replace(/\s+/g, '');
          output.value = bytesToUtf8(binaryToBytes(clean));
        } else {
          output.value = Array.from(utf8ToBytes(text))
            .map((b) => b.toString(2).padStart(8, '0'))
            .join(' ');
        }
        break;
      case 'unicode':
        output.value = reversed.value ? decodeUnicode(text) : encodeUnicode(text);
        break;
    }
  } catch (e) {
    errorMsg.value = e instanceof Error ? e.message : String(e);
  }
}

async function doCopy() {
  if (output.value) await copyToClipboard(output.value);
}
function swapToInput() {
  if (output.value) {
    input.value = output.value;
    output.value = '';
  }
}
function clearAll() {
  input.value = '';
  output.value = '';
  errorMsg.value = '';
}

watch([input, mode, reversed], run, { immediate: true });
</script>

<style scoped>
.mode-tabs {
  display: flex;
  gap: 2px;
  margin-bottom: 14px;
  border-bottom: 1px solid var(--xuya-border);
}
.mode-tab {
  padding: 8px 16px;
  font-size: 13px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition:
    color var(--xuya-duration-fast),
    border-color var(--xuya-duration-fast);
}
.mode-tab:hover {
  color: var(--xuya-text);
}
.mode-tab.active {
  color: var(--xuya-accent);
  border-bottom-color: var(--xuya-accent);
  font-weight: 600;
}

.enc-toolbar {
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
.tb-stat {
  font-size: 11.5px;
  color: var(--xuya-text-tertiary);
}

.enc-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
}
.enc-col {
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
  min-height: 280px;
  margin: 0;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  overflow: auto;
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.65;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--xuya-text);
}
.error-output {
  border-color: var(--xuya-danger);
}
.error-output code {
  color: var(--xuya-danger);
}
</style>
