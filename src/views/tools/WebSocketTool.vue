<template>
  <ToolShell
    title="WebSocket 调试"
    :icon="Webhook"
    description="连接 WebSocket 服务，双向收发消息，实时消息流，支持搜索过滤与导出。"
  >
    <!-- 连接栏 -->
    <div class="conn-row">
      <input
        v-model="url"
        class="url-input"
        type="text"
        list="ws-url-history"
        placeholder="wss://echo.websocket.events"
        spellcheck="false"
        @keydown.enter="onConnect"
      />
      <datalist id="ws-url-history">
        <option v-for="u in urlHistory" :key="u" :value="u" />
      </datalist>
      <span class="status-light" :class="statusClass"></span>
      <span class="status-text">{{ statusText }}{{ uptime ? ' · ' + uptime : '' }}</span>
      <BaseButton
        v-if="!connected"
        variant="primary"
        :disabled="connecting || !url"
        @click="onConnect"
      >
        <Loader2 v-if="connecting" :size="14" class="spin" />
        {{ connecting ? '连接中' : '连接' }}
      </BaseButton>
      <BaseButton v-else variant="danger" @click="onDisconnect">断开</BaseButton>
    </div>

    <!-- 消息工具栏 -->
    <div class="msg-toolbar">
      <div class="stats">
        <span class="stat-item send">↑ {{ sentCount }}</span>
        <span class="stat-item recv">↓ {{ recvCount }}</span>
        <span class="stat-item total">共 {{ messages.length }} 条</span>
      </div>
      <input
        v-model="searchQuery"
        class="search-input"
        type="text"
        placeholder="搜索消息…"
        spellcheck="false"
      />
      <div class="seg">
        <button :class="{ active: msgFilter === 'all' }" @click="msgFilter = 'all'">全部</button>
        <button :class="{ active: msgFilter === 'send' }" @click="msgFilter = 'send'">发送</button>
        <button :class="{ active: msgFilter === 'recv' }" @click="msgFilter = 'recv'">接收</button>
        <button :class="{ active: msgFilter === 'system' }" @click="msgFilter = 'system'">
          系统
        </button>
      </div>
      <button v-if="messages.length" class="mini-btn" @click="exportMessages">导出</button>
      <button v-if="messages.length" class="mini-btn danger" @click="messages = []">清空</button>
    </div>

    <div class="ws-grid">
      <!-- 消息流 -->
      <div class="msg-pane">
        <div ref="msgListRef" class="msg-list">
          <div v-if="!filteredMessages.length" class="empty">
            {{ messages.length ? '无匹配消息' : '连接后在此查看消息流' }}
          </div>
          <div
            v-for="(m, i) in filteredMessages"
            :key="i"
            class="msg-item"
            :class="m.dir"
            @click="copyMessage(m)"
          >
            <div class="msg-meta">
              <span class="msg-dir">
                {{ m.dir === 'send' ? '↑ 发送' : m.dir === 'recv' ? '↓ 接收' : '● 系统' }}
              </span>
              <span class="msg-time">{{ m.time }}</span>
              <span v-if="m.size" class="msg-size">{{ formatSize(m.size) }}</span>
            </div>
            <pre class="msg-data">{{ m.data }}</pre>
          </div>
        </div>
      </div>

      <!-- 发送区 -->
      <div class="send-pane">
        <div class="send-head">
          <span class="send-title">发送消息</span>
          <div class="seg">
            <button :class="{ active: sendFormat === 'text' }" @click="sendFormat = 'text'">
              文本
            </button>
            <button :class="{ active: sendFormat === 'json' }" @click="sendFormat = 'json'">
              JSON
            </button>
          </div>
        </div>
        <textarea
          v-model="sendText"
          class="send-editor"
          :placeholder="sendPlaceholder"
          spellcheck="false"
          :disabled="!connected"
          @keydown.ctrl.enter="onSend"
        ></textarea>
        <div class="send-actions">
          <BaseButton variant="primary" :disabled="!connected || !sendText" @click="onSend">
            发送
            <span class="shortcut">Ctrl+Enter</span>
          </BaseButton>
          <button v-if="sendFormat === 'json'" class="mini-btn" @click="formatJson">格式化</button>
          <label class="auto-scroll">
            <input v-model="autoScroll" type="checkbox" />
            自动滚动
          </label>
        </div>

        <div class="quick-section">
          <span class="quick-label">快捷发送</span>
          <div class="quick-list">
            <button
              v-for="q in QUICK"
              :key="q"
              class="quick-item"
              :disabled="!connected"
              @click="sendText = q"
            >
              {{ q }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { Webhook, Loader2 } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';
import { wsConnect, wsSend, wsDisconnect, type WsEvent } from '@/composables/useWsTool';

const toast = useToast();

const url = useToolState('ws', 'url', 'wss://echo.websocket.events');
const urlHistory = useToolState<string[]>('ws', 'urlHistory', []);
const sendText = useToolState('ws', 'sendText', 'Hello, WebSocket!');
const sendFormat = useToolState<'text' | 'json'>('ws', 'sendFormat', 'text');
const autoScroll = useToolState('ws', 'autoScroll', true);

const connected = ref(false);
const connecting = ref(false);
const connId = ref<string | null>(null);
const searchQuery = ref('');
const msgFilter = ref<'all' | 'send' | 'recv' | 'system'>('all');

interface Msg {
  dir: 'send' | 'recv' | 'system';
  data: string;
  time: string;
  size: number;
}
const messages = ref<Msg[]>([]);

const QUICK = ['ping', 'pong', '{"type":"ping"}', '{"action":"subscribe","channel":"test"}'];

const statusClass = computed(() =>
  connected.value ? 'on' : connecting.value ? 'connecting' : 'off',
);
const statusText = computed(() =>
  connected.value ? '已连接' : connecting.value ? '连接中…' : '未连接',
);

const sentCount = computed(() => messages.value.filter((m) => m.dir === 'send').length);
const recvCount = computed(() => messages.value.filter((m) => m.dir === 'recv').length);

const filteredMessages = computed(() => {
  const q = searchQuery.value.toLowerCase();
  return messages.value.filter((m) => {
    if (msgFilter.value !== 'all' && m.dir !== msgFilter.value) return false;
    if (q && !m.data.toLowerCase().includes(q)) return false;
    return true;
  });
});

const sendPlaceholder = computed(() =>
  sendFormat.value === 'json' ? '{\n  "key": "value"\n}' : '输入要发送的文本…',
);

const msgListRef = ref<HTMLElement | null>(null);
function scrollToBottom() {
  if (autoScroll.value && msgListRef.value) {
    msgListRef.value.scrollTop = msgListRef.value.scrollHeight;
  }
}
watch(
  () => messages.value.length,
  () => nextTick(scrollToBottom),
);

let unlisten: UnlistenFn | undefined;
let uptimeTimer: ReturnType<typeof setInterval> | undefined;
const uptime = ref('');
const connectTime = ref<number | null>(null);

function nowTime() {
  return new Date().toLocaleTimeString('zh-CN', { hour12: false });
}

function byteSize(s: string): number {
  return new TextEncoder().encode(s).length;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  return `${(bytes / 1024).toFixed(1)} KB`;
}

function startUptime() {
  connectTime.value = Date.now();
  uptimeTimer = setInterval(() => {
    if (!connectTime.value) return;
    const secs = Math.floor((Date.now() - connectTime.value) / 1000);
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    uptime.value = m > 0 ? `${m}m${s}s` : `${s}s`;
  }, 1000);
}

function stopUptime() {
  connectTime.value = null;
  uptime.value = '';
  if (uptimeTimer) clearInterval(uptimeTimer);
}

async function onConnect() {
  if (!url.value.trim()) return;
  connecting.value = true;
  messages.value = [];
  try {
    const id = await wsConnect(url.value);
    connId.value = id;
    connected.value = true;
    messages.value.push({
      dir: 'system',
      data: `已连接到 ${url.value}`,
      time: nowTime(),
      size: 0,
    });
    urlHistory.value = [url.value, ...urlHistory.value.filter((u) => u !== url.value)].slice(0, 10);
    startUptime();
  } catch (e) {
    messages.value.push({
      dir: 'system',
      data: `连接失败: ${e}`,
      time: nowTime(),
      size: 0,
    });
    toast.error('连接失败');
  } finally {
    connecting.value = false;
  }
}

async function onDisconnect() {
  if (connId.value) {
    try {
      await wsDisconnect(connId.value);
    } catch {
      /* ignore */
    }
    connId.value = null;
    connected.value = false;
    stopUptime();
    messages.value.push({
      dir: 'system',
      data: '已断开连接',
      time: nowTime(),
      size: 0,
    });
  }
}

async function onSend() {
  if (!connected.value || !connId.value || !sendText.value) return;
  try {
    await wsSend(connId.value, sendText.value);
    messages.value.push({
      dir: 'send',
      data: sendText.value,
      time: nowTime(),
      size: byteSize(sendText.value),
    });
  } catch (e) {
    toast.error('发送失败');
    messages.value.push({
      dir: 'system',
      data: `发送失败: ${e}`,
      time: nowTime(),
      size: 0,
    });
  }
}

function formatJson() {
  try {
    sendText.value = JSON.stringify(JSON.parse(sendText.value), null, 2);
    toast.success('已格式化');
  } catch {
    toast.error('无效的 JSON');
  }
}

async function copyMessage(m: Msg) {
  await copyToClipboard(m.data, '已复制消息');
}

async function exportMessages() {
  const text = messages.value
    .map((m) => {
      const dir = m.dir === 'send' ? '↑ SEND' : m.dir === 'recv' ? '↓ RECV' : '● SYS';
      return `[${m.time}] ${dir}\n${m.data}`;
    })
    .join('\n\n');
  await copyToClipboard(text, `已导出 ${messages.value.length} 条消息`);
}

onMounted(async () => {
  try {
    unlisten = await listen<WsEvent>('ws-message', (event) => {
      const payload = event.payload;
      if (connId.value && payload.connectionId !== connId.value) return;
      if (payload.kind === 'message') {
        messages.value.push({
          dir: 'recv',
          data: payload.data,
          time: nowTime(),
          size: byteSize(payload.data),
        });
      } else if (payload.kind === 'close' || payload.kind === 'error') {
        messages.value.push({
          dir: 'system',
          data: payload.data,
          time: nowTime(),
          size: 0,
        });
        connected.value = false;
        connId.value = null;
        stopUptime();
      }
    });
  } catch {
    /* 非 Tauri 环境 */
  }
});

onUnmounted(() => {
  unlisten?.();
  stopUptime();
  if (connId.value) wsDisconnect(connId.value).catch(() => {});
});
</script>

<style scoped>
.conn-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
  flex-wrap: wrap;
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
.status-light {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}
.status-light.on {
  background: var(--xuya-success);
  box-shadow: 0 0 8px var(--xuya-success);
}
.status-light.connecting {
  background: var(--xuya-warn);
  animation: pulse 1s infinite;
}
.status-light.off {
  background: var(--xuya-text-tertiary);
}
@keyframes pulse {
  50% {
    opacity: 0.4;
  }
}
.status-text {
  font-size: 12px;
  color: var(--xuya-text-secondary);
  font-family: var(--xuya-font-mono);
  white-space: nowrap;
}

.msg-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}
.stats {
  display: flex;
  gap: 10px;
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  font-weight: 600;
}
.stat-item.send {
  color: var(--xuya-accent);
}
.stat-item.recv {
  color: var(--xuya-success);
}
.stat-item.total {
  color: var(--xuya-text-tertiary);
}
.search-input {
  width: 140px;
  padding: 5px 10px;
  font-size: 12px;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  outline: none;
  transition: border-color var(--xuya-duration-fast);
}
.search-input:focus {
  border-color: var(--xuya-accent);
}
.search-input::placeholder {
  color: var(--xuya-text-tertiary);
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
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  cursor: pointer;
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
.mini-btn {
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
.mini-btn.danger:hover {
  color: var(--xuya-danger);
  border-color: var(--xuya-danger);
}

.ws-grid {
  display: grid;
  grid-template-columns: 1fr 340px;
  gap: 14px;
  flex: 1;
  min-height: 0;
}
.msg-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-lg);
  overflow: hidden;
}
.msg-list {
  flex: 1;
  overflow: auto;
  padding: 10px;
}
.empty {
  text-align: center;
  color: var(--xuya-text-tertiary);
  font-size: 13px;
  padding: 40px 0;
}
.msg-item {
  margin-bottom: 6px;
  padding: 8px 12px;
  border-radius: var(--xuya-radius-sm);
  border-left: 3px solid;
  cursor: pointer;
  transition: transform var(--xuya-duration-fast);
}
.msg-item:hover {
  transform: translateX(2px);
}
.msg-item.send {
  background: var(--xuya-accent-soft);
  border-left-color: var(--xuya-accent);
}
.msg-item.recv {
  background: var(--xuya-success-soft);
  border-left-color: var(--xuya-success);
}
.msg-item.system {
  background: var(--xuya-input-bg);
  border-left-color: var(--xuya-text-tertiary);
}
.msg-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}
.msg-dir {
  font-size: 11px;
  font-weight: 700;
}
.msg-item.send .msg-dir {
  color: var(--xuya-accent);
}
.msg-item.recv .msg-dir {
  color: var(--xuya-success);
}
.msg-item.system .msg-dir {
  color: var(--xuya-text-tertiary);
}
.msg-time {
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
}
.msg-size {
  font-size: 10px;
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
  margin-left: auto;
}
.msg-data {
  margin: 0;
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  line-height: 1.5;
  color: var(--xuya-text);
  white-space: pre-wrap;
  word-break: break-all;
}

.send-pane {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.send-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.send-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.send-editor {
  width: 100%;
  min-height: 120px;
  padding: 10px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.5;
  resize: vertical;
  transition: border-color var(--xuya-duration-fast);
}
.send-editor:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.send-editor::placeholder {
  color: var(--xuya-text-tertiary);
}
.send-editor:disabled {
  opacity: 0.5;
}
.send-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}
.shortcut {
  font-size: 10px;
  opacity: 0.7;
  margin-left: 4px;
}
.auto-scroll {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  cursor: pointer;
  margin-left: auto;
}
.auto-scroll input {
  accent-color: var(--xuya-accent);
  width: 13px;
  height: 13px;
}

.quick-section {
  margin-top: 4px;
}
.quick-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
  display: block;
  margin-bottom: 6px;
}
.quick-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.quick-item {
  text-align: left;
  padding: 6px 10px;
  font-size: 11.5px;
  font-family: var(--xuya-font-mono);
  color: var(--xuya-text-secondary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.quick-item:hover:not(:disabled) {
  color: var(--xuya-text);
  border-color: var(--xuya-accent);
}
.quick-item:disabled {
  opacity: 0.4;
  cursor: not-allowed;
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
  .ws-grid {
    grid-template-columns: 1fr;
  }
}
</style>
