<template>
  <ToolShell title="二维码解码" :icon="ScanQrCode" description="上传二维码图片,自动识别并提取内容,支持拖拽粘贴。">
    <div class="decode-grid">
      <!-- 上传区 -->
      <div class="upload-col">
        <div
            class="drop-zone"
            :class="{ dragging, 'has-image': !!previewUrl }"
            @dragover.prevent="dragging = true"
            @dragleave.prevent="dragging = false"
            @drop.prevent="onDrop"
            @click="triggerFileInput"
            @paste="onPaste"
        >
          <input ref="fileInputRef" type="file" accept="image/*" class="hidden-input" @change="onFileSelect" />
          <template v-if="previewUrl">
            <img :src="previewUrl" class="preview-img" alt="二维码预览" />
            <button class="clear-btn" @click.stop="clearAll">
              <X :size="14" />
            </button>
          </template>
          <template v-else>
            <Upload :size="36" class="drop-icon" />
            <span class="drop-title">点击选择 / 拖拽 / 粘贴图片</span>
            <span class="drop-hint">支持 PNG / JPG / WEBP / BMP</span>
          </template>
        </div>
      </div>

      <!-- 结果区 -->
      <div class="result-col">
        <div class="col-head">
          <span>解码结果</span>
          <BaseButton v-if="result" variant="ghost" @click="copyResult">
            <Copy :size="13" /> 复制
          </BaseButton>
        </div>

        <div v-if="decoding" class="result-status">
          <Loader2 :size="20" class="spin" /> 解码中…
        </div>

        <div v-else-if="error" class="result-status error">
          <AlertCircle :size="20" />
          <span>{{ error }}</span>
        </div>

        <div v-else-if="result" class="result-content">
          <div class="result-type">
            <span class="type-badge" :class="resultType">{{ resultTypeLabel }}</span>
          </div>
          <pre class="result-text" :class="{ link: isLink }" @click="isLink ? openLink() : undefined">{{ result }}</pre>
          <div v-if="isLink" class="result-actions">
            <BaseButton variant="ghost" @click="openLink">
              <ExternalLink :size="13" /> 打开链接
            </BaseButton>
          </div>
        </div>

        <div v-else class="result-placeholder">
          <ScanQrCode :size="40" />
          <p>上传二维码图片后自动解码</p>
        </div>
      </div>
    </div>

    <!-- 历史记录 -->
    <div v-if="history.length" class="history-section">
      <div class="section-label">解码历史</div>
      <div class="history-list">
        <button v-for="(h, i) in history" :key="i" class="history-item" @click="historyClick(h)">
          <span class="hist-type" :class="h.type">{{ h.typeLabel }}</span>
          <span class="hist-text">{{ h.text }}</span>
          <span class="hist-time">{{ h.time }}</span>
        </button>
      </div>
      <button class="clear-history" @click="history = []">清空历史</button>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { ScanQrCode, Upload, X, Copy, AlertCircle, Loader2, ExternalLink } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';
import jsQR from 'jsqr';

const toast = useToast();

const fileInputRef = ref<HTMLInputElement | null>(null);
const previewUrl = ref('');
const imageData = ref<ImageData | null>(null);
const result = ref('');
const error = ref('');
const decoding = ref(false);
const dragging = ref(false);

interface HistoryItem {
  text: string;
  type: string;
  typeLabel: string;
  time: string;
}

const history = ref<HistoryItem[]>([]);

const resultType = computed(() => {
  if (!result.value) return 'text';
  if (/^https?:\/\//i.test(result.value)) return 'url';
  if (/^mailto:/i.test(result.value)) return 'email';
  if (/^tel:/i.test(result.value)) return 'phone';
  if (/^WIFI:/i.test(result.value)) return 'wifi';
  if (/^BEGIN:VCARD/i.test(result.value)) return 'vcard';
  return 'text';
});

const resultTypeLabel = computed(() => {
  const map: Record<string, string> = {
    url: '🔗 链接',
    email: '📧 邮箱',
    phone: '📞 电话',
    wifi: '📶 WiFi',
    vcard: '👤 名片',
    text: '📝 文本',
  };
  return map[resultType.value] ?? '📝 文本';
});

const isLink = computed(() => resultType.value === 'url');

function triggerFileInput() {
  if (!previewUrl.value) fileInputRef.value?.click();
}

function onFileSelect(e: Event) {
  const input = e.target as HTMLInputElement;
  if (input.files?.[0]) loadImage(input.files[0]);
}

function onDrop(e: DragEvent) {
  dragging.value = false;
  const file = e.dataTransfer?.files[0];
  if (file) loadImage(file);
}

function onPaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items;
  if (!items) return;
  for (const item of items) {
    if (item.type.startsWith('image/')) {
      const file = item.getAsFile();
      if (file) {
        loadImage(file);
        e.preventDefault();
        break;
      }
    }
  }
}

function loadImage(file: File) {
  if (!file.type.startsWith('image/')) {
    toast.error('请选择图片文件');
    return;
  }

  clearAll();
  previewUrl.value = URL.createObjectURL(file);

  const img = new Image();
  img.onload = () => {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      error.value = '无法创建 Canvas 上下文';
      return;
    }

    // 限制最大尺寸提升性能
    const maxSize = 1500;
    let { width, height } = img;
    if (width > maxSize || height > maxSize) {
      const scale = maxSize / Math.max(width, height);
      width = Math.round(width * scale);
      height = Math.round(height * scale);
    }

    canvas.width = width;
    canvas.height = height;
    ctx.drawImage(img, 0, 0, width, height);
    imageData.value = ctx.getImageData(0, 0, width, height);
    decode();
  };
  img.onerror = () => {
    error.value = '图片加载失败';
  };
  img.src = previewUrl.value;
}

function decode() {
  if (!imageData.value) return;
  decoding.value = true;
  error.value = '';
  result.value = '';

  // 异步解码避免 UI 卡顿
  setTimeout(() => {
    try {
      const code = jsQR(imageData.value!.data, imageData.value!.width, imageData.value!.height, {
        inversionAttempts: 'attemptBoth',
      });

      if (code && code.data) {
        result.value = code.data;
        addToHistory(code.data);
        toast.success('解码成功');
      } else {
        error.value = '未在图片中找到二维码,请确保图片清晰且包含完整二维码';
      }
    } catch (e) {
      error.value = '解码失败: ' + (e instanceof Error ? e.message : String(e));
    } finally {
      decoding.value = false;
    }
  }, 50);
}

function addToHistory(text: string) {
  const type = resultType.value;
  const typeLabels: Record<string, string> = {
    url: '链接', email: '邮箱', phone: '电话', wifi: 'WiFi', vcard: '名片', text: '文本',
  };
  const now = new Date();
  const time = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;

  history.value.unshift({
    text: text.length > 80 ? text.slice(0, 80) + '…' : text,
    type,
    typeLabel: typeLabels[type] ?? '文本',
    time,
  });

  if (history.value.length > 20) history.value.pop();
}

function historyClick(h: HistoryItem) {
  result.value = h.text.endsWith('…') ? h.text : h.text;
}

async function copyResult() {
  if (result.value) await copyToClipboard(result.value, '已复制解码结果');
}

async function openLink() {
  if (isLink.value) {
    try {
      const opener = await import('@tauri-apps/plugin-opener');
      await opener.openUrl(result.value);
    } catch {
      window.open(result.value, '_blank');
    }
  }
}

function clearAll() {
  if (previewUrl.value) URL.revokeObjectURL(previewUrl.value);
  previewUrl.value = '';
  imageData.value = null;
  result.value = '';
  error.value = '';
  if (fileInputRef.value) fileInputRef.value.value = '';
}

// 全局粘贴监听
function globalPaste(e: ClipboardEvent) {
  if (!e.clipboardData?.items) return;
  for (const item of e.clipboardData.items) {
    if (item.type.startsWith('image/')) {
      const file = item.getAsFile();
      if (file) {
        loadImage(file);
        e.preventDefault();
        break;
      }
    }
  }
}

onMounted(() => {
  window.addEventListener('paste', globalPaste);
});

onUnmounted(() => {
  window.removeEventListener('paste', globalPaste);
  if (previewUrl.value) URL.revokeObjectURL(previewUrl.value);
});
</script>

<style scoped>
.decode-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
}
.upload-col,
.result-col {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 0;
}

/* 上传区 */
.drop-zone {
  flex: 1;
  min-height: 280px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 2px dashed var(--xuya-border-strong);
  border-radius: var(--xuya-radius);
  background: var(--xuya-input-bg);
  cursor: pointer;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  position: relative;
  overflow: hidden;
}
.drop-zone:hover {
  border-color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
}
.drop-zone.dragging {
  border-color: var(--xuya-accent);
  background: var(--xuya-accent-soft-strong);
  transform: scale(1.01);
}
.drop-zone.has-image {
  border-style: solid;
  border-color: var(--xuya-border);
  cursor: default;
}
.drop-zone.has-image:hover {
  background: var(--xuya-input-bg);
}
.hidden-input {
  display: none;
}
.preview-img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
.drop-icon {
  color: var(--xuya-text-tertiary);
  margin-bottom: 4px;
}
.drop-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.drop-hint {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
}
.clear-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--xuya-radius-sm);
  background: var(--xuya-overlay);
  color: var(--xuya-on-accent);
  border: none;
  cursor: pointer;
  transition: background var(--xuya-duration-fast) var(--xuya-ease);
}
.clear-btn:hover {
  background: var(--xuya-danger);
}

/* 结果区 */
.col-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.result-status {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 13px;
  color: var(--xuya-text-tertiary);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  background: var(--xuya-input-bg);
}
.result-status.error {
  color: var(--xuya-danger);
  flex-direction: column;
  gap: 10px;
  padding: 24px;
  text-align: center;
}
.result-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10px;
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  background: var(--xuya-input-bg);
  padding: 14px;
  overflow: auto;
}
.result-type {
  display: flex;
  gap: 6px;
}
.type-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: var(--xuya-radius-sm);
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent);
}
.type-badge.url { background: var(--xuya-accent-soft); color: var(--xuya-accent); }
.type-badge.email { background: var(--xuya-success-soft); color: var(--xuya-success); }
.type-badge.phone { background: var(--xuya-warn-soft); color: var(--xuya-warn); }
.type-badge.wifi { background: var(--xuya-accent-soft); color: var(--xuya-accent); }
.type-badge.vcard { background: var(--xuya-success-soft); color: var(--xuya-success); }
.type-badge.text { background: var(--xuya-border); color: var(--xuya-text-secondary); }

.result-text {
  margin: 0;
  padding: 12px;
  background: var(--xuya-bg-subtle);
  border: 1px solid var(--xuya-border-light);
  border-radius: var(--xuya-radius-sm);
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  line-height: 1.6;
  color: var(--xuya-text);
  white-space: pre-wrap;
  word-break: break-all;
}
.result-text.link {
  cursor: pointer;
  color: var(--xuya-accent);
  text-decoration: underline;
  text-decoration-style: dashed;
  text-underline-offset: 3px;
}
.result-actions {
  display: flex;
  gap: 8px;
}
.result-placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--xuya-text-tertiary);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  background: var(--xuya-input-bg);
}
.result-placeholder p {
  font-size: 13px;
}

/* 历史 */
.history-section {
  margin-top: 16px;
}
.section-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.6px;
  color: var(--xuya-text-tertiary);
  margin-bottom: 8px;
}
.history-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 180px;
  overflow-y: auto;
}
.history-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 10px;
  font-size: 12px;
  border-radius: var(--xuya-radius-sm);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border-light);
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  text-align: left;
  width: 100%;
  cursor: pointer;
}
.history-item:hover {
  background: var(--xuya-border-light);
}
.hist-type {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 7px;
  border-radius: var(--xuya-radius-sm);
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent);
  flex-shrink: 0;
}
.hist-type.url { background: var(--xuya-accent-soft); color: var(--xuya-accent); }
.hist-type.email { background: var(--xuya-success-soft); color: var(--xuya-success); }
.hist-type.phone { background: var(--xuya-warn-soft); color: var(--xuya-warn); }
.hist-type.wifi { background: var(--xuya-accent-soft); color: var(--xuya-accent); }
.hist-type.vcard { background: var(--xuya-success-soft); color: var(--xuya-success); }
.hist-type.text { background: var(--xuya-border); color: var(--xuya-text-secondary); }
.hist-text {
  flex: 1;
  min-width: 0;
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.hist-time {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  flex-shrink: 0;
}
.clear-history {
  margin-top: 6px;
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
  transition: color var(--xuya-duration-fast) var(--xuya-ease);
}
.clear-history:hover {
  color: var(--xuya-danger);
}

.spin {
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
