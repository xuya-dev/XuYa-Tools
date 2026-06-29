<template>
  <ToolShell title="SQL 格式化" :icon="Database" description="美化 / 压缩 SQL,关键字大写,支持多种方言。">
    <div class="sql-toolbar">
      <BaseButton variant="primary" @click="format">美化</BaseButton>
      <BaseButton @click="minify">压缩</BaseButton>
      <span class="sep"></span>
      <div class="seg">
        <button :class="{ active: dialect === 'mysql' }" @click="dialect = 'mysql'">MySQL</button>
        <button :class="{ active: dialect === 'postgres' }" @click="dialect = 'postgres'">PostgreSQL</button>
        <button :class="{ active: dialect === 'standard' }" @click="dialect = 'standard'">标准</button>
      </div>
      <label class="check"><input v-model="uppercase" type="checkbox" /> 关键字大写</label>
      <span style="flex:1"></span>
      <BaseButton variant="ghost" :disabled="!output" @click="copy"><Copy :size="13" /> 复制</BaseButton>
      <BaseButton variant="ghost" @click="clearAll">清空</BaseButton>
    </div>

    <div class="sql-grid">
      <div class="sql-col">
        <div class="col-head"><span>输入 SQL</span><span class="stat">{{ input.length }} 字符</span></div>
        <textarea v-model="input" class="editor" placeholder="SELECT * FROM users WHERE id = 1" spellcheck="false"></textarea>
      </div>
      <div class="sql-col">
        <div class="col-head">
          <span>输出</span>
          <span class="stat" :class="{ err: errorMsg }">{{ errorMsg ? '错误' : (output ? output.length + ' 字符' : '') }}</span>
        </div>
        <pre class="output" :class="{ 'has-error': errorMsg }"><code>{{ errorMsg || output || '点击「美化」查看格式化结果' }}</code></pre>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Database, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const input = useToolState('sql', 'input', 'select u.id, u.name, p.title from users u join posts p on u.id = p.user_id where u.active = 1 and p.published = 1 order by p.created_at desc');
const output = ref('');
const errorMsg = ref('');
const dialect = useToolState<'mysql' | 'postgres' | 'standard'>('sql', 'dialect', 'mysql');
const uppercase = useToolState('sql', 'uppercase', true);

/** 简易 SQL 格式化器 (无依赖, 关键字高亮 + 缩进) */
const KEYWORDS = [
  'SELECT', 'FROM', 'WHERE', 'INSERT', 'INTO', 'VALUES', 'UPDATE', 'SET', 'DELETE',
  'JOIN', 'LEFT', 'RIGHT', 'INNER', 'OUTER', 'FULL', 'CROSS', 'ON', 'USING',
  'GROUP', 'BY', 'ORDER', 'HAVING', 'LIMIT', 'OFFSET', 'UNION', 'ALL', 'DISTINCT',
  'AS', 'AND', 'OR', 'NOT', 'IN', 'NOT IN', 'EXISTS', 'BETWEEN', 'LIKE', 'IS', 'NULL',
  'CREATE', 'TABLE', 'INDEX', 'VIEW', 'DATABASE', 'DROP', 'ALTER', 'ADD', 'COLUMN',
  'PRIMARY', 'KEY', 'FOREIGN', 'REFERENCES', 'CONSTRAINT', 'UNIQUE', 'DEFAULT',
  'CASE', 'WHEN', 'THEN', 'ELSE', 'END', 'IF', 'BEGIN', 'COMMIT', 'ROLLBACK',
  'WITH', 'RECURSIVE', 'RETURNING', 'CONFLICT', 'DO', 'NOTHING', 'ASC', 'DESC',
  'COUNT', 'SUM', 'AVG', 'MIN', 'MAX', 'CAST', 'CONVERT',
];
const KEYWORD_SET = new Set(KEYWORDS);

function formatSql(sql: string, upper: boolean): string {
  // 1. 规范化空白
  let s = sql.replace(/\s+/g, ' ').trim();
  if (!s) return '';

  // 2. 在关键字前后确保空格 (基于 token)
  // 简单分词: 引号字符串、注释、标识符、操作符、关键字
  const tokens = tokenize(s);
  let out = '';
  let indent = 0;
  let afterSelect = false;
  let prevType = '';

  const newLine = () => '\n' + '  '.repeat(Math.max(0, indent));

  for (let i = 0; i < tokens.length; i++) {
    const t = tokens[i]!;
    const upperT = t.toUpperCase();
    const isKw = KEYWORD_SET.has(upperT);

    if (isKw) {
      const kw = upper ? upperT : t.toLowerCase();
      // 主子句换行 + 重置缩进
      if (['SELECT', 'FROM', 'WHERE', 'INSERT', 'UPDATE', 'DELETE', 'CREATE', 'WITH', 'ORDER', 'GROUP', 'HAVING', 'LIMIT', 'UNION', 'VALUES', 'RETURNING'].includes(upperT)) {
        if (out) out = out.replace(/,\s*$/, ',');
        if (out && !out.endsWith('\n')) out += newLine();
        indent = upperT === 'SELECT' || upperT === 'WITH' ? 1 : (upperT === 'FROM' ? 0 : 0);
        out += kw + ' ';
        afterSelect = upperT === 'SELECT';
        prevType = 'clause';
        continue;
      }
      if (upperT === 'JOIN' || upperT === 'LEFT' || upperT === 'RIGHT' || upperT === 'INNER' || upperT === 'CROSS' || upperT === 'FULL') {
        if (out && !out.endsWith('\n')) out += newLine();
        out += kw + ' ';
        prevType = 'join';
        continue;
      }
      if (upperT === 'AND' || upperT === 'OR') {
        out = out.replace(/\s+$/, '');
        out += '\n' + '  '.repeat(Math.max(1, indent)) + kw + ' ';
        prevType = 'and';
        continue;
      }
      if (upperT === 'ON') {
        out = out.replace(/\s+$/, '');
        out += ' ' + kw + ' ';
        prevType = 'on';
        continue;
      }
      // 其他关键字
      out += (prevType === 'ident' || prevType === 'value' || prevType === 'paren-close' ? ' ' : '') + kw + ' ';
      prevType = 'kw';
      continue;
    }

    // 逗号: SELECT 字段列表换行
    if (t === ',') {
      if (afterSelect) {
        out = out.replace(/\s+$/, '');
        out += ',' + '\n' + '  '.repeat(indent > 0 ? 1 : 0) + '  ';
      } else {
        out = out.replace(/\s+$/, '');
        out += ', ';
      }
      prevType = 'comma';
      continue;
    }

    // 括号
    if (t === '(') {
      out = out.replace(/\s+$/, '');
      out += '(';
      prevType = 'paren-open';
      continue;
    }
    if (t === ')') {
      out = out.replace(/,\s*$/, '');
      out = out.replace(/\s+$/, '');
      out += ') ';
      prevType = 'paren-close';
      continue;
    }

    // 分号
    if (t === ';') {
      out = out.replace(/\s+$/, '');
      out += ';\n';
      indent = 0;
      afterSelect = false;
      prevType = 'semicolon';
      continue;
    }

    // 标识符 / 值 / 操作符
    const needSpace = prevType && !['paren-open'].includes(prevType) && !['(', '.'].includes(tokens[i - 1] || '');
    out += (needSpace ? ' ' : '') + t;
    prevType = /['"`]/.test(t[0] || '') ? 'value' : 'ident';
  }

  // 清理多余空格/空行
  return out.replace(/[ \t]+\n/g, '\n').replace(/\n{3,}/g, '\n\n').replace(/\s+$/, '\n').replace(/^\s+/, '');
}

/** 简易分词 */
function tokenize(sql: string): string[] {
  const tokens: string[] = [];
  let i = 0;
  while (i < sql.length) {
    const c = sql[i]!;
    // 空白
    if (/\s/.test(c)) { i++; continue; }
    // 字符串 (单引号/双引号/反引号)
    if (c === "'" || c === '"' || c === '`') {
      let j = i + 1;
      while (j < sql.length && sql[j] !== c) { if (sql[j] === '\\') j++; j++; }
      tokens.push(sql.slice(i, Math.min(j + 1, sql.length)));
      i = j + 1;
      continue;
    }
    // 注释 -- 或 //
    if ((c === '-' && sql[i + 1] === '-') || (c === '/' && sql[i + 1] === '/')) {
      let j = i;
      while (j < sql.length && sql[j] !== '\n') j++;
      i = j;
      continue;
    }
    // 标识符 / 数字 / 关键字
    if (/[a-zA-Z_0-9.]/.test(c)) {
      let j = i;
      while (j < sql.length && /[a-zA-Z_0-9.]/.test(sql[j]!)) j++;
      tokens.push(sql.slice(i, j));
      i = j;
      continue;
    }
    // 操作符 / 标点
    if (/[<>=!]+/.test(c)) {
      let j = i;
      while (j < sql.length && /[<>=!]/.test(sql[j]!)) j++;
      tokens.push(sql.slice(i, j));
      i = j;
      continue;
    }
    // 单字符
    tokens.push(c);
    i++;
  }
  return tokens;
}

function format() {
  errorMsg.value = '';
  if (!input.value.trim()) return;
  try {
    output.value = formatSql(input.value, uppercase.value);
  } catch (e) {
    errorMsg.value = '格式化失败: ' + e;
  }
}

function minify() {
  errorMsg.value = '';
  if (!input.value.trim()) return;
  output.value = input.value.replace(/\s+/g, ' ').trim();
}

function clearAll() {
  input.value = '';
  output.value = '';
  errorMsg.value = '';
}

async function copy() {
  if (output.value) await copyToClipboard(output.value, '已复制 SQL');
}
</script>

<style scoped>
.sql-toolbar { display: flex; align-items: center; gap: 8px; margin-bottom: 14px; flex-wrap: wrap; }
.sep { width: 1px; height: 20px; background: var(--xuya-border); margin: 0 2px; }
.seg { display: inline-flex; background: var(--xuya-input-bg); border-radius: var(--xuya-radius-sm); padding: 2px; gap: 2px; }
.seg button { padding: 4px 12px; font-size: 12px; color: var(--xuya-text-secondary); background: transparent; border: none; border-radius: var(--xuya-radius-sm); transition: var(--xuya-duration-fast); }
.seg button.active { background: var(--xuya-bg-elevated); color: var(--xuya-accent); font-weight: 600; box-shadow: var(--xuya-shadow-sm); }
.check { display: inline-flex; align-items: center; gap: 6px; font-size: 12px; color: var(--xuya-text-secondary); cursor: pointer; }
.check input { accent-color: var(--xuya-accent); width: 15px; height: 15px; }

.sql-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; flex: 1; min-height: 0; }
.sql-col { display: flex; flex-direction: column; gap: 6px; min-height: 0; }
.col-head { display: flex; justify-content: space-between; font-size: 12px; font-weight: 600; color: var(--xuya-text-secondary); }
.stat { font-size: 10.5px; font-weight: 400; opacity: .75; }
.stat.err { color: var(--xuya-danger); opacity: 1; }
.editor { flex: 1; min-height: 280px; padding: 12px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); font-family: var(--xuya-font-mono); font-size: 12.5px; line-height: 1.6; resize: none; transition: border-color var(--xuya-duration-fast), box-shadow var(--xuya-duration-fast); }
.editor:focus { outline: none; border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }
.editor::placeholder { color: var(--xuya-text-tertiary); }
.output { flex: 1; min-height: 280px; margin: 0; padding: 12px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); overflow: auto; font-family: var(--xuya-font-mono); font-size: 12.5px; line-height: 1.6; white-space: pre-wrap; word-break: break-all; color: var(--xuya-text); }
.output.has-error code { color: var(--xuya-danger); }
</style>
