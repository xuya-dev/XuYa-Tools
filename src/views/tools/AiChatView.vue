<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useRoute } from 'vue-router';
import { Sparkles } from '@lucide/vue';
import type { Webview } from '@tauri-apps/api/webview';
import type { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { tools } from '@/config/tools';
import { createBrandIcon } from '@/config/aiIcons';
import ToolShell from '@/components/layout/ToolShell.vue';

const route = useRoute();
const vendorId = computed(() => String(route.params.id));
const tool = computed(() => tools.find((t) => t.id === 'ai_' + vendorId.value));

const title = computed(() => tool.value?.name ?? 'AI 对话');
const description = computed(() => tool.value?.description ?? '');
const icon = computed(() =>
  tool.value?.brandIcon ? createBrandIcon(tool.value.brandIcon) : Sparkles,
);

const containerEl = ref<HTMLElement | null>(null);
const useIframe = ref(false);
const openedInWindow = ref(false);
const error = ref('');
let webview: Webview | null = null;
let externalWindow: WebviewWindow | null = null;
let resizeObserver: ResizeObserver | null = null;

async function destroyWebview() {
  if (!webview) return;
  try {
    await webview.close();
  } catch {
    /* ignore */
  }
  webview = null;
}

async function closeExternalWindow() {
  if (!externalWindow) return;
  try {
    await externalWindow.close();
  } catch {
    /* ignore */
  }
  externalWindow = null;
}

async function openExternalWindow() {
  if (!tool.value?.url) return;
  try {
    await closeExternalWindow();
    const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
    externalWindow = new WebviewWindow(`ai-chat-${vendorId.value}`, {
      url: tool.value.url,
      title: title.value,
      width: 1100,
      height: 720,
      center: true,
    });
    externalWindow.once('tauri://error', () => {
      closeExternalWindow();
      useIframe.value = true;
      openedInWindow.value = false;
    });
    openedInWindow.value = true;
    useIframe.value = false;
  } catch (e) {
    console.error(e);
    useIframe.value = true;
    openedInWindow.value = false;
  }
}

async function syncBounds() {
  if (!webview || !containerEl.value) return;
  const rect = containerEl.value.getBoundingClientRect();
  const { LogicalPosition, LogicalSize } = await import('@tauri-apps/api/dpi');
  await webview.setPosition(new LogicalPosition(rect.left, rect.top));
  await webview.setSize(new LogicalSize(rect.width, rect.height));
}

async function createWebview() {
  await destroyWebview();
  await closeExternalWindow();
  useIframe.value = false;
  openedInWindow.value = false;
  error.value = '';
  if (!tool.value?.url || !containerEl.value) {
    error.value = '未配置网页地址';
    return;
  }
  try {
    const [{ Webview }, { getCurrentWindow }] = await Promise.all([
      import('@tauri-apps/api/webview'),
      import('@tauri-apps/api/window'),
    ]);
    try {
      const win = getCurrentWindow();
      const rect = containerEl.value.getBoundingClientRect();
      webview = new Webview(win, `ai-chat-${vendorId.value}`, {
        url: tool.value.url,
        x: rect.left,
        y: rect.top,
        width: rect.width,
        height: rect.height,
      });
      await webview.setAutoResize(true);
      webview.once('tauri://error', async () => {
        await destroyWebview();
        await openExternalWindow();
      });
    } catch (e) {
      console.error('create webview error', e);
      await openExternalWindow();
    }
  } catch (e) {
    console.error('tauri not available', e);
    useIframe.value = true;
  }
}

onMounted(() => {
  createWebview();
  if (containerEl.value) {
    resizeObserver = new ResizeObserver(() => syncBounds());
    resizeObserver.observe(containerEl.value);
  }
});

onUnmounted(async () => {
  resizeObserver?.disconnect();
  await destroyWebview();
  await closeExternalWindow();
});

watch(vendorId, createWebview);
</script>

<template>
  <ToolShell :title="title" :description="description" :icon="icon">
    <div class="webview-host">
      <div ref="containerEl" class="webview-surface">
        <iframe
          v-if="useIframe"
          :src="tool?.url"
          class="webview-iframe"
          sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
          title="在线对话"
        ></iframe>
        <div v-if="openedInWindow" class="webview-placeholder">
          <p>已在应用内新窗口中打开</p>
          <a v-if="tool?.url" :href="tool.url" target="_blank">在浏览器中打开</a>
        </div>
      </div>
      <div v-if="error" class="webview-error">
        <p>{{ error }}</p>
      </div>
      <div v-if="useIframe" class="webview-hint">
        <span>若页面无法加载，可</span>
        <a v-if="tool?.url" :href="tool.url" target="_blank">在浏览器中打开</a>
      </div>
    </div>
  </ToolShell>
</template>

<style scoped>
.webview-host {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  position: relative;
}

.webview-surface {
  flex: 1;
  min-height: 0;
  position: relative;
}

.webview-iframe {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  border: none;
  background: var(--xuya-bg-elevated);
}

.webview-placeholder,
.webview-error {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-bg-elevated);
}

.webview-placeholder a,
.webview-error a,
.webview-hint a {
  color: var(--xuya-accent);
  text-decoration: none;
}

.webview-placeholder a:hover,
.webview-hint a:hover {
  text-decoration: underline;
}

.webview-hint {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 12px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-bg-elevated);
  border-top: 1px solid var(--xuya-border);
}
</style>
