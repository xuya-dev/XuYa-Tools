<template>
  <ToolShell
    title="代码格式化"
    :icon="Code2"
    description="美化 / 压缩 XML、HTML、CSS、JavaScript 代码,纯前端实现无需联网。"
  >
    <div class="bar">
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
      <div class="seg">
        <button :class="{ active: mode === 'beautify' }" @click="mode = 'beautify'">美化</button>
        <button :class="{ active: mode === 'minify' }" @click="mode = 'minify'">压缩</button>
      </div>
      <div class="opt-group">
        <label class="opt">
          缩进
          <select v-model.number="indent" class="sel">
            <option :value="2">2 空格</option>
            <option :value="4">4 空格</option>
            <option :value="-1">Tab</option>
          </select>
        </label>
      </div>
    </div>

    <div class="grid">
      <div class="col">
        <div class="col-head">
          <span>原始代码</span>
          <button class="mini-btn" @click="loadSample">示例</button>
        </div>
        <textarea
          v-model="input"
          class="editor"
          :placeholder="`粘贴要${mode === 'beautify' ? '美化' : '压缩'}的 ${currentLabel} 代码…`"
          spellcheck="false"
        ></textarea>
      </div>
      <div class="col">
        <div class="col-head">
          <span>结果</span>
          <div class="col-actions">
            <span v-if="stats" class="stat">{{ stats }}</span>
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
import { Code2, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

type Lang = 'xml' | 'html' | 'css' | 'js';
const langs: { id: Lang; label: string }[] = [
  { id: 'xml', label: 'XML' },
  { id: 'html', label: 'HTML' },
  { id: 'css', label: 'CSS' },
  { id: 'js', label: 'JavaScript' },
];

const lang = useToolState<Lang>('codebeautify', 'lang', 'html');
const mode = useToolState<'beautify' | 'minify'>('codebeautify', 'mode', 'beautify');
const indent = useToolState<number>('codebeautify', 'indent', 2);
const input = useToolState('codebeautify', 'input', '');

const output = ref('');
const errorMsg = ref('');
const stats = ref('');
const currentLabel = computed(() => langs.find((l) => l.id === lang.value)?.label ?? '');

function indentStr() {
  return indent.value === -1 ? '\t' : ' '.repeat(indent.value);
}

// ===== 运行 =====
function run() {
  errorMsg.value = '';
  output.value = '';
  stats.value = '';
  const text = input.value;
  if (!text.trim()) return;
  try {
    if (lang.value === 'css') {
      output.value = mode.value === 'beautify' ? beautifyCss(text) : minifyCss(text);
    } else if (lang.value === 'js') {
      output.value = mode.value === 'beautify' ? beautifyJs(text) : minifyJs(text);
    } else {
      // xml / html 共用标签树美化
      output.value = mode.value === 'beautify' ? beautifyMarkup(text) : minifyMarkup(text);
    }
    const before = text.length;
    const after = output.value.length;
    stats.value = `${before} → ${after} 字符`;
  } catch (e) {
    errorMsg.value = `❌ ${e instanceof Error ? e.message : String(e)}`;
  }
}

// ===== CSS =====
function beautifyCss(src: string): string {
  const ind = indentStr();
  // 规整空格,统一换行
  let s = src.replace(/\r\n/g, '\n').replace(/[ \t]+/g, ' ');
  // 在 { } ; 后换行
  s = s
    .replace(/\s*\{\s*/g, ' {\n')
    .replace(/\s*\}\s*/g, '\n}\n')
    .replace(/\s*;\s*/g, ';\n');
  // 逗号分隔的选择器各自一行
  const lines = s.split('\n');
  const out: string[] = [];
  let depth = 0;
  for (let line of lines) {
    line = line.trim();
    if (!line) continue;
    if (line === '}') {
      depth = Math.max(0, depth - 1);
      out.push(ind.repeat(depth) + '}');
      continue;
    }
    if (line.endsWith('{')) {
      out.push(ind.repeat(depth) + line);
      depth++;
    } else {
      // 普通声明或选择器片段
      out.push(ind.repeat(depth) + line);
    }
  }
  return out.join('\n') + '\n';
}

function minifyCss(src: string): string {
  return src
    .replace(/\/\*[\s\S]*?\*\//g, '') // 注释
    .replace(/\s+/g, ' ')
    .replace(/\s*([{}:;,])\s*/g, '$1')
    .replace(/;}/g, '}')
    .trim();
}

// ===== XML / HTML(基于标签的轻量美化,不严格解析) =====
function beautifyMarkup(src: string): string {
  const ind = indentStr();
  // 把 <...>、/>、文本节点切分成 token
  const tokens: string[] = [];
  let i = 0;
  while (i < src.length) {
    if (src[i] === '<') {
      const end = src.indexOf('>', i);
      if (end === -1) {
        tokens.push(src.slice(i));
        break;
      }
      tokens.push(src.slice(i, end + 1));
      i = end + 1;
    } else {
      const next = src.indexOf('<', i);
      const text = (next === -1 ? src.slice(i) : src.slice(i, next)).trim();
      if (text) tokens.push(text);
      i = next === -1 ? src.length : next;
    }
  }
  const out: string[] = [];
  let depth = 0;
  // void 元素与自闭合
  const voidTags = new Set([
    'area',
    'base',
    'br',
    'col',
    'embed',
    'hr',
    'img',
    'input',
    'link',
    'meta',
    'param',
    'source',
    'track',
    'wbr',
  ]);
  for (const tok of tokens) {
    if (tok.startsWith('</')) {
      depth = Math.max(0, depth - 1);
      out.push(ind.repeat(depth) + tok);
    } else if (tok.startsWith('<')) {
      const m = /^<([a-zA-Z][\w-]*)/.exec(tok);
      const tag = m ? m[1]!.toLowerCase() : '';
      out.push(ind.repeat(depth) + tok);
      if (
        !tok.endsWith('/>') &&
        !voidTags.has(tag) &&
        !tok.startsWith('<?') &&
        !tok.startsWith('<!')
      ) {
        depth++;
      }
    } else {
      // 文本节点
      out.push(ind.repeat(depth) + tok);
    }
  }
  return out.join('\n') + '\n';
}

function minifyMarkup(src: string): string {
  return src
    .replace(/<!--[\s\S]*?-->/g, '') // HTML 注释
    .replace(/>\s+</g, '><')
    .replace(/\s{2,}/g, ' ')
    .trim();
}

// ===== JavaScript(轻量美化:基于括号/分号缩进,非完整解析器) =====
function beautifyJs(src: string): string {
  const ind = indentStr();
  let s = src.replace(/\r\n/g, '\n');
  // 移除单行/块注释外的多余空格,保留字符串字面量
  // 先按字符扫描,避免破坏字符串内的 { } ;
  let depth = 0;
  let out = '';
  let i = 0;
  let atLineStart = true;
  let inString: string | null = null;
  let inLineComment = false;
  let inBlockComment = false;

  const newlineIndent = () => '\n' + ind.repeat(Math.max(0, depth));

  while (i < s.length) {
    const ch = s[i];
    const two = s.slice(i, i + 2);

    if (inLineComment) {
      out += ch;
      if (ch === '\n') {
        inLineComment = false;
        atLineStart = true;
      }
      i++;
      continue;
    }
    if (inBlockComment) {
      out += ch;
      if (two === '*/') {
        out += s[i + 1] ?? '';
        i += 2;
        inBlockComment = false;
      } else {
        i++;
      }
      continue;
    }
    if (inString) {
      out += ch;
      if (ch === '\\') {
        out += s[i + 1] ?? '';
        i += 2;
        continue;
      }
      if (ch === inString) inString = null;
      i++;
      continue;
    }

    // 字符串/注释起始判断
    if (ch === '"' || ch === "'" || ch === '`') {
      inString = ch;
      out += ch;
      atLineStart = false;
      i++;
      continue;
    }
    if (two === '//') {
      inLineComment = true;
      out += two;
      i += 2;
      continue;
    }
    if (two === '/*') {
      inBlockComment = true;
      out += two;
      i += 2;
      continue;
    }

    // 处理缩进点
    if (ch === '{') {
      out += ' {';
      depth++;
      out += newlineIndent();
      atLineStart = true;
      i++;
      // 跳过后续空白
      while (i < s.length && /\s/.test(s[i] ?? '')) i++;
      continue;
    }
    if (ch === '}') {
      depth = Math.max(0, depth - 1);
      if (!atLineStart) out = out.replace(/[ \t]*$/, '');
      if (!out.endsWith('\n')) out += newlineIndent();
      out += '}';
      atLineStart = false;
      i++;
      continue;
    }
    if (ch === ';') {
      out += ';';
      // 换行(除非后面紧跟 })
      let j = i + 1;
      while (j < s.length && /[ \t]/.test(s[j] ?? '')) j++;
      if (j < s.length && s[j] !== '}' && s[j] !== ';') {
        out += newlineIndent();
        atLineStart = true;
      }
      i++;
      while (i < s.length && /[ \t]/.test(s[i] ?? '')) i++;
      continue;
    }
    if (ch === '\n') {
      // 折叠多余空行
      if (!out.endsWith('\n')) {
        out += newlineIndent();
        atLineStart = true;
      }
      i++;
      while (i < s.length && /[ \t\n]/.test(s[i] ?? '')) i++;
      continue;
    }
    if (/[ \t]/.test(ch ?? '')) {
      // 折叠连续空白,但保留单词间一个空格
      if (!atLineStart && out && !/\s$/.test(out)) out += ' ';
      i++;
      while (i < s.length && /[ \t]/.test(s[i] ?? '')) i++;
      continue;
    }

    out += ch;
    atLineStart = false;
    i++;
  }
  return out.replace(/\n{3,}/g, '\n\n').trim() + '\n';
}

function minifyJs(src: string): string {
  let out = '';
  let i = 0;
  let inString: string | null = null;
  let inLineComment = false;
  let inBlockComment = false;
  while (i < src.length) {
    const ch = src[i];
    const two = src.slice(i, i + 2);
    if (inLineComment) {
      if (ch === '\n') {
        inLineComment = false;
        out += '\n';
      }
      i++;
      continue;
    }
    if (inBlockComment) {
      if (two === '*/') {
        i += 2;
        inBlockComment = false;
      } else i++;
      continue;
    }
    if (inString) {
      out += ch;
      if (ch === '\\') {
        out += src[i + 1] ?? '';
        i += 2;
        continue;
      }
      if (ch === inString) inString = null;
      i++;
      continue;
    }
    if (ch === '"' || ch === "'" || ch === '`') {
      inString = ch;
      out += ch;
      i++;
      continue;
    }
    if (two === '//') {
      inLineComment = true;
      i += 2;
      continue;
    }
    if (two === '/*') {
      inBlockComment = true;
      i += 2;
      continue;
    }
    out += ch;
    i++;
  }
  // 折叠行间多余空白
  return out
    .replace(/\s{2,}/g, ' ')
    .replace(/\s*([{}();,=+\-*/<>!&|?:])\s*/g, '$1')
    .replace(/\n/g, '')
    .trim();
}

function loadSample() {
  const samples: Record<Lang, string> = {
    xml: '<root><user id="1"><name>张三</name><age>18</age></user><user id="2"><name>李四</name></user></root>',
    html: '<!DOCTYPE html><html><head><title>Test</title></head><body><h1>Hello</h1><p>World</p></body></html>',
    css: 'body{margin:0;padding:0;font-family:sans-serif}.btn{color:#fff;background:#007bff;padding:8px 16px;border:none;border-radius:4px;cursor:pointer}',
    js: 'function add(a,b){return a+b}const result=add(1,2);if(result>0){console.log("positive:"+result)}else{console.log("zero")}',
  };
  input.value = samples[lang.value];
}

async function doCopy() {
  if (output.value) await copyToClipboard(output.value, '已复制');
}

watch([input, lang, mode, indent], run, { immediate: true });
</script>

<style scoped>
.bar {
  display: flex;
  align-items: center;
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
.opt-group {
  margin-left: auto;
}
.opt {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
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
  align-items: center;
  gap: 8px;
}
.stat {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
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
