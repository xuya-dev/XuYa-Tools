<template>
  <ToolShell title="Markdown 预览" :icon="FileText" description="实时分栏:左侧编辑,右侧预览,支持代码块、表格、列表。">
    <div class="md-toolbar">
      <BaseButton variant="ghost" @click="loadSample"><Sparkles :size="13" /> 示例</BaseButton>
      <BaseButton variant="ghost" @click="clear">清空</BaseButton>
      <span style="flex:1"></span>
      <div class="seg">
        <button :class="{ active: view === 'split' }" @click="view = 'split'">分栏</button>
        <button :class="{ active: view === 'preview' }" @click="view = 'preview'">仅预览</button>
      </div>
    </div>

    <div class="md-grid" :class="{ 'preview-only': view === 'preview' }">
      <div v-show="view !== 'preview'" class="md-edit-col">
        <div class="col-head"><span>Markdown</span><span class="stat">{{ md.length }} 字符</span></div>
        <textarea v-model="md" class="editor" placeholder="# 标题&#10;&#10;输入 Markdown..." spellcheck="false"></textarea>
      </div>
      <div class="md-preview-col">
        <div class="col-head"><span>预览</span></div>
        <div class="preview" v-html="rendered"></div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { FileText, Sparkles } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';

const view = ref<'split' | 'preview'>('split');
const md = ref(`# XuYa Tools

一个**程序员工具箱**,支持以下功能:

- JSON 格式化
- 正则测试
- 颜色工具

## 代码示例

\`\`\`js
function greet(name) {
  return \`Hello, \${name}!\`;
}
\`\`\`

## 表格

| 工具 | 分类 |
|------|------|
| JSON | 格式转换 |
| 正则 | 格式转换 |

> 提示:实时渲染,无需点击按钮。

行内代码 \`const x = 1\` 与 [链接](https://example.com)。
`);

/** 轻量 Markdown 渲染 (无依赖) */
function render(src: string): string {
  if (!src.trim()) return '<p style="color:var(--xuya-text-tertiary)">输入 Markdown 后实时预览</p>';
  const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  let lines = src.split('\n');
  let html = '';
  let inCode = false;
  let codeBuf: string[] = [];
  let paraBuf: string[] = [];
  let listBuf: string[] = [];

  const inline = (s: string) => esc(s)
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\*([^*]+)\*/g, '<em>$1</em>')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>');

  const flushPara = () => { if (paraBuf.length) { html += '<p>' + inline(paraBuf.join(' ')) + '</p>'; paraBuf = []; } };
  const flushList = () => {
    if (listBuf.length) {
      html += '<ul>' + listBuf.map((l) => `<li>${inline(l)}</li>`).join('') + '</ul>';
      listBuf = [];
    }
  };

  for (const raw of lines) {
    const line = raw;
    // 代码块
    if (line.trim().startsWith('```')) {
      if (inCode) { html += `<pre><code>${esc(codeBuf.join('\n'))}</code></pre>`; inCode = false; codeBuf = []; }
      else { flushPara(); flushList(); inCode = true; }
      continue;
    }
    if (inCode) { codeBuf.push(line); continue; }

    // 标题
    const h = line.match(/^(#{1,6})\s+(.*)/);
    if (h) { flushPara(); flushList(); const lvl = h[1]!.length; html += `<h${lvl}>${inline(h[2]!)}</h${lvl}>`; continue; }
    // 引用
    if (line.startsWith('> ')) { flushPara(); flushList(); html += `<blockquote>${inline(line.slice(2))}</blockquote>`; continue; }
    // 分割线
    if (/^(-{3,}|\*{3,})$/.test(line.trim())) { flushPara(); flushList(); html += '<hr/>'; continue; }
    // 列表
    if (line.match(/^[-*]\s+/)) { flushPara(); listBuf.push(line.replace(/^[-*]\s+/, '')); continue; }
    // 表格 (简化:连续 | 行)
    if (line.includes('|') && line.trim().startsWith('|')) { flushPara(); flushList(); html += `<div class="md-raw-row">${inline(line)}</div>`; continue; }
    // 空行
    if (!line.trim()) { flushPara(); flushList(); continue; }
    paraBuf.push(line);
  }
  if (inCode) html += `<pre><code>${esc(codeBuf.join('\n'))}</code></pre>`;
  flushPara(); flushList();
  return html;
}

const rendered = computed(() => render(md.value));

function loadSample() {
  md.value = '# 示例文档\n\n这是 **Markdown** 示例。\n\n- 列表项 1\n- 列表项 2\n\n```js\nconsole.log("hello");\n```\n';
}
function clear() { md.value = ''; }
</script>

<style scoped>
.md-toolbar { display: flex; align-items: center; gap: 8px; margin-bottom: 14px; }
.seg { display: inline-flex; background: var(--xuya-input-bg); border-radius: var(--xuya-radius-sm); padding: 2px; gap: 2px; margin-left: auto; }
.seg button { padding: 5px 14px; font-size: 12px; color: var(--xuya-text-secondary); background: transparent; border: none; border-radius: var(--xuya-radius-sm); transition: .1s; }
.seg button.active { background: var(--xuya-bg-elevated); color: var(--xuya-accent); font-weight: 600; }
.md-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; flex: 1; min-height: 0; }
.md-grid.preview-only { grid-template-columns: 1fr; }
.md-edit-col, .md-preview-col { display: flex; flex-direction: column; gap: 6px; min-height: 0; }
.col-head { display: flex; justify-content: space-between; font-size: 12px; font-weight: 600; color: var(--xuya-text-secondary); }
.stat { font-size: 10.5px; font-weight: 400; opacity: .75; }
.editor { flex: 1; min-height: 320px; padding: 14px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); font-family: var(--xuya-font-mono); font-size: 13px; line-height: 1.6; resize: none; transition: border-color .12s, box-shadow .12s; }
.editor:focus { outline: none; border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }
.preview { flex: 1; min-height: 320px; padding: 16px 18px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); overflow: auto; font-size: 14px; line-height: 1.7; color: var(--xuya-text); }
.preview :deep(h1) { font-size: 22px; font-weight: 700; margin: 0 0 12px; padding-bottom: 8px; border-bottom: 1px solid var(--xuya-border); }
.preview :deep(h2) { font-size: 18px; font-weight: 600; margin: 18px 0 8px; }
.preview :deep(h3) { font-size: 15px; font-weight: 600; margin: 14px 0 6px; }
.preview :deep(p) { margin: 8px 0; }
.preview :deep(ul) { margin: 8px 0; padding-left: 22px; }
.preview :deep(li) { margin: 3px 0; }
.preview :deep(code) { background: var(--xuya-card-bg); padding: 2px 6px; border-radius: var(--xuya-radius-sm); font-family: var(--xuya-font-mono); font-size: 12.5px; color: var(--xuya-accent); }
.preview :deep(pre) { background: var(--xuya-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-sm); padding: 12px; overflow: auto; margin: 10px 0; }
.preview :deep(pre code) { background: none; padding: 0; color: var(--xuya-text); }
.preview :deep(blockquote) { border-left: 3px solid var(--xuya-accent); padding: 6px 14px; margin: 10px 0; color: var(--xuya-text-secondary); background: var(--xuya-accent-soft); border-radius: 0 var(--xuya-radius-sm) var(--xuya-radius-sm) 0; }
.preview :deep(a) { color: var(--xuya-accent); }
.preview :deep(hr) { border: none; border-top: 1px solid var(--xuya-border); margin: 14px 0; }
.preview :deep(strong) { font-weight: 700; }
.preview :deep(.md-raw-row) { font-family: var(--xuya-font-mono); font-size: 12px; color: var(--xuya-text-tertiary); }
</style>
