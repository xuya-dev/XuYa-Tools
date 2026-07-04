<template>
  <ToolShell
    title="敏感信息脱敏"
    :icon="EyeOff"
    description="一键打码文本中的手机号、身份证、邮箱、银行卡、IP 等敏感信息,支持自定义规则。"
  >
    <!-- 规则开关 -->
    <div class="rules-bar">
      <label v-for="r in rules" :key="r.id" class="rule-chip" :class="{ on: r.enabled.value }">
        <input v-model="r.enabled.value" type="checkbox" />
        <span class="rule-name">{{ r.label }}</span>
        <code class="rule-sample">{{ r.sample }}</code>
      </label>
    </div>

    <div class="grid">
      <!-- 输入 -->
      <div class="col">
        <div class="col-head">
          <span>原始文本</span>
          <div class="col-actions">
            <button class="mini-btn" @click="clearAll">清空</button>
          </div>
        </div>
        <textarea
          v-model="input"
          class="editor"
          placeholder="粘贴含敏感信息的文本,如日志、客户数据、文档…"
          spellcheck="false"
        ></textarea>
      </div>

      <!-- 输出 -->
      <div class="col">
        <div class="col-head">
          <span>脱敏结果</span>
          <div class="col-actions">
            <span v-if="redactCount > 0" class="count-badge">已打码 {{ redactCount }} 处</span>
            <button class="mini-btn" :disabled="!output" @click="doCopy">
              <Copy :size="12" />
              复制
            </button>
          </div>
        </div>
        <pre class="output"><code>{{ output || '—' }}</code></pre>
      </div>
    </div>

    <!-- 自定义规则 -->
    <details class="custom">
      <summary>自定义正则规则(高级)</summary>
      <div class="custom-body">
        <div v-for="(c, i) in customs" :key="i" class="custom-row">
          <input
            v-model="c.pattern"
            class="c-input"
            placeholder="正则表达式,如 \d{4}"
            spellcheck="false"
          />
          <input v-model="c.replacement" class="c-input c-repl" placeholder="替换文本,如 ****" />
          <button class="mini-btn danger" @click="customs.splice(i, 1)">删除</button>
        </div>
        <button class="mini-btn" @click="customs.push({ pattern: '', replacement: '****' })">
          + 添加规则
        </button>
      </div>
    </details>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { EyeOff, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const input = useToolState('redact', 'input', '');

// 各规则开关(持久化)
const enPhone = useToolState('redact', 'phone', true);
const enIdCard = useToolState('redact', 'idcard', true);
const enEmail = useToolState('redact', 'email', true);
const enBank = useToolState('redact', 'bank', true);
const enIp = useToolState('redact', 'ip', true);
const customs = useToolState<{ pattern: string; replacement: string }[]>('redact', 'customs', []);

const rules = [
  { id: 'phone', label: '手机号', sample: '138****5678', enabled: enPhone },
  { id: 'idcard', label: '身份证', sample: '110***********1234', enabled: enIdCard },
  { id: 'email', label: '邮箱', sample: 'z***@example.com', enabled: enEmail },
  { id: 'bank', label: '银行卡', sample: '6222****1234', enabled: enBank },
  { id: 'ip', label: 'IPv4', sample: '192.168.*.*', enabled: enIp },
];

const output = ref('');
const redactCount = ref(0);

function run() {
  const text = input.value;
  if (!text.trim()) {
    output.value = '';
    redactCount.value = 0;
    return;
  }
  let result = text;
  let count = 0;
  const apply = (re: RegExp, fn: (m: string) => string) => {
    result = result.replace(re, (m) => {
      count++;
      return fn(m);
    });
  };

  if (enPhone.value) {
    // 11 位手机号:1[3-9]xxxxxxxxx
    apply(/1[3-9]\d{9}/g, (m) => m.slice(0, 3) + '****' + m.slice(7));
  }
  if (enIdCard.value) {
    // 18 位身份证(末位 X),保留前 3 后 4
    apply(/\b\d{17}[\dXx]\b/g, (m) => m.slice(0, 3) + '*'.repeat(11) + m.slice(14));
    // 15 位老身份证
    apply(/\b\d{15}\b/g, (m) => m.slice(0, 3) + '*'.repeat(8) + m.slice(11));
  }
  if (enEmail.value) {
    // 邮箱:保留首字符与域名
    apply(/[\w.+-]+@[\w-]+\.[\w.-]+/g, (m) => {
      const at = m.indexOf('@');
      const name = m.slice(0, at);
      const domain = m.slice(at);
      if (name.length <= 1) return '*' + domain;
      return name[0] + '*'.repeat(Math.max(2, name.length - 1)) + domain;
    });
  }
  if (enBank.value) {
    // 16~19 位卡号:保留前 4 后 4
    apply(/\b\d{16,19}\b/g, (m) => m.slice(0, 4) + '*'.repeat(m.length - 8) + m.slice(-4));
  }
  if (enIp.value) {
    // IPv4:保留前两段
    apply(/\b(?:\d{1,3}\.){3}\d{1,3}\b/g, (m) => {
      const parts = m.split('.');
      return `${parts[0]}.${parts[1]}.*.*`;
    });
  }

  // 自定义规则
  for (const c of customs.value) {
    if (!c.pattern.trim()) continue;
    try {
      const re = new RegExp(c.pattern, 'g');
      apply(re, () => c.replacement || '****');
    } catch {
      /* 非法正则跳过 */
    }
  }

  output.value = result;
  redactCount.value = count;
}

function clearAll() {
  input.value = '';
  output.value = '';
  redactCount.value = 0;
}

async function doCopy() {
  if (output.value) await copyToClipboard(output.value, '已复制');
}

watch([input, enPhone, enIdCard, enEmail, enBank, enIp, customs], run, {
  immediate: true,
  deep: true,
});
</script>

<style scoped>
.rules-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 16px;
}
.rule-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  border-radius: var(--xuya-radius-sm);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  font-size: 12px;
  color: var(--xuya-text-secondary);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.rule-chip.on {
  border-color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent);
}
.rule-chip input[type='checkbox'] {
  accent-color: var(--xuya-accent);
}
.rule-name {
  font-weight: 600;
}
.rule-sample {
  font-family: var(--xuya-font-mono);
  font-size: 10.5px;
  opacity: 0.7;
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
  gap: 6px;
}
.count-badge {
  font-size: 11px;
  color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
  padding: 2px 8px;
  border-radius: var(--xuya-radius-sm);
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
.mini-btn.danger:hover {
  color: var(--xuya-danger);
  border-color: var(--xuya-danger);
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
  color: var(--xuya-text);
}

.custom {
  margin-top: 14px;
  padding: 10px 14px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  font-size: 12px;
}
.custom summary {
  cursor: pointer;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.custom-body {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.custom-row {
  display: flex;
  gap: 8px;
}
.c-input {
  flex: 1;
  padding: 6px 10px;
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text);
  outline: none;
}
.c-repl {
  flex: 0 0 120px;
}
.c-input:focus {
  border-color: var(--xuya-accent);
}
</style>
