<template>
  <ToolShell
    title="Markdown 预览"
    :icon="FileText"
    description="实时分栏编辑预览，支持 GFM 表格、任务列表、代码高亮。"
  >
    <div class="md-toolbar">
      <div class="seg">
        <button :class="{ active: view === 'split' }" @click="view = 'split'">分栏</button>
        <button :class="{ active: view === 'preview' }" @click="view = 'preview'">仅预览</button>
        <button :class="{ active: view === 'edit' }" @click="view = 'edit'">仅编辑</button>
      </div>

      <span class="sep"></span>

      <span class="tb-stat">{{ md.length }} 字符 · {{ md.split('\n').length }} 行</span>

      <span style="flex: 1"></span>

      <BaseButton variant="ghost" :disabled="!rendered" @click="copyHtml">复制 HTML</BaseButton>
      <BaseButton variant="ghost" :disabled="!rendered" @click="exportHtml">导出 HTML</BaseButton>
      <BaseButton variant="ghost" :disabled="!md" @click="copyMd">复制 MD</BaseButton>
      <BaseButton variant="ghost" @click="clear">清空</BaseButton>
    </div>

    <div class="md-grid" :class="'view-' + view">
      <div v-show="view !== 'preview'" class="md-edit-col">
        <div class="col-head">
          <span>Markdown</span>
          <span class="stat">{{ wordCount }} 词</span>
        </div>
        <textarea
          v-model="md"
          class="editor"
          placeholder="# 标题&#10;&#10;输入 Markdown 内容…"
          spellcheck="false"
        ></textarea>
      </div>
      <div v-show="view !== 'edit'" class="md-preview-col">
        <div class="col-head">
          <span>预览</span>
        </div>
        <div class="preview" v-html="rendered || placeholder"></div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { FileText } from '@lucide/vue';
import { Marked } from 'marked';
import { markedHighlight } from 'marked-highlight';
import hljs from 'highlight.js/lib/core';
import javascript from 'highlight.js/lib/languages/javascript';
import typescript from 'highlight.js/lib/languages/typescript';
import python from 'highlight.js/lib/languages/python';
import bash from 'highlight.js/lib/languages/bash';
import xml from 'highlight.js/lib/languages/xml';
import css from 'highlight.js/lib/languages/css';
import json from 'highlight.js/lib/languages/json';
import sql from 'highlight.js/lib/languages/sql';
import yaml from 'highlight.js/lib/languages/yaml';
import markdown from 'highlight.js/lib/languages/markdown';
import shell from 'highlight.js/lib/languages/shell';
import go from 'highlight.js/lib/languages/go';
import rust from 'highlight.js/lib/languages/rust';
import java from 'highlight.js/lib/languages/java';
import c from 'highlight.js/lib/languages/c';
import cpp from 'highlight.js/lib/languages/cpp';

const HL_LANGS: Record<string, unknown> = {
  javascript, typescript, python, bash, xml, css, json, sql, yaml, markdown, shell,
  go, rust, java, c, cpp,
};
for (const [name, lang] of Object.entries(HL_LANGS)) hljs.registerLanguage(name, lang as never);
hljs.registerAliases(['js', 'jsx'], { languageName: 'javascript' });
hljs.registerAliases(['ts', 'tsx'], { languageName: 'typescript' });
hljs.registerAliases(['py'], { languageName: 'python' });
hljs.registerAliases(['sh'], { languageName: 'bash' });
hljs.registerAliases(['html'], { languageName: 'xml' });
hljs.registerAliases(['yml'], { languageName: 'yaml' });
hljs.registerAliases(['rs'], { languageName: 'rust' });
hljs.registerAliases(['c++', 'cpp'], { languageName: 'cpp' });
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';
import { useToast } from '@/composables/useToast';

const toast = useToast();

const md = useToolState('markdown', 'input', '');
const view = useToolState<'split' | 'preview' | 'edit'>('markdown', 'view', 'split');

const markedInstance = new Marked(
  markedHighlight({
    langPrefix: 'hljs language-',
    highlight(code, lang) {
      if (lang && hljs.getLanguage(lang)) {
        try {
          return hljs.highlight(code, { language: lang }).value;
        } catch {
          /* fallback */
        }
      }
      return code;
    },
  }),
);

markedInstance.setOptions({ gfm: true, breaks: false });

const rendered = computed(() => {
  if (!md.value.trim()) return '';
  try {
    return markedInstance.parse(md.value) as string;
  } catch {
    return '<p style="color:var(--xuya-danger)">Markdown 解析错误</p>';
  }
});

const placeholder = '<div class="md-empty">输入 Markdown 后实时预览</div>';

const wordCount = computed(() => {
  const text = md.value.trim();
  if (!text) return 0;
  const cjk = text.match(/[\u4e00-\u9fa5]/g)?.length ?? 0;
  const en = text
    .replace(/[\u4e00-\u9fa5]/g, ' ')
    .trim()
    .split(/\s+/)
    .filter(Boolean).length;
  return cjk + en;
});

async function copyHtml() {
  if (!rendered.value) return;
  await copyToClipboard(rendered.value, '已复制 HTML');
}

async function copyMd() {
  if (!md.value) return;
  await copyToClipboard(md.value, '已复制 Markdown');
}

function exportHtml() {
  if (!rendered.value) return;
  const fullHtml = `<!DOCTYPE html>
<html lang="zh">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Markdown 导出</title>
<style>
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 760px; margin: 40px auto; padding: 0 20px; color: #1f2328; line-height: 1.7; }
h1 { font-size: 2em; border-bottom: 1px solid #eee; padding-bottom: .3em; }
h2 { font-size: 1.5em; border-bottom: 1px solid #eee; padding-bottom: .3em; }
h3 { font-size: 1.25em; }
pre { background: #f6f8fa; padding: 14px; border-radius: 6px; overflow: auto; }
code { font-family: 'JetBrains Mono', monospace; font-size: .92em; }
p code, li code { background: #f0f0f0; padding: 2px 6px; border-radius: 4px; }
table { border-collapse: collapse; width: 100%; margin: 12px 0; }
th, td { border: 1px solid #d0d7de; padding: 6px 13px; }
th { background: #f6f8fa; font-weight: 600; }
blockquote { border-left: 4px solid #dfe2e5; margin: 0; padding: 4px 16px; color: #636c76; }
img { max-width: 100%; }
a { color: #0969da; }
hr { border: none; border-top: 2px solid #eee; margin: 24px 0; }
ul, ol { padding-left: 2em; }
</style>
</head>
<body>
${rendered.value}
</body>
</html>`;
  const blob = new Blob([fullHtml], { type: 'text/html;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'markdown.html';
  a.click();
  URL.revokeObjectURL(url);
  toast.success('已导出 markdown.html');
}

function clear() {
  md.value = '';
}
</script>

<style scoped>
.md-toolbar {
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

.md-grid {
  display: grid;
  gap: 14px;
  flex: 1;
  min-height: 0;
}
.md-grid.view-split {
  grid-template-columns: 1fr 1fr;
}
.md-grid.view-preview,
.md-grid.view-edit {
  grid-template-columns: 1fr;
}
.md-edit-col,
.md-preview-col {
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

.editor {
  flex: 1;
  min-height: 320px;
  padding: 14px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 13px;
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

.preview {
  flex: 1;
  min-height: 320px;
  padding: 20px 24px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  overflow: auto;
  font-size: 14px;
  line-height: 1.75;
  color: var(--xuya-text);
}
.preview :deep(.md-empty) {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--xuya-text-tertiary);
  font-size: 13px;
}

.preview :deep(h1) {
  font-size: 1.9em;
  font-weight: 700;
  margin: 0 0 14px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--xuya-border);
}
.preview :deep(h2) {
  font-size: 1.5em;
  font-weight: 600;
  margin: 22px 0 10px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--xuya-border-light);
}
.preview :deep(h3) {
  font-size: 1.2em;
  font-weight: 600;
  margin: 18px 0 8px;
}
.preview :deep(h4) {
  font-size: 1em;
  font-weight: 600;
  margin: 14px 0 6px;
}
.preview :deep(h5),
.preview :deep(h6) {
  font-size: 0.9em;
  font-weight: 600;
  color: var(--xuya-text-secondary);
  margin: 12px 0 6px;
}
.preview :deep(p) {
  margin: 8px 0;
}
.preview :deep(strong) {
  font-weight: 700;
}
.preview :deep(em) {
  font-style: italic;
}
.preview :deep(del) {
  text-decoration: line-through;
  color: var(--xuya-text-tertiary);
}

.preview :deep(ul),
.preview :deep(ol) {
  margin: 8px 0;
  padding-left: 26px;
}
.preview :deep(li) {
  margin: 4px 0;
}
.preview :deep(li input[type='checkbox']) {
  margin-right: 6px;
  accent-color: var(--xuya-accent);
}
.preview :deep(ul ul),
.preview :deep(ol ol),
.preview :deep(ul ol),
.preview :deep(ol ul) {
  margin: 4px 0;
}

.preview :deep(code) {
  background: var(--xuya-card-bg);
  padding: 2px 6px;
  border-radius: var(--xuya-radius-sm);
  font-family: var(--xuya-font-mono);
  font-size: 0.88em;
  color: var(--xuya-accent);
}
.preview :deep(pre) {
  background: var(--xuya-output-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  padding: 14px;
  overflow: auto;
  margin: 12px 0;
}
.preview :deep(pre code) {
  background: none;
  padding: 0;
  border: none;
  color: var(--xuya-text);
  font-size: 12.5px;
  line-height: 1.6;
}

.preview :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 12px 0;
  font-size: 13px;
}
.preview :deep(th),
.preview :deep(td) {
  border: 1px solid var(--xuya-border);
  padding: 6px 13px;
}
.preview :deep(th) {
  background: var(--xuya-bg-subtle);
  font-weight: 600;
}
.preview :deep(tr:nth-child(2n)) {
  background: var(--xuya-bg-subtle);
}

.preview :deep(blockquote) {
  border-left: 3px solid var(--xuya-accent);
  padding: 8px 16px;
  margin: 12px 0;
  color: var(--xuya-text-secondary);
  background: var(--xuya-accent-soft);
  border-radius: 0 var(--xuya-radius-sm) var(--xuya-radius-sm) 0;
}
.preview :deep(blockquote p) {
  margin: 4px 0;
}

.preview :deep(a) {
  color: var(--xuya-accent);
  text-decoration: none;
}
.preview :deep(a:hover) {
  text-decoration: underline;
}
.preview :deep(img) {
  max-width: 100%;
  border-radius: var(--xuya-radius-sm);
}
.preview :deep(hr) {
  border: none;
  border-top: 1px solid var(--xuya-border);
  margin: 20px 0;
}

.preview :deep(.hljs) {
  color: var(--xuya-text);
  background: none;
}
.preview :deep(.hljs-comment),
.preview :deep(.hljs-quote) {
  color: var(--xuya-text-tertiary);
  font-style: italic;
}
.preview :deep(.hljs-keyword),
.preview :deep(.hljs-selector-tag),
.preview :deep(.hljs-built_in),
.preview :deep(.hljs-name),
.preview :deep(.hljs-tag) {
  color: var(--xuya-syn-key);
}
.preview :deep(.hljs-string),
.preview :deep(.hljs-attr),
.preview :deep(.hljs-attribute),
.preview :deep(.hljs-addition) {
  color: var(--xuya-syn-str);
}
.preview :deep(.hljs-number),
.preview :deep(.hljs-literal),
.preview :deep(.hljs-symbol) {
  color: var(--xuya-syn-num);
}
.preview :deep(.hljs-title),
.preview :deep(.hljs-function .hljs-title),
.preview :deep(.hljs-section) {
  color: var(--xuya-info);
  font-weight: 600;
}
.preview :deep(.hljs-variable),
.preview :deep(.hljs-params),
.preview :deep(.hljs-template-variable) {
  color: var(--xuya-warn);
}
.preview :deep(.hljs-type),
.preview :deep(.hljs-class .hljs-title) {
  color: var(--xuya-purple);
}
.preview :deep(.hljs-deletion) {
  color: var(--xuya-danger);
}
</style>
