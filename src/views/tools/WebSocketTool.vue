<template>
  <ToolShell title="WebSocket 调试" :icon="Webhook" description="连接 WebSocket 服务,双向收发消息,实时查看消息流。">
    <!-- 连接栏 -->
    <div class="conn-row">
      <input v-model="url" class="url-input" type="text" placeholder="wss://echo.websocket.events" spellcheck="false" @keydown.enter="onConnect" />
      <span class="status-light" :class="statusClass"></span>
      <span class="status-text">{{ statusText }}</span>
      <BaseButton v-if="!connected" variant="primary" :disabled="connecting || !url" @click="onConnect">
        <Loader2 v-if="connecting" :size="14" class="spin" />
        {{ connecting ? '连接中' : '连接' }}
      </BaseButton>
      <BaseButton v-else variant="danger" @click="onDisconnect">断开</BaseButton>
    </div>

    <div class="ws-grid">
      <!-- 消息流 -->
      <div class="msg-pane">
        <div class="pane-head">
          <span>消息流 ({{ messages.length }})</span>
          <div class="pane-actions">
            <label class="auto-scroll"><input type="checkbox" v-model="autoScroll" checked /> 自动滚动</label>
            <button class="mini-btn" @click="messages = []" v-if="messages.length">清空</button>
          </div>
        </div>
        <div class="msg-list" ref="msgListRef">
          <div v-if="!messages.length" class="empty">连接后在此查看消息流</div>
          <div v-for="(m, i) in messages" :key="i" class="msg-item" :class="m.dir">
            <span class="msg-dir">{{ m.dir === 'send' ? '→ 发送' : m.dir === 'recv' ? '← 接收' : '● 系统' }}</span>
            <span class="msg-time">{{ m.time }}</span>
            <pre class="msg-data">{{ m.data }}</pre>
          </div>
        </div>
      </div>

      <!-- 发送区 -->
      <div class="send-pane">
        <div class="pane-head">
          <span>发送消息</span>
          <div class="seg">
            <button :class="{ active: sendFormat === 'text' }" @click="sendFormat = 'text'">文本</button>
            <button :class="{ active: sendFormat === 'json' }" @click="sendFormat = 'json'">JSON</button>
          </div>
        </div>
        <textarea v-model="sendText" class="send-editor" :placeholder="sendPlaceholder" spellcheck="false" @keydown.ctrl.enter="onSend" :disabled="!connected"></textarea>
        <div class="send-actions">
          <BaseButton variant="primary" :disabled="!connected || !sendText" @click="onSend">发送 (Ctrl+Enter)</BaseButton>
          <button class="mini-btn" @click="formatJson" v-if="sendFormat === 'json'">格式化</button>
        </div>

        <!-- 快捷发送 -->
        <div class="quick-send" v-if="connected">
          <div class="section-label">快捷</div>
          <div class="quick-list">
            <button v-for="q in QUICK" :key="q" class="quick-item" @click="sendText = q">{{ q }}</button>
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
import { wsConnect, wsSend, wsDisconnect, type WsEvent } from '@/composables/useWsTool';

const toast = useToast();

const url = ref('wss://echo.websocket.events');
const connected = ref(false);
const connecting = ref(false);
const connId = ref<string | null>(null);
const autoScroll = ref(true);

interface Msg { dir: 'send' | 'recv' | 'system'; data: string; time: string; }
const messages = ref<Msg[]>([]);

const sendText = ref('Hello, WebSocket!');
const sendFormat = ref<'text' | 'json'>('text');

const QUICK = ['ping', '{"type":"ping"}', '{"action":"subscribe","channel":"test"}'];

const statusClass = computed(() => connected.value ? 'on' : (connecting.value ? 'connecting' : 'off'));
const statusText = computed(() => connected.value ? '已连接' : (connecting.value ? '连接中...' : '未连接'));

const sendPlaceholder = computed(() => sendFormat.value === 'json' ? '{\n  "key": "value"\n}' : '输入要发送的文本...');

const msgListRef = ref<HTMLElement | null>(null);
function scrollToBottom() {
  if (autoScroll.value && msgListRef.value) {
    msgListRef.value.scrollTop = msgListRef.value.scrollHeight;
  }
}
watch(() => messages.value.length, () => nextTick(scrollToBottom));

let unlisten: UnlistenFn | undefined;

function nowTime() {
  return new Date().toLocaleTimeString('zh-CN', { hour12: false });
}

async function onConnect() {
  if (!url.value.trim()) return;
  connecting.value = true;
  messages.value = [];
  try {
    const id = await wsConnect(url.value);
    connId.value = id;
    connected.value = true;
    messages.value.push({ dir: 'system', data: `已连接到 ${url.value}`, time: nowTime() });
  } catch (e) {
    messages.value.push({ dir: 'system', data: `连接失败: ${e}`, time: nowTime() });
    toast.error('连接失败: ' + e);
  } finally {
    connecting.value = false;
  }
}

async function onDisconnect() {
  if (connId.value) {
    try {
      await wsDisconnect(connId.value);
    } catch { /* ignore */ }
    connId.value = null;
    connected.value = false;
    messages.value.push({ dir: 'system', data: '已断开连接', time: nowTime() });
  }
}

async function onSend() {
  if (!connected.value || !connId.value || !sendText.value) return;
  try {
    await wsSend(connId.value, sendText.value);
    messages.value.push({ dir: 'send', data: sendText.value, time: nowTime() });
  } catch (e) {
    toast.error('发送失败: ' + e);
    messages.value.push({ dir: 'system', data: `发送失败: ${e}`, time: nowTime() });
  }
}

function formatJson() {
  try {
    const parsed = JSON.parse(sendText.value);
    sendText.value = JSON.stringify(parsed, null, 2);
    toast.success('已格式化');
  } catch {
    toast.error('无效的 JSON');
  }
}

onMounted(async () => {
  try {
    unlisten = await listen<WsEvent>('ws-message', (event) => {
      const payload = event.payload;
      if (connId.value && payload.connectionId !== connId.value) return; // 只关心当前连接
      const dir = payload.kind === 'message' ? 'recv' : 'system';
      const prefix = payload.kind === 'error' ? '错误: ' : payload.kind === 'close' ? '' : payload.kind === 'open' ? '' : '';
      messages.value.push({ dir, data: prefix + payload.data, time: nowTime() });
      if (payload.kind === 'close' || payload.kind === 'error') {
        connected.value = false;
        connId.value = null;
      }
    });
  } catch { /* 非 Tauri 环境忽略 */ }
});

onUnmounted(() => {
  unlisten?.();
  if (connId.value) { wsDisconnect(connId.value).catch(() => {}); }
});
</script>

<style scoped>
.conn-row { display: flex; align-items: center; gap: 10px; margin-bottom: 14px; }
.url-input { flex: 1; padding: 9px 14px; font-size: 13px; font-family: var(--xuya-font-mono); border-radius: var(--xuya-radius-sm); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); transition: border-color .12s, box-shadow .12s; }
.url-input:focus { outline: none; border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }
.status-light { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
.status-light.on { background: var(--xuya-success); box-shadow: 0 0 8px rgba(52,211,153,.6); }
.status-light.connecting { background: var(--xuya-warn); animation: pulse 1s infinite; }
.status-light.off { background: var(--xuya-text-3); }
@keyframes pulse { 50% { opacity: .4; } }
.status-text { font-size: 12px; color: var(--xuya-text-secondary); min-width: 64px; }

.ws-grid { display: grid; grid-template-columns: 1fr 360px; gap: 14px; flex: 1; min-height: 0; }
.msg-pane { display: flex; flex-direction: column; min-height: 0; background: var(--xuya-card-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-lg); }
.pane-head { display: flex; align-items: center; justify-content: space-between; padding: 10px 14px; border-bottom: 1px solid var(--xuya-border); font-size: 12px; font-weight: 600; color: var(--xuya-text-secondary); }
.pane-actions { display: flex; align-items: center; gap: 10px; }
.auto-scroll { display: inline-flex; align-items: center; gap: 4px; font-size: 11px; color: var(--xuya-text-tertiary); font-weight: 400; cursor: pointer; }
.auto-scroll input { accent-color: var(--xuya-accent); }
.mini-btn { padding: 3px 10px; font-size: 11px; color: var(--xuya-text-secondary); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); border-radius: var(--xuya-radius-sm); transition: .1s; }
.mini-btn:hover { color: var(--xuya-text); border-color: var(--xuya-accent); }
.msg-list { flex: 1; overflow: auto; padding: 10px; }
.empty { text-align: center; color: var(--xuya-text-tertiary); font-size: 13px; padding: 40px 0; }
.msg-item { margin-bottom: 8px; padding: 8px 12px; border-radius: var(--xuya-radius-sm); border-left: 3px solid; }
.msg-item.send { background: var(--xuya-accent-soft); border-left-color: var(--xuya-accent); }
.msg-item.recv { background: var(--xuya-success-soft); border-left-color: var(--xuya-success); }
.msg-item.system { background: var(--xuya-input-bg); border-left-color: var(--xuya-text-3); }
.msg-dir { font-size: 11px; font-weight: 700; margin-right: 8px; }
.msg-item.send .msg-dir { color: var(--xuya-accent); }
.msg-item.recv .msg-dir { color: var(--xuya-success); }
.msg-item.system .msg-dir { color: var(--xuya-text-3); }
.msg-time { font-size: 10.5px; color: var(--xuya-text-tertiary); font-family: var(--xuya-font-mono); }
.msg-data { margin: 4px 0 0; font-family: var(--xuya-font-mono); font-size: 12px; line-height: 1.5; color: var(--xuya-text); white-space: pre-wrap; word-break: break-all; }

.send-pane { display: flex; flex-direction: column; gap: 10px; }
.seg { display: inline-flex; background: var(--xuya-input-bg); border-radius: var(--xuya-radius-sm); padding: 2px; gap: 2px; }
.seg button { padding: 4px 12px; font-size: 11px; color: var(--xuya-text-secondary); background: transparent; border: none; border-radius: 4px; transition: .1s; }
.seg button.active { background: var(--xuya-bg-elevated); color: var(--xuya-accent); font-weight: 600; }
.send-editor { width: 100%; min-height: 120px; padding: 10px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); font-family: var(--xuya-font-mono); font-size: 12.5px; line-height: 1.5; resize: vertical; transition: border-color .12s; }
.send-editor:focus { outline: none; border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }
.send-actions { display: flex; gap: 8px; align-items: center; }
.quick-send { margin-top: 4px; }
.section-label { font-size: 11px; text-transform: uppercase; letter-spacing: .5px; color: var(--xuya-text-tertiary); margin-bottom: 6px; }
.quick-list { display: flex; flex-direction: column; gap: 4px; }
.quick-item { text-align: left; padding: 6px 10px; font-size: 11.5px; font-family: var(--xuya-font-mono); color: var(--xuya-text-secondary); background: var(--xuya-input-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-sm); transition: .1s; }
.quick-item:hover { color: var(--xuya-text); border-color: var(--xuya-accent); }

.spin { animation: spin .8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
