<template>
  <ToolShell
    title="HTTP 请求"
    :icon="Send"
    description="发送 HTTP 请求测试接口，支持 Auth、Headers、Query、Body，JSON 高亮响应。"
  >
    <!-- 请求行 -->
    <div class="req-row">
      <select v-model="method" class="method-select">
        <option v-for="m in METHODS" :key="m" :value="m">{{ m }}</option>
      </select>
      <input
        v-model="url"
        class="url-input"
        type="text"
        placeholder="https://httpbin.org/get"
        spellcheck="false"
        @keydown.enter="onSend"
      />
      <BaseButton variant="primary" :disabled="loading || !url" @click="onSend">
        <Loader2 v-if="loading" :size="14" class="spin" />
        {{ loading ? '发送中' : '发送' }}
      </BaseButton>
      <BaseButton variant="ghost" :disabled="!url" @click="exportCurl">cURL</BaseButton>
    </div>

    <!-- 请求 Tabs -->
    <div class="tabs">
      <button
        v-for="t in TABS"
        :key="t.id"
        class="tab"
        :class="{ active: activeTab === t.id }"
        @click="activeTab = t.id"
      >
        {{ t.label }}
        <span v-if="t.id === 'headers' && activeHeaders" class="badge">{{ activeHeaders }}</span>
        <span v-if="t.id === 'query' && activeQuery" class="badge">{{ activeQuery }}</span>
        <span v-if="t.id === 'body' && body" class="badge dot"></span>
        <span v-if="t.id === 'auth' && authType !== 'none'" class="badge dot"></span>
      </button>
      <div class="tab-extra">
        <label class="timeout-label">
          超时
          <input v-model.number="timeoutSec" type="number" min="0" class="timeout-input" />
          s
        </label>
      </div>
    </div>

    <!-- Tab 内容 -->
    <div class="tab-body">
      <!-- Headers / Query (共用 KV 编辑器) -->
      <div v-show="activeTab === 'headers' || activeTab === 'query'" class="kv-editor">
        <div v-for="(item, i) in activeTab === 'headers' ? headers : query" :key="i" class="kv-row">
          <input v-model="item.enabled" type="checkbox" class="kv-check" />
          <input
            v-model="item.key"
            class="kv-input"
            :placeholder="activeTab === 'headers' ? 'Header 名' : '参数名'"
            spellcheck="false"
          />
          <input v-model="item.value" class="kv-input" placeholder="值" spellcheck="false" />
          <button class="kv-del" @click="(activeTab === 'headers' ? headers : query).splice(i, 1)">
            <X :size="14" />
          </button>
        </div>
        <button
          class="kv-add"
          @click="
            (activeTab === 'headers' ? headers : query).push({ key: '', value: '', enabled: true })
          "
        >
          <Plus :size="13" />
          添加{{ activeTab === 'headers' ? ' Header' : '参数' }}
        </button>
      </div>

      <!-- Auth -->
      <div v-show="activeTab === 'auth'" class="auth-editor">
        <div class="seg">
          <button :class="{ active: authType === 'none' }" @click="authType = 'none'">无</button>
          <button :class="{ active: authType === 'bearer' }" @click="authType = 'bearer'">
            Bearer Token
          </button>
          <button :class="{ active: authType === 'basic' }" @click="authType = 'basic'">
            Basic Auth
          </button>
          <button :class="{ active: authType === 'apikey' }" @click="authType = 'apikey'">
            API Key
          </button>
        </div>
        <template v-if="authType === 'bearer'">
          <input
            v-model="authToken"
            class="auth-input"
            placeholder="Bearer Token 值"
            spellcheck="false"
          />
        </template>
        <template v-if="authType === 'basic'">
          <div class="auth-row">
            <input v-model="authUser" class="auth-input" placeholder="用户名" spellcheck="false" />
            <input v-model="authPass" class="auth-input" type="password" placeholder="密码" />
          </div>
        </template>
        <template v-if="authType === 'apikey'">
          <div class="auth-row">
            <input
              v-model="apiKeyHeader"
              class="auth-input"
              placeholder="Header 名 (如 X-API-Key)"
              spellcheck="false"
            />
            <input
              v-model="apiKeyValue"
              class="auth-input"
              placeholder="API Key 值"
              spellcheck="false"
            />
          </div>
        </template>
      </div>

      <!-- Body -->
      <div v-show="activeTab === 'body'">
        <div class="body-bar">
          <div class="seg">
            <button
              v-for="bt in BODY_TYPES"
              :key="bt"
              :class="{ active: bodyType === bt }"
              @click="bodyType = bt"
            >
              {{ bt.toUpperCase() }}
            </button>
          </div>
          <span v-if="bodyType === 'form'" class="body-hint">格式: key=value&key2=value2</span>
        </div>
        <textarea
          v-model="body"
          class="body-editor"
          :placeholder="bodyPlaceholder"
          spellcheck="false"
        ></textarea>
      </div>
    </div>

    <!-- 响应 -->
    <div v-if="response || errorMsg" class="response">
      <div class="resp-head">
        <template v-if="response">
          <span class="status-badge" :class="statusClass">
            {{ response.status }} {{ response.statusText }}
          </span>
          <span class="resp-meta">
            {{ response.elapsedMs }}ms · {{ formatSize(response.sizeBytes) }}
          </span>
        </template>
        <template v-else>
          <span class="status-badge err">ERROR</span>
        </template>
        <span style="flex: 1"></span>
        <label v-if="isJsonResponse" class="fmt-check">
          <input v-model="formatJson" type="checkbox" />
          格式化
        </label>
        <button v-if="response?.body" class="mini-btn" @click="copy(response.body)">
          <Copy :size="12" />
          复制
        </button>
      </div>

      <div v-if="errorMsg" class="resp-error">{{ errorMsg }}</div>

      <template v-if="response">
        <div class="resp-tabs">
          <button
            class="resp-tab"
            :class="{ active: respTab === 'body' }"
            @click="respTab = 'body'"
          >
            Body
          </button>
          <button
            class="resp-tab"
            :class="{ active: respTab === 'headers' }"
            @click="respTab = 'headers'"
          >
            Headers ({{ response.headers.length }})
          </button>
        </div>

        <div v-if="respTab === 'headers'" class="resp-headers">
          <div v-for="(h, i) in response.headers" :key="i" class="resp-header-row">
            <span class="rh-key">{{ h.key }}:</span>
            <span class="rh-val">{{ h.value }}</span>
          </div>
        </div>

        <pre
          v-if="respTab === 'body'"
          class="resp-body"
        ><code v-if="highlightedBody" class="hljs" v-html="highlightedBody"></code><template v-else>{{ response.body || '(空响应体)' }}</template></pre>
      </template>
    </div>

    <!-- 历史 -->
    <div v-if="history.length" class="history">
      <div class="history-head">
        <span class="section-label">历史 ({{ history.length }})</span>
        <button class="clear-hist" @click="history = []">清空</button>
      </div>
      <div class="history-list">
        <button v-for="(h, i) in history" :key="i" class="history-item" @click="loadHistory(h)">
          <span class="hist-status" :class="statusClassOf(h.status)">{{ h.status || 'ERR' }}</span>
          <span class="hist-method">{{ h.method }}</span>
          <span class="hist-url">{{ h.url }}</span>
          <span class="hist-time">{{ h.time }}</span>
        </button>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { Send, Plus, X, Copy, Loader2 } from '@lucide/vue';
import hljs from 'highlight.js/lib/core';
import jsonLang from 'highlight.js/lib/languages/json';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';
import {
  httpRequest,
  type HttpMethod,
  type BodyType,
  type KeyValue,
  type HttpResponseOutput,
} from '@/composables/useHttpTool';

hljs.registerLanguage('json', jsonLang);
const toast = useToast();

const METHODS: HttpMethod[] = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'];
const BODY_TYPES: BodyType[] = ['json', 'form', 'raw'];
const TABS = [
  { id: 'headers' as const, label: 'Headers' },
  { id: 'query' as const, label: 'Query' },
  { id: 'auth' as const, label: 'Auth' },
  { id: 'body' as const, label: 'Body' },
];

const method = useToolState<HttpMethod>('http', 'method', 'GET');
const url = useToolState('http', 'url', '');
const activeTab = ref<'headers' | 'query' | 'auth' | 'body'>('headers');
const bodyType = useToolState<BodyType>('http', 'bodyType', 'json');
const timeoutSec = useToolState('http', 'timeout', 30);
const headers = useToolState<KeyValue[]>('http', 'headers', [
  { key: '', value: '', enabled: true },
]);
const query = useToolState<KeyValue[]>('http', 'query', [{ key: '', value: '', enabled: true }]);
const body = useToolState('http', 'body', '');

const authType = useToolState<'none' | 'bearer' | 'basic' | 'apikey'>('http', 'authType', 'none');
const authToken = useToolState('http', 'authToken', '');
const authUser = useToolState('http', 'authUser', '');
const authPass = useToolState('http', 'authPass', '');
const apiKeyHeader = useToolState('http', 'apiKeyHeader', 'X-API-Key');
const apiKeyValue = useToolState('http', 'apiKeyValue', '');

const loading = ref(false);
const response = ref<HttpResponseOutput | null>(null);
const errorMsg = ref('');
const respTab = ref<'body' | 'headers'>('body');
const formatJson = ref(true);

interface HistoryItem {
  method: HttpMethod;
  url: string;
  status: number;
  time: string;
}
const history = useToolState<HistoryItem[]>('http', 'history', []);

const activeHeaders = computed(
  () => headers.value.filter((h) => h.enabled !== false && h.key.trim()).length,
);
const activeQuery = computed(
  () => query.value.filter((q) => q.enabled !== false && q.key.trim()).length,
);

const authHeaders = computed<KeyValue[]>(() => {
  switch (authType.value) {
    case 'bearer':
      return authToken.value
        ? [{ key: 'Authorization', value: `Bearer ${authToken.value}`, enabled: true }]
        : [];
    case 'basic':
      if (!authUser.value && !authPass.value) return [];
      return [
        {
          key: 'Authorization',
          value: `Basic ${btoa(`${authUser.value}:${authPass.value}`)}`,
          enabled: true,
        },
      ];
    case 'apikey':
      return apiKeyValue.value
        ? [{ key: apiKeyHeader.value, value: apiKeyValue.value, enabled: true }]
        : [];
    default:
      return [];
  }
});

const bodyPlaceholder = computed(() => {
  if (bodyType.value === 'json') return '{\n  "key": "value"\n}';
  if (bodyType.value === 'form') return 'key=value&key2=value2';
  return '原始请求体...';
});

const isJsonResponse = computed(() => {
  const b = response.value?.body;
  if (!b) return false;
  const t = b.trimStart();
  return t.startsWith('{') || t.startsWith('[');
});

const formattedBody = computed(() => {
  if (!response.value?.body) return '';
  if (!isJsonResponse.value || !formatJson.value) return response.value.body;
  try {
    return JSON.stringify(JSON.parse(response.value.body), null, 2);
  } catch {
    return response.value.body;
  }
});

const highlightedBody = computed(() => {
  if (!isJsonResponse.value || !formatJson.value || !formattedBody.value) return '';
  try {
    return hljs.highlight(formattedBody.value, { language: 'json' }).value;
  } catch {
    return '';
  }
});

const statusClass = computed(() => {
  const s = response.value?.status ?? 0;
  if (s === 0) return 'err';
  if (s < 300) return 'ok';
  if (s < 400) return 'warn';
  return 'err';
});

function statusClassOf(s: number) {
  if (s === 0) return 'err';
  if (s < 300) return 'ok';
  if (s < 400) return 'warn';
  return 'err';
}

async function onSend() {
  if (!url.value.trim()) return;
  loading.value = true;
  errorMsg.value = '';
  response.value = null;
  respTab.value = 'body';
  try {
    const res = await httpRequest({
      method: method.value,
      url: url.value,
      headers: [...headers.value, ...authHeaders.value],
      query: query.value,
      body: body.value || null,
      bodyType: bodyType.value,
      timeoutMs: timeoutSec.value > 0 ? timeoutSec.value * 1000 : null,
    });
    response.value = res;
    if (res.error) {
      errorMsg.value = res.error;
      toast.error(res.error);
    }
    history.value = [
      {
        method: method.value,
        url: url.value,
        status: res.status,
        time: new Date().toLocaleTimeString('zh-CN', { hour12: false }),
      },
      ...history.value.filter((h) => h.url !== url.value || h.method !== method.value),
    ].slice(0, 15);
  } catch (e) {
    errorMsg.value = String(e);
    toast.error('请求失败');
  } finally {
    loading.value = false;
  }
}

function loadHistory(h: HistoryItem) {
  method.value = h.method;
  url.value = h.url;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

async function copy(text: string) {
  await copyToClipboard(text, '已复制响应体');
}

function genCurl(): string {
  const parts: string[] = [`curl -X ${method.value}`];
  let fullUrl = url.value;
  const aq = query.value.filter((q) => q.enabled !== false && q.key.trim());
  if (aq.length) {
    const qs = aq
      .map((q) => `${encodeURIComponent(q.key)}=${encodeURIComponent(q.value)}`)
      .join('&');
    fullUrl += (fullUrl.includes('?') ? '&' : '?') + qs;
  }
  parts.push(`'${fullUrl}'`);
  const allH = [
    ...headers.value.filter((h) => h.enabled !== false && h.key.trim()),
    ...authHeaders.value,
  ];
  for (const h of allH) parts.push(`-H '${h.key}: ${h.value}'`);
  if (body.value && ['POST', 'PUT', 'PATCH'].includes(method.value)) {
    parts.push(`-d '${body.value.replace(/'/g, "'\\''")}'`);
  }
  return parts.join(' \\\n  ');
}

async function exportCurl() {
  await copyToClipboard(genCurl(), '已复制 cURL 命令');
}
</script>

<style scoped>
.req-row {
  display: flex;
  gap: 8px;
  margin-bottom: 14px;
  flex-wrap: wrap;
}
.method-select {
  width: 110px;
  padding: 0 10px;
  font-size: 13px;
  font-weight: 700;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-accent);
  cursor: pointer;
  outline: none;
}
.url-input {
  flex: 1;
  min-width: 200px;
  padding: 9px 14px;
  font-size: 13px;
  font-family: var(--xuya-font-mono);
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.url-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.url-input::placeholder {
  color: var(--xuya-text-tertiary);
}

.tabs {
  display: flex;
  gap: 4px;
  border-bottom: 1px solid var(--xuya-border);
  margin-bottom: 14px;
  align-items: center;
  flex-wrap: wrap;
}
.tab {
  padding: 8px 14px;
  font-size: 13px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  transition:
    color var(--xuya-duration-fast),
    border-color var(--xuya-duration-fast);
  cursor: pointer;
}
.tab:hover {
  color: var(--xuya-text);
}
.tab.active {
  color: var(--xuya-accent);
  border-bottom-color: var(--xuya-accent);
  font-weight: 600;
}
.tab .badge {
  font-size: 10px;
  padding: 1px 6px;
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent);
  border-radius: 99px;
}
.tab .badge.dot {
  width: 6px;
  height: 6px;
  padding: 0;
  border-radius: 50%;
}
.tab-extra {
  margin-left: auto;
  padding-bottom: 6px;
}
.timeout-label {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.timeout-input {
  width: 48px;
  padding: 3px 6px;
  font-size: 12px;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
}

.tab-body {
  min-height: 100px;
  margin-bottom: 18px;
}
.kv-editor {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.kv-row {
  display: flex;
  gap: 6px;
  align-items: center;
}
.kv-check {
  accent-color: var(--xuya-accent);
  width: 15px;
  height: 15px;
  flex-shrink: 0;
}
.kv-input {
  flex: 1;
  min-width: 0;
  padding: 6px 10px;
  font-size: 12.5px;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  transition: border-color var(--xuya-duration-fast);
}
.kv-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
}
.kv-del {
  width: 28px;
  height: 28px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text-tertiary);
  background: transparent;
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.kv-del:hover {
  color: var(--xuya-danger);
  background: var(--xuya-danger-soft);
}
.kv-add {
  align-self: flex-start;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 5px 12px;
  font-size: 12px;
  color: var(--xuya-accent);
  border: 1px dashed var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  margin-top: 4px;
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
  background: transparent;
}
.kv-add:hover {
  border-color: var(--xuya-accent);
}

.auth-editor {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.auth-input {
  width: 100%;
  padding: 7px 12px;
  font-size: 13px;
  font-family: var(--xuya-font-mono);
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  outline: none;
  transition: border-color var(--xuya-duration-fast);
}
.auth-input:focus {
  border-color: var(--xuya-accent);
}
.auth-row {
  display: flex;
  gap: 8px;
}

.seg {
  display: inline-flex;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  padding: 2px;
  gap: 2px;
}
.seg button {
  padding: 4px 12px;
  font-size: 11px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-radius: var(--xuya-radius-sm);
  transition: all var(--xuya-duration-fast);
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
}

.body-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}
.body-hint {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
}
.body-editor {
  width: 100%;
  min-height: 120px;
  padding: 10px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.6;
  resize: vertical;
}
.body-editor:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

.response {
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-lg);
  padding: 16px;
}
.resp-head {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}
.status-badge {
  padding: 4px 12px;
  font-size: 13px;
  font-weight: 700;
  border-radius: var(--xuya-radius-sm);
  font-family: var(--xuya-font-mono);
}
.status-badge.ok {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.status-badge.warn {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}
.status-badge.err {
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
}
.resp-meta {
  font-size: 12px;
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
}
.fmt-check {
  font-size: 11px;
  color: var(--xuya-text-secondary);
  display: inline-flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
}
.fmt-check input {
  accent-color: var(--xuya-accent);
  width: 13px;
  height: 13px;
}
.mini-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  font-size: 11px;
  color: var(--xuya-text-secondary);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.mini-btn:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}
.resp-error {
  padding: 10px 14px;
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
  border-radius: var(--xuya-radius-sm);
  font-size: 12.5px;
  margin-bottom: 12px;
}

.resp-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 10px;
}
.resp-tab {
  font-size: 12px;
  color: var(--xuya-text-secondary);
  padding: 3px 0;
  background: none;
  border: none;
  cursor: pointer;
  transition: color var(--xuya-duration-fast);
}
.resp-tab:hover {
  color: var(--xuya-text);
}
.resp-tab.active {
  color: var(--xuya-accent);
  font-weight: 600;
}
.resp-headers {
  max-height: 200px;
  overflow: auto;
  margin-bottom: 12px;
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
}
.resp-header-row {
  display: flex;
  gap: 8px;
  padding: 5px 12px;
  font-size: 11.5px;
  border-bottom: 1px solid var(--xuya-border-light);
}
.resp-header-row:last-child {
  border-bottom: none;
}
.rh-key {
  color: var(--xuya-accent);
  font-family: var(--xuya-font-mono);
  flex-shrink: 0;
}
.rh-val {
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  word-break: break-all;
}
.resp-body {
  margin: 0;
  padding: 14px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.6;
  max-height: 400px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--xuya-text);
}
.resp-body :deep(.hljs-attr) {
  color: var(--xuya-syn-key);
}
.resp-body :deep(.hljs-string) {
  color: var(--xuya-syn-str);
}
.resp-body :deep(.hljs-number) {
  color: var(--xuya-syn-num);
}
.resp-body :deep(.hljs-literal) {
  color: var(--xuya-syn-bool);
}
.resp-body :deep(.hljs-keyword) {
  color: var(--xuya-syn-null);
}

.history {
  margin-top: 16px;
}
.history-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}
.section-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
}
.clear-hist {
  font-size: 10px;
  color: var(--xuya-text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
}
.clear-hist:hover {
  color: var(--xuya-danger);
}
.history-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.history-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 10px;
  font-size: 12px;
  border-radius: var(--xuya-radius-sm);
  transition: all var(--xuya-duration-fast);
  text-align: left;
  width: 100%;
  cursor: pointer;
  background: transparent;
  border: 1px solid transparent;
}
.history-item:hover {
  background: var(--xuya-input-bg);
  border-color: var(--xuya-border);
}
.hist-status {
  font-family: var(--xuya-font-mono);
  font-size: 11px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: var(--xuya-radius-sm);
  min-width: 36px;
  text-align: center;
  flex-shrink: 0;
}
.hist-status.ok {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.hist-status.warn {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}
.hist-status.err {
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
}
.hist-method {
  font-weight: 700;
  color: var(--xuya-accent);
  min-width: 56px;
  flex-shrink: 0;
}
.hist-url {
  flex: 1;
  min-width: 0;
  color: var(--xuya-text-secondary);
  font-family: var(--xuya-font-mono);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.hist-time {
  color: var(--xuya-text-tertiary);
  font-size: 11px;
  flex-shrink: 0;
}

.spin {
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 600px) {
  .method-select {
    width: 90px;
  }
}
</style>
