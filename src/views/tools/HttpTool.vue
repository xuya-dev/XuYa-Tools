<template>
  <ToolShell title="HTTP 请求" :icon="Send" description="发送 HTTP 请求测试接口,支持 Headers / Query / Body,完整展示响应。">
    <!-- 请求行 -->
    <div class="req-row">
      <select v-model="method" class="method-select">
        <option v-for="m in METHODS" :key="m" :value="m">{{ m }}</option>
      </select>
      <input v-model="url" class="url-input" type="text" placeholder="https://httpbin.org/get" spellcheck="false" @keydown.enter="onSend" />
      <BaseButton variant="primary" :disabled="loading || !url" @click="onSend">
        <Loader2 v-if="loading" :size="14" class="spin" />
        {{ loading ? '发送中' : '发送' }}
      </BaseButton>
    </div>

    <!-- 请求 Tabs -->
    <div class="tabs">
      <button v-for="t in TABS" :key="t.id" class="tab" :class="{ active: activeTab === t.id }" @click="activeTab = t.id">
        {{ t.label }}
        <span v-if="t.id === 'headers' && activeHeaders" class="badge">{{ activeHeaders }}</span>
        <span v-if="t.id === 'query' && activeQuery" class="badge">{{ activeQuery }}</span>
        <span v-if="t.id === 'body' && body" class="badge dot"></span>
      </button>
      <div class="tab-extra">
        <label class="timeout-label">超时
          <input v-model.number="timeoutSec" type="number" min="0" class="timeout-input" />s
        </label>
      </div>
    </div>

    <!-- Tab 内容 -->
    <div class="tab-body">
      <!-- Headers -->
      <div v-show="activeTab === 'headers'" class="kv-editor">
        <div v-for="(h, i) in headers" :key="i" class="kv-row">
          <input v-model="h.enabled" type="checkbox" class="kv-check" />
          <input v-model="h.key" class="kv-input" placeholder="Header 名 (如 Content-Type)" spellcheck="false" />
          <input v-model="h.value" class="kv-input" placeholder="值" spellcheck="false" />
          <button class="kv-del" @click="headers.splice(i, 1)"><X :size="14" /></button>
        </div>
        <button class="kv-add" @click="headers.push({ key: '', value: '', enabled: true })"><Plus :size="13" /> 添加 Header</button>
      </div>

      <!-- Query -->
      <div v-show="activeTab === 'query'" class="kv-editor">
        <div v-for="(q, i) in query" :key="i" class="kv-row">
          <input v-model="q.enabled" type="checkbox" class="kv-check" />
          <input v-model="q.key" class="kv-input" placeholder="参数名 (如 page)" spellcheck="false" />
          <input v-model="q.value" class="kv-input" placeholder="值" spellcheck="false" />
          <button class="kv-del" @click="query.splice(i, 1)"><X :size="14" /></button>
        </div>
        <button class="kv-add" @click="query.push({ key: '', value: '', enabled: true })"><Plus :size="13" /> 添加参数</button>
      </div>

      <!-- Body -->
      <div v-show="activeTab === 'body'">
        <div class="body-bar">
          <div class="seg">
            <button v-for="bt in BODY_TYPES" :key="bt" :class="{ active: bodyType === bt }" @click="bodyType = bt">{{ bt.toUpperCase() }}</button>
          </div>
          <span v-if="bodyType === 'form'" class="body-hint">格式: key=value&key2=value2</span>
        </div>
        <textarea v-model="body" class="body-editor" :placeholder="bodyPlaceholder" spellcheck="false"></textarea>
      </div>
    </div>

    <!-- 响应 -->
    <div v-if="response || errorMsg" class="response">
      <div class="resp-head">
        <template v-if="response">
          <span class="status-badge" :class="statusClass">{{ response.status }} {{ response.statusText }}</span>
          <span class="resp-meta">{{ response.elapsedMs }}ms · {{ formatSize(response.sizeBytes) }}</span>
        </template>
        <template v-else>
          <span class="status-badge err">ERROR</span>
        </template>
        <span style="flex:1"></span>
        <button v-if="response?.body" class="mini-btn" @click="copy(response.body)"><Copy :size="12" /> 复制</button>
      </div>

      <div v-if="errorMsg" class="resp-error">{{ errorMsg }}</div>

      <template v-if="response">
        <!-- 响应 Tabs -->
        <div class="resp-tabs">
          <button class="resp-tab active">Body</button>
          <button class="resp-tab" @click="showHeaders = !showHeaders">{{ showHeaders ? '隐藏' : '查看' }} Headers ({{ response.headers.length }})</button>
        </div>

        <div v-if="showHeaders" class="resp-headers">
          <div v-for="(h, i) in response.headers" :key="i" class="resp-header-row">
            <span class="rh-key">{{ h.key }}:</span>
            <span class="rh-val">{{ h.value }}</span>
          </div>
        </div>

        <pre class="resp-body" :class="{ 'is-json': isJsonBody }"><code>{{ response.body || '(空响应体)' }}</code></pre>
      </template>
    </div>

    <!-- 历史 -->
    <div v-if="history.length" class="history">
      <div class="section-label">历史 ({{ history.length }})</div>
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
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';
import { httpRequest, type HttpMethod, type BodyType, type KeyValue, type HttpResponseOutput } from '@/composables/useHttpTool';

const toast = useToast();

const METHODS: HttpMethod[] = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'];
const BODY_TYPES: BodyType[] = ['json', 'form', 'raw'];
const TABS = [
  { id: 'headers' as const, label: 'Headers' },
  { id: 'query' as const, label: 'Query' },
  { id: 'body' as const, label: 'Body' },
];

const method = ref<HttpMethod>('GET');
const url = ref('https://httpbin.org/get');
const activeTab = ref<'headers' | 'query' | 'body'>('headers');
const bodyType = ref<BodyType>('json');
const timeoutSec = ref(30);

const headers = ref<KeyValue[]>([{ key: '', value: '', enabled: true }]);
const query = ref<KeyValue[]>([{ key: '', value: '', enabled: true }]);
const body = ref('');

const loading = ref(false);
const response = ref<HttpResponseOutput | null>(null);
const errorMsg = ref('');
const showHeaders = ref(false);

const activeHeaders = computed(() => headers.value.filter((h) => h.enabled !== false && h.key.trim()).length);
const activeQuery = computed(() => query.value.filter((q) => q.enabled !== false && q.key.trim()).length);

const bodyPlaceholder = computed(() => {
  if (bodyType.value === 'json') return '{\n  "key": "value"\n}';
  if (bodyType.value === 'form') return 'key=value&key2=value2';
  return '原始请求体...';
});

const isJsonBody = computed(() => response.value?.body?.trimStart().startsWith('{') || response.value?.body?.trimStart().startsWith('['));

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

interface HistoryItem { method: HttpMethod; url: string; status: number; time: string; }
const history = ref<HistoryItem[]>([]);

async function onSend() {
  if (!url.value.trim()) return;
  loading.value = true;
  errorMsg.value = '';
  response.value = null;
  try {
    const res = await httpRequest({
      method: method.value,
      url: url.value,
      headers: headers.value,
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
    history.value.unshift({
      method: method.value,
      url: url.value,
      status: res.status,
      time: new Date().toLocaleTimeString('zh-CN', { hour12: false }),
    });
    if (history.value.length > 10) history.value.pop();
  } catch (e) {
    errorMsg.value = String(e);
    toast.error('请求失败: ' + e);
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
</script>

<style scoped>
.req-row { display: flex; gap: 8px; margin-bottom: 14px; }
.method-select {
  width: 110px; padding: 0 10px; font-size: 13px; font-weight: 700;
  border-radius: var(--xuya-radius-sm); border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg); color: var(--xuya-accent); cursor: pointer; outline: none;
}
.url-input {
  flex: 1; padding: 9px 14px; font-size: 13px; font-family: var(--xuya-font-mono);
  border-radius: var(--xuya-radius-sm); border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg); color: var(--xuya-text); transition: border-color .12s, box-shadow .12s;
}
.url-input:focus { outline: none; border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }

.tabs { display: flex; gap: 4px; border-bottom: 1px solid var(--xuya-border); margin-bottom: 14px; align-items: center; }
.tab {
  padding: 8px 14px; font-size: 13px; color: var(--xuya-text-secondary);
  background: transparent; border: none; border-bottom: 2px solid transparent; margin-bottom: -1px;
  display: inline-flex; align-items: center; gap: 6px; transition: color .12s, border-color .12s;
}
.tab:hover { color: var(--xuya-text); }
.tab.active { color: var(--xuya-accent); border-bottom-color: var(--xuya-accent); font-weight: 600; }
.tab .badge { font-size: 10px; padding: 1px 6px; background: var(--xuya-accent-soft); color: var(--xuya-accent); border-radius: 99px; }
.tab .badge.dot { width: 6px; height: 6px; padding: 0; border-radius: 50%; }
.tab-extra { margin-left: auto; padding-bottom: 6px; }
.timeout-label { font-size: 11px; color: var(--xuya-text-tertiary); display: inline-flex; align-items: center; gap: 4px; }
.timeout-input { width: 48px; padding: 3px 6px; font-size: 12px; border-radius: var(--xuya-radius-sm); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); }

.tab-body { min-height: 120px; margin-bottom: 18px; }
.kv-editor { display: flex; flex-direction: column; gap: 6px; }
.kv-row { display: flex; gap: 6px; align-items: center; }
.kv-check { accent-color: var(--xuya-accent); width: 15px; height: 15px; flex-shrink: 0; }
.kv-input { flex: 1; padding: 6px 10px; font-size: 12.5px; border-radius: var(--xuya-radius-sm); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); transition: border-color .12s; }
.kv-input:focus { outline: none; border-color: var(--xuya-accent); }
.kv-del { width: 28px; height: 28px; flex-shrink: 0; display: flex; align-items: center; justify-content: center; border-radius: var(--xuya-radius-sm); color: var(--xuya-text-tertiary); transition: .1s; }
.kv-del:hover { color: var(--xuya-danger); background: var(--xuya-danger-soft); }
.kv-add { align-self: flex-start; display: inline-flex; align-items: center; gap: 4px; padding: 5px 12px; font-size: 12px; color: var(--xuya-accent); border: 1px dashed var(--xuya-border); border-radius: var(--xuya-radius-sm); margin-top: 4px; transition: .1s; }
.kv-add:hover { border-color: var(--xuya-accent); }

.body-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 8px; }
.seg { display: inline-flex; background: var(--xuya-input-bg); border-radius: var(--xuya-radius-sm); padding: 2px; gap: 2px; }
.seg button { padding: 4px 12px; font-size: 11px; color: var(--xuya-text-secondary); background: transparent; border: none; border-radius: var(--xuya-radius-sm); transition: .1s; }
.seg button.active { background: var(--xuya-bg-elevated); color: var(--xuya-accent); font-weight: 600; }
.body-hint { font-size: 11px; color: var(--xuya-text-tertiary); }
.body-editor { width: 100%; min-height: 120px; padding: 10px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); font-family: var(--xuya-font-mono); font-size: 12.5px; line-height: 1.6; resize: vertical; }
.body-editor:focus { outline: none; border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }

.response { background: var(--xuya-card-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-lg); padding: 16px; }
.resp-head { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
.status-badge { padding: 4px 12px; font-size: 13px; font-weight: 700; border-radius: var(--xuya-radius-sm); font-family: var(--xuya-font-mono); }
.status-badge.ok { background: var(--xuya-success-soft); color: var(--xuya-success); }
.status-badge.warn { background: var(--xuya-warn-soft); color: var(--xuya-warn); }
.status-badge.err { background: var(--xuya-danger-soft); color: var(--xuya-danger); }
.resp-meta { font-size: 12px; color: var(--xuya-text-tertiary); font-family: var(--xuya-font-mono); }
.mini-btn { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; font-size: 11px; color: var(--xuya-text-secondary); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); border-radius: var(--xuya-radius-sm); transition: .1s; }
.mini-btn:hover { color: var(--xuya-text); border-color: var(--xuya-accent); }
.resp-error { padding: 10px 14px; background: var(--xuya-danger-soft); color: var(--xuya-danger); border-radius: var(--xuya-radius-sm); font-size: 12.5px; margin-bottom: 12px; }

.resp-tabs { display: flex; gap: 8px; margin-bottom: 10px; }
.resp-tab { font-size: 12px; color: var(--xuya-text-secondary); padding: 3px 0; }
.resp-tab.active { color: var(--xuya-accent); font-weight: 600; }
.resp-headers { max-height: 180px; overflow: auto; margin-bottom: 12px; border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-sm); }
.resp-header-row { display: flex; gap: 8px; padding: 5px 12px; font-size: 11.5px; border-bottom: 1px solid var(--xuya-border-light); }
.resp-header-row:last-child { border-bottom: none; }
.rh-key { color: var(--xuya-accent); font-family: var(--xuya-font-mono); flex-shrink: 0; }
.rh-val { color: var(--xuya-text); font-family: var(--xuya-font-mono); word-break: break-all; }
.resp-body { margin: 0; padding: 14px; background: var(--xuya-input-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius); font-family: var(--xuya-font-mono); font-size: 12.5px; line-height: 1.6; max-height: 360px; overflow: auto; white-space: pre-wrap; word-break: break-all; color: var(--xuya-text); }
.resp-body.is-json code { color: var(--xuya-text); }

.history { margin-top: 16px; }
.section-label { font-size: 11px; text-transform: uppercase; letter-spacing: .5px; color: var(--xuya-text-tertiary); margin-bottom: 8px; }
.history-list { display: flex; flex-direction: column; gap: 4px; }
.history-item { display: flex; align-items: center; gap: 10px; padding: 6px 10px; font-size: 12px; border-radius: var(--xuya-radius-sm); transition: .1s; text-align: left; width: 100%; }
.history-item:hover { background: var(--xuya-input-bg); }
.hist-status { font-family: var(--xuya-font-mono); font-size: 11px; font-weight: 700; padding: 1px 6px; border-radius: var(--xuya-radius-sm); min-width: 36px; text-align: center; }
.hist-status.ok { background: var(--xuya-success-soft); color: var(--xuya-success); }
.hist-status.warn { background: var(--xuya-warn-soft); color: var(--xuya-warn); }
.hist-status.err { background: var(--xuya-danger-soft); color: var(--xuya-danger); }
.hist-method { font-weight: 700; color: var(--xuya-accent); min-width: 56px; }
.hist-url { flex: 1; color: var(--xuya-text-secondary); font-family: var(--xuya-font-mono); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.hist-time { color: var(--xuya-text-tertiary); font-size: 11px; }

.spin { animation: spin .8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
