<template>
  <ToolShell title="编码转换" :icon="Binary" description="Base64 / URL / HTML 实体 / Hex / Binary 双向转换。">
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

    <div class="encode-grid">
      <div class="encode-col">
        <div class="col-head">
          <span>原文 ({{ modeLabel }})</span>
          <button class="mini-btn" @click="swapToInput">↑ 用结果填充</button>
        </div>
        <textarea
          v-model="input"
          class="editor"
          :placeholder="`输入要${currentDirectionLabel}的文本`"
          spellcheck="false"
        ></textarea>
      </div>

      <div class="encode-col">
        <div class="col-head">
          <span>结果</span>
          <div class="col-actions">
            <button class="mini-btn" :disabled="!output" @click="doCopy">
              <Copy :size="12" /> 复制
            </button>
            <button class="mini-btn" @click="reverseDirection">
              <ArrowLeftRight :size="12" /> {{ reversed ? '正向' : '反向' }}
            </button>
          </div>
        </div>
        <pre class="output" :class="{ err: !!errorMsg }"><code>{{ errorMsg || output || '—' }}</code></pre>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Binary, Copy, ArrowLeftRight } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

type Mode = 'base64' | 'url' | 'html' | 'hex' | 'binary';
const modes: { id: Mode; label: string }[] = [
  { id: 'base64', label: 'Base64' },
  { id: 'url', label: 'URL' },
  { id: 'html', label: 'HTML 实体' },
  { id: 'hex', label: 'Hex' },
  { id: 'binary', label: 'Binary' },
];

const mode = useToolState<Mode>('encoding', 'mode', 'base64');
const input = useToolState('encoding', 'input', '');
const output = ref('');
const errorMsg = ref('');
const reversed = useToolState('encoding', 'reversed', false);

const modeLabel = computed(() => modes.find((m) => m.id === mode.value)?.label ?? '');
const currentDirectionLabel = computed(() => (reversed.value ? '解码' : '编码'));

// ===== UTF-8 安全的 Base64 =====
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

// ===== 各模式编/解码 =====
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
      case 'hex': {
        if (reversed.value) {
          const clean = text.replace(/0x/gi, '').replace(/\s+/g, '');
          output.value = bytesToUtf8(hexToBytes(clean));
        } else {
          output.value = Array.from(utf8ToBytes(text))
            .map((b) => b.toString(16).padStart(2, '0'))
            .join(' ');
        }
        break;
      }
      case 'binary': {
        if (reversed.value) {
          const clean = text.replace(/\s+/g, '');
          output.value = bytesToUtf8(binaryToBytes(clean));
        } else {
          output.value = Array.from(utf8ToBytes(text))
            .map((b) => b.toString(2).padStart(8, '0'))
            .join(' ');
        }
        break;
      }
    }
  } catch (e) {
    errorMsg.value = `❌ ${e instanceof Error ? e.message : String(e)}`;
  }
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
  const map: Record<string, string> = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' };
  return s.replace(/[&<>"']/g, (c) => map[c] ?? c);
}
function decodeHtmlEntities(s: string): string {
  const txt = document.createElement('textarea');
  txt.innerHTML = s;
  return txt.value;
}

// ===== 操作 =====
async function doCopy() {
  if (output.value) await copyToClipboard(output.value);
}
function swapToInput() {
  if (output.value) {
    input.value = output.value;
    output.value = '';
  }
}
function reverseDirection() {
  reversed.value = !reversed.value;
}

watch([input, mode, reversed], run, { immediate: true });
</script>

<style scoped>
.mode-tabs {
  display: flex;
  gap: 6px;
  margin-bottom: 16px;
  border-bottom: 1px solid var(--xuya-border);
}
.mode-tab {
  padding: 8px 14px;
  font-size: 13px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: color 0var(--xuya-duration-fast), border-color 0var(--xuya-duration-fast);
}
.mode-tab:hover {
  color: var(--xuya-text);
}
.mode-tab.active {
  color: var(--xuya-accent);
  border-bottom-color: var(--xuya-accent);
  font-weight: 600;
}

.encode-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
}
.encode-col {
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
  transition: all 0var(--xuya-duration-fast);
}
.mini-btn:hover:not(:disabled) {
  color: var(--xuya-text);
  border-color: var(--xuya-accent);
}
.mini-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.editor {
  flex: 1;
  min-height: 300px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.6;
  resize: none;
  transition: border-color 0var(--xuya-duration-fast), box-shadow 0var(--xuya-duration-fast);
}
.editor:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.editor::placeholder { color: var(--xuya-text-tertiary); }

.output {
  flex: 1;
  min-height: 300px;
  margin: 0;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  overflow: auto;
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--xuya-text);
}
.output.err code {
  color: var(--xuya-danger);
}
</style>
