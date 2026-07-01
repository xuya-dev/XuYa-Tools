<template>
  <ToolShell
    title="二维码解码"
    :icon="ScanLine"
    description="上传 / 拖拽 / 粘贴二维码图片，自动识别并提取内容，智能解析 WiFi、名片、链接。"
  >
    <div class="decode-grid">
      <div class="upload-col">
        <div class="col-head">
          <span>图片</span>
          <span v-if="previewUrl" class="stat">{{ imgInfo }}</span>
        </div>
        <div
          class="drop-zone"
          :class="{ dragging, 'has-image': !!previewUrl }"
          @dragover.prevent="dragging = true"
          @dragleave.prevent="dragging = false"
          @drop.prevent="onDrop"
          @click="triggerFileInput"
        >
          <input ref="fileInputRef" type="file" accept="image/*" hidden @change="onFileSelect" />
          <template v-if="previewUrl">
            <img :src="previewUrl" class="preview-img" alt="二维码预览" />
            <button class="clear-btn" title="移除图片" @click.stop="clearAll">
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

      <div class="result-col">
        <div class="col-head">
          <span>解码结果</span>
          <div v-if="result" class="head-actions">
            <span class="type-badge" :class="resultType">{{ resultTypeLabel }}</span>
            <BaseButton variant="ghost" @click="copyResult">
              <Copy :size="13" />
              复制
            </BaseButton>
          </div>
        </div>

        <div v-if="decoding" class="result-box loading">
          <Loader2 :size="20" class="spin" />
          解码中…
        </div>

        <div v-else-if="error" class="result-box error">
          <AlertCircle :size="20" />
          <span>{{ error }}</span>
        </div>

        <div v-else-if="result" class="result-box">
          <!-- WiFi 结构化展示 -->
          <template v-if="wifiData">
            <div class="parsed-section">
              <div class="parsed-row">
                <span class="parsed-label">网络名称</span>
                <span class="parsed-value mono">{{ wifiData.ssid || '—' }}</span>
                <button
                  v-if="wifiData.ssid"
                  class="copy-mini"
                  @click="copyText(wifiData.ssid, 'SSID')"
                >
                  <Copy :size="11" />
                </button>
              </div>
              <div class="parsed-row">
                <span class="parsed-label">密码</span>
                <span class="parsed-value mono">{{ wifiData.password || '(无密码)' }}</span>
                <button
                  v-if="wifiData.password"
                  class="copy-mini"
                  @click="copyText(wifiData.password, '密码')"
                >
                  <Copy :size="11" />
                </button>
              </div>
              <div class="parsed-row">
                <span class="parsed-label">加密方式</span>
                <span class="parsed-value">{{ wifiData.encryption }}</span>
              </div>
            </div>
          </template>

          <!-- vCard 结构化展示 -->
          <template v-else-if="vcardData">
            <div class="parsed-section">
              <div v-if="vcardData.name" class="parsed-row">
                <span class="parsed-label">姓名</span>
                <span class="parsed-value">{{ vcardData.name }}</span>
              </div>
              <div v-if="vcardData.phone" class="parsed-row">
                <span class="parsed-label">电话</span>
                <span class="parsed-value mono">{{ vcardData.phone }}</span>
                <button class="copy-mini" @click="copyText(vcardData.phone, '电话')">
                  <Copy :size="11" />
                </button>
              </div>
              <div v-if="vcardData.email" class="parsed-row">
                <span class="parsed-label">邮箱</span>
                <span class="parsed-value mono">{{ vcardData.email }}</span>
                <button class="copy-mini" @click="copyText(vcardData.email, '邮箱')">
                  <Copy :size="11" />
                </button>
              </div>
              <div v-if="vcardData.org" class="parsed-row">
                <span class="parsed-label">组织</span>
                <span class="parsed-value">{{ vcardData.org }}</span>
              </div>
            </div>
          </template>

          <!-- 原始文本 -->
          <div class="raw-section">
            <span class="raw-label">原始内容</span>
            <pre
              class="result-text"
              :class="{ link: isLink }"
              @click="isLink ? openLink() : undefined"
              >{{ result }}</pre>
          </div>

          <!-- 操作按钮 -->
          <div v-if="isLink" class="result-actions">
            <BaseButton variant="ghost" @click="openLink">
              <ExternalLink :size="13" />
              打开链接
            </BaseButton>
          </div>
        </div>

        <div v-else class="result-box placeholder">
          <ScanLine :size="40" />
          <p>上传二维码图片后自动解码</p>
        </div>
      </div>
    </div>

    <div v-if="history.length" class="history-section">
      <div class="section-label">
        <span>解码历史</span>
        <button class="clear-history" @click="history = []">清空</button>
      </div>
      <div class="history-list">
        <button v-for="(h, i) in history" :key="i" class="history-item" @click="restoreResult(h)">
          <span class="hist-badge" :class="h.type">{{ h.typeLabel }}</span>
          <span class="hist-text">{{ h.text }}</span>
          <span class="hist-time">{{ h.time }}</span>
        </button>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { ScanLine, Upload, X, Copy, AlertCircle, Loader2, ExternalLink } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';
import jsQR from 'jsqr';

const toast = useToast();

const fileInputRef = ref<HTMLInputElement | null>(null);
const previewUrl = ref('');
const imageData = ref<ImageData | null>(null);
const result = ref('');
const error = ref('');
const decoding = ref(false);
const dragging = ref(false);
const imgInfo = ref('');

interface HistoryItem {
  text: string;
  fullText: string;
  type: string;
  typeLabel: string;
  time: string;
}

const history = useToolState<HistoryItem[]>('qrdecode', 'history', []);

const resultType = computed(() => {
  if (!result.value) return 'text';
  if (/^https?:\/\//i.test(result.value)) return 'url';
  if (/^mailto:/i.test(result.value)) return 'email';
  if (/^tel:/i.test(result.value)) return 'phone';
  if (/^WIFI:/i.test(result.value)) return 'wifi';
  if (/^BEGIN:VCARD/i.test(result.value)) return 'vcard';
  if (/^sms:/i.test(result.value)) return 'sms';
  return 'text';
});

const resultTypeLabel = computed(() => {
  const map: Record<string, string> = {
    url: '链接',
    email: '邮箱',
    phone: '电话',
    wifi: 'WiFi',
    vcard: '名片',
    sms: '短信',
    text: '文本',
  };
  return map[resultType.value] ?? '文本';
});

const isLink = computed(() => resultType.value === 'url');

interface WifiInfo {
  ssid: string;
  password: string;
  encryption: string;
  hidden: boolean;
}
const wifiData = computed<WifiInfo | null>(() => {
  if (resultType.value !== 'wifi') return null;
  const m = result.value.match(/WIFI:(?:T:([^;]*);)?S:([^;]*);(?:P:([^;]*);)?(?:H:([^;]*);)?;?/i);
  if (!m) return null;
  return {
    encryption: (m[1] || 'nopass').toUpperCase(),
    ssid: m[2] || '',
    password: m[3] || '',
    hidden: (m[4] || '').toLowerCase() === 'true',
  };
});

interface VcardInfo {
  name: string;
  phone: string;
  email: string;
  org: string;
}
const vcardData = computed<VcardInfo | null>(() => {
  if (resultType.value !== 'vcard') return null;
  const get = (key: string) => {
    const m = result.value.match(new RegExp(`${key}[^:]*:(.+)`, 'i'));
    return m?.[1]?.trim() || '';
  };
  return {
    name: get('FN'),
    phone: get('TEL'),
    email: get('EMAIL'),
    org: get('ORG'),
  };
});

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

function loadImage(file: File) {
  if (!file.type.startsWith('image/')) {
    toast.error('请选择图片文件');
    return;
  }

  clearAll();
  previewUrl.value = URL.createObjectURL(file);
  imgInfo.value = `${(file.size / 1024).toFixed(1)} KB`;

  const img = new Image();
  img.onload = () => {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      error.value = '无法创建 Canvas 上下文';
      return;
    }

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
    imgInfo.value = `${width}×${height} · ${(file.size / 1024).toFixed(1)} KB`;
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
        error.value = '未在图片中找到二维码，请确保图片清晰且包含完整二维码';
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
    url: '链接',
    email: '邮箱',
    phone: '电话',
    wifi: 'WiFi',
    vcard: '名片',
    sms: '短信',
    text: '文本',
  };
  const now = new Date();
  const time = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;

  history.value = [
    {
      text: text.length > 60 ? text.slice(0, 60) + '…' : text,
      fullText: text,
      type,
      typeLabel: typeLabels[type] ?? '文本',
      time,
    },
    ...history.value.filter((h) => h.fullText !== text),
  ].slice(0, 20);
}

function restoreResult(h: HistoryItem) {
  result.value = h.fullText;
  error.value = '';
}

async function copyResult() {
  if (result.value) await copyToClipboard(result.value, '已复制解码结果');
}

async function copyText(text: string, label: string) {
  await copyToClipboard(text, `已复制${label}`);
}

async function openLink() {
  if (!isLink.value) return;
  try {
    const opener = await import('@tauri-apps/plugin-opener');
    await opener.openUrl(result.value);
  } catch {
    window.open(result.value, '_blank');
  }
}

function clearAll() {
  if (previewUrl.value) URL.revokeObjectURL(previewUrl.value);
  previewUrl.value = '';
  imageData.value = null;
  result.value = '';
  error.value = '';
  imgInfo.value = '';
  if (fileInputRef.value) fileInputRef.value.value = '';
}

onMounted(() => window.addEventListener('paste', globalPaste));
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

.col-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.head-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.stat {
  font-size: 10.5px;
  font-weight: 400;
  opacity: 0.75;
  font-family: var(--xuya-font-mono);
}

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
  color: #fff;
  border: none;
  cursor: pointer;
  transition: background var(--xuya-duration-fast);
}
.clear-btn:hover {
  background: var(--xuya-danger);
}

.type-badge {
  font-size: 10.5px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 99px;
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent);
}
.type-badge.url {
  background: var(--xuya-info-soft);
  color: var(--xuya-info);
}
.type-badge.email {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.type-badge.phone {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}
.type-badge.wifi {
  background: var(--xuya-purple-soft);
  color: var(--xuya-purple);
}
.type-badge.vcard {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.type-badge.sms {
  background: var(--xuya-info-soft);
  color: var(--xuya-info);
}
.type-badge.text {
  background: var(--xuya-bg-subtle);
  color: var(--xuya-text-secondary);
}

.result-box {
  flex: 1;
  min-height: 280px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  background: var(--xuya-input-bg);
  padding: 14px;
  overflow: auto;
}
.result-box.loading,
.result-box.error {
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: 10px;
  font-size: 13px;
  color: var(--xuya-text-tertiary);
}
.result-box.error {
  color: var(--xuya-danger);
}
.result-box.placeholder {
  align-items: center;
  justify-content: center;
  color: var(--xuya-text-tertiary);
}
.result-box.placeholder p {
  font-size: 13px;
}

.parsed-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  background: var(--xuya-bg-subtle);
  border: 1px solid var(--xuya-border-light);
  border-radius: var(--xuya-radius-sm);
}
.parsed-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.parsed-label {
  width: 64px;
  font-size: 11.5px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
  flex-shrink: 0;
}
.parsed-value {
  flex: 1;
  font-size: 13px;
  color: var(--xuya-text);
  word-break: break-all;
}
.parsed-value.mono {
  font-family: var(--xuya-font-mono);
}
.copy-mini {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  color: var(--xuya-text-tertiary);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-bg-elevated);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
  flex-shrink: 0;
}
.copy-mini:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}

.raw-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.raw-label {
  font-size: 10.5px;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
}
.result-text {
  margin: 0;
  padding: 10px 12px;
  background: var(--xuya-bg-subtle);
  border: 1px solid var(--xuya-border-light);
  border-radius: var(--xuya-radius-sm);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
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

.history-section {
  margin-top: 16px;
}
.section-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--xuya-text-tertiary);
  margin-bottom: 8px;
}
.clear-history {
  font-size: 10px;
  text-transform: none;
  letter-spacing: 0;
  color: var(--xuya-text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
}
.clear-history:hover {
  color: var(--xuya-danger);
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
  transition: all var(--xuya-duration-fast);
  text-align: left;
  width: 100%;
  cursor: pointer;
}
.history-item:hover {
  background: var(--xuya-accent-soft);
  border-color: var(--xuya-accent);
}
.hist-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 7px;
  border-radius: var(--xuya-radius-sm);
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent);
  flex-shrink: 0;
}
.hist-badge.url {
  background: var(--xuya-info-soft);
  color: var(--xuya-info);
}
.hist-badge.email {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.hist-badge.phone {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}
.hist-badge.wifi {
  background: var(--xuya-purple-soft);
  color: var(--xuya-purple);
}
.hist-badge.vcard {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.hist-badge.text {
  background: var(--xuya-bg-subtle);
  color: var(--xuya-text-secondary);
}
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

.spin {
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
