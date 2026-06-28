<template>
  <ToolShell title="JSON 格式化" :icon="Braces" description="美化、压缩、转义、反转义 JSON,支持错误定位与字符统计。">
    <div class="json-toolbar">
      <BaseButton variant="primary" @click="format">美化</BaseButton>
      <BaseButton @click="minify">压缩</BaseButton>
      <BaseButton @click="escape">转义</BaseButton>
      <BaseButton @click="unescape">反转义</BaseButton>
      <span class="sep"></span>
      <BaseButton variant="ghost" :disabled="!output" @click="copyOutput">
        <Copy :size="13" /> 复制结果
      </BaseButton>
      <BaseButton variant="ghost" @click="clearAll">清空</BaseButton>
      <BaseButton variant="ghost" @click="loadSample">示例</BaseButton>
    </div>

    <div class="editor-grid">
      <div class="editor-col">
        <div class="col-head">
          <span>输入</span>
          <span class="stat">{{ inputStats }}</span>
        </div>
        <textarea
          v-model="input"
          class="editor"
          placeholder='在此粘贴 JSON,例如 {"name":"XuYa","version":1}'
          spellcheck="false"
        ></textarea>
      </div>

      <div class="editor-col">
        <div class="col-head">
          <span>输出</span>
          <span class="stat" :class="{ err: !!errorMsg }">{{ outputStats }}</span>
        </div>
        <pre class="output" :class="{ 'has-error': errorMsg }"><code>{{ errorMsg || output }}</code></pre>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { Braces, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';

const toast = useToast();
const input = ref('');
const output = ref('');
const errorMsg = ref('');

const inputStats = computed(() => statsOf(input.value));
const outputStats = computed(() => (errorMsg.value ? '解析失败' : statsOf(output.value)));

function statsOf(s: string) {
  if (!s) return '';
  const lines = s.split('\n').length;
  return `${s.length} 字符 · ${lines} 行`;
}

/** 美化 */
function format() {
  errorMsg.value = '';
  if (!input.value.trim()) return toast.error('请先输入 JSON');
  try {
    output.value = JSON.stringify(JSON.parse(input.value), null, 2);
    toast.success('格式化成功');
  } catch (e) {
    errorMsg.value = formatError(e, input.value);
  }
}

/** 压缩 */
function minify() {
  errorMsg.value = '';
  if (!input.value.trim()) return toast.error('请先输入 JSON');
  try {
    output.value = JSON.stringify(JSON.parse(input.value));
    toast.success('压缩成功');
  } catch (e) {
    errorMsg.value = formatError(e, input.value);
  }
}

/** 转义:转为可嵌入代码的字符串字面量 */
function escape() {
  errorMsg.value = '';
  if (!input.value) return toast.error('请先输入内容');
  output.value = JSON.stringify(input.value);
  toast.success('转义成功');
}

/** 反转义:还原字符串字面量 */
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

/** 友好的错误信息 (含行列定位) */
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

async function copyOutput() {
  if (!output.value && !errorMsg.value) return;
  await copyToClipboard(errorMsg.value || output.value);
}

function clearAll() {
  input.value = '';
  output.value = '';
  errorMsg.value = '';
}

function loadSample() {
  input.value = JSON.stringify(
    { name: 'XuYa Tools', version: '0.1.0', features: ['JSON', 'Base64', 'JWT'], nested: { ok: true } },
    null,
    2
  );
  output.value = '';
  errorMsg.value = '';
}
</script>

<style scoped>
.json-toolbar {
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
  margin: 0 4px;
}

.editor-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
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
  line-height: 1.6;
  resize: none;
  transition: border-color 0.12s, box-shadow 0.12s;
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
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}
.output code {
  color: var(--xuya-text);
}
.output.has-error code {
  color: var(--xuya-danger);
}
</style>
