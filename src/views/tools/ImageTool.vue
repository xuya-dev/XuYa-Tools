<template>
  <ToolShell
    title="图片工具"
    :icon="ImageIcon"
    description="图片压缩、缩放、格式转换 (PNG/JPG/WebP)、Base64 互转。"
  >
    <div class="img-grid">
      <!-- 上传区 -->
      <div class="img-upload-col">
        <div
          class="drop-zone"
          :class="{ dragging, 'has-image': !!previewUrl }"
          @dragover.prevent="dragging = true"
          @dragleave.prevent="dragging = false"
          @drop.prevent="onDrop"
          @click="triggerFileInput"
        >
          <input ref="fileInput" type="file" accept="image/*" hidden @change="onFileSelect" />
          <template v-if="previewUrl">
            <img :src="previewUrl" class="preview-img" alt="预览" />
            <button class="clear-img-btn" @click.stop="clearAll">
              <X :size="14" />
            </button>
          </template>
          <template v-else>
            <ImageIcon :size="36" class="drop-icon" />
            <span class="drop-title">点击 / 拖拽上传图片</span>
            <span class="drop-hint">PNG / JPG / WebP / GIF / BMP</span>
          </template>
        </div>
        <div v-if="originalInfo" class="img-info">
          <span>{{ originalInfo.name }}</span>
          <span>{{ originalInfo.w }}×{{ originalInfo.h }}</span>
          <span>{{ formatSize(originalInfo.size) }}</span>
          <span>{{ originalInfo.type }}</span>
        </div>
      </div>

      <!-- 控制区 -->
      <div class="img-controls-col">
        <div class="control-group">
          <label class="ctrl-label">输出格式</label>
          <div class="seg">
            <button
              v-for="f in OUTPUT_FORMATS"
              :key="f"
              :class="{ active: outFormat === f }"
              @click="outFormat = f"
            >
              {{ f.toUpperCase() }}
            </button>
          </div>
        </div>

        <div class="control-group">
          <label class="ctrl-label">缩放比例 {{ scalePct }}%</label>
          <input v-model.number="scalePct" type="range" min="1" max="100" step="1" class="range" />
        </div>

        <div v-if="outFormat !== 'png'" class="control-group">
          <label class="ctrl-label">质量 {{ quality }}%</label>
          <input v-model.number="quality" type="range" min="1" max="100" step="1" class="range" />
        </div>

        <div class="control-group">
          <label class="ctrl-label">旋转 {{ rotation }}°</label>
          <div class="seg">
            <button :class="{ active: rotation === 0 }" @click="rotation = 0">0°</button>
            <button @click="rotate(-90)">
              <RotateCcw :size="13" />
              90°
            </button>
            <button @click="rotate(90)">
              <RotateCw :size="13" />
              90°
            </button>
            <button :class="{ active: rotation === 180 }" @click="rotation = 180">180°</button>
          </div>
        </div>

        <div class="control-group">
          <label class="ctrl-label">
            翻转 / 滤镜
            <button class="reset-btn" @click="resetTransforms">重置</button>
          </label>
          <div class="seg">
            <button :class="{ active: flipH }" @click="flipH = !flipH">
              <FlipHorizontal :size="13" />
              水平
            </button>
            <button :class="{ active: flipV }" @click="flipV = !flipV">
              <FlipVertical :size="13" />
              垂直
            </button>
            <button :class="{ active: grayscale }" @click="grayscale = !grayscale">灰度</button>
          </div>
        </div>

        <div class="control-actions">
          <BaseButton variant="primary" :disabled="!previewUrl || processing" @click="processImage">
            <RefreshCw v-if="processing" :size="14" class="spin" />
            {{ processing ? '处理中' : '处理图片' }}
          </BaseButton>
        </div>

        <!-- 结果 -->
        <div v-if="resultUrl" class="result-section">
          <div class="result-info">
            <span>{{ resultInfo.w }}×{{ resultInfo.h }}</span>
            <span>{{ formatSize(resultInfo.size) }}</span>
            <span v-if="originalInfo" :class="sizeDeltaClass">{{ sizeDeltaText }}</span>
          </div>
          <img :src="resultUrl" class="result-img" alt="结果" />
          <div class="result-actions">
            <BaseButton variant="ghost" :disabled="!resultUrl" @click="downloadResult">
              <Download :size="13" />
              下载
            </BaseButton>
            <BaseButton variant="ghost" :disabled="!resultUrl" @click="copyBase64">
              <Copy :size="13" />
              复制 Base64
            </BaseButton>
          </div>
        </div>

        <!-- Base64 → 图片 -->
        <details class="b64-section">
          <summary>Base64 → 图片</summary>
          <textarea
            v-model="b64Input"
            class="b64-input"
            placeholder="粘贴 data:image/...;base64,... 或纯 Base64 字符串"
            spellcheck="false"
          ></textarea>
          <BaseButton variant="ghost" :disabled="!b64Input.trim()" @click="fromBase64">
            解析为图片
          </BaseButton>
        </details>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import {
  Image as ImageIcon,
  X,
  Download,
  Copy,
  RefreshCw,
  RotateCcw,
  RotateCw,
  FlipHorizontal,
  FlipVertical,
} from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';

const toast = useToast();

const OUTPUT_FORMATS = ['png', 'jpeg', 'webp'] as const;
type OutFormat = (typeof OUTPUT_FORMATS)[number];

const fileInput = ref<HTMLInputElement | null>(null);
const previewUrl = ref('');
const dragging = ref(false);
const processing = ref(false);

const outFormat = ref<OutFormat>('webp');
const scalePct = ref(100);
const quality = ref(80);
const rotation = ref(0);
const flipH = ref(false);
const flipV = ref(false);
const grayscale = ref(false);

function rotate(delta: number) {
  rotation.value = (((rotation.value + delta) % 360) + 360) % 360;
}

function resetTransforms() {
  rotation.value = 0;
  flipH.value = false;
  flipV.value = false;
  grayscale.value = false;
}

const originalInfo = ref<{ name: string; w: number; h: number; size: number; type: string } | null>(
  null,
);
const resultUrl = ref('');
const resultInfo = ref<{ w: number; h: number; size: number }>({ w: 0, h: 0, size: 0 });

const b64Input = ref('');

const sizeDeltaClass = computed(() => {
  if (!originalInfo.value || !resultInfo.value.size) return '';
  return resultInfo.value.size < originalInfo.value.size ? 'delta-down' : 'delta-up';
});

const sizeDeltaText = computed(() => {
  if (!originalInfo.value || !resultInfo.value.size) return '';
  const diff = resultInfo.value.size - originalInfo.value.size;
  const pct = Math.abs((diff / originalInfo.value.size) * 100).toFixed(1);
  return diff < 0 ? `↓${pct}%` : `↑${pct}%`;
});

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

function triggerFileInput() {
  if (!previewUrl.value) fileInput.value?.click();
}

function onFileSelect(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (file) loadImage(file);
}

function onDrop(e: DragEvent) {
  dragging.value = false;
  const file = e.dataTransfer?.files?.[0];
  if (file) loadImage(file);
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
    originalInfo.value = {
      name: file.name,
      w: img.naturalWidth,
      h: img.naturalHeight,
      size: file.size,
      type: file.type,
    };
  };
  img.src = previewUrl.value;
}

async function processImage() {
  if (!previewUrl.value || !originalInfo.value) return;
  processing.value = true;
  try {
    const img = new Image();
    img.src = previewUrl.value;
    await new Promise((resolve) => {
      img.onload = resolve;
    });

    const scaledW = Math.max(1, Math.round(img.naturalWidth * (scalePct.value / 100)));
    const scaledH = Math.max(1, Math.round(img.naturalHeight * (scalePct.value / 100)));
    const rot = ((rotation.value % 360) + 360) % 360;
    const swapDims = rot === 90 || rot === 270;
    const targetW = swapDims ? scaledH : scaledW;
    const targetH = swapDims ? scaledW : scaledH;

    const canvas = document.createElement('canvas');
    canvas.width = targetW;
    canvas.height = targetH;
    const ctx = canvas.getContext('2d')!;
    ctx.imageSmoothingQuality = 'high';
    if (grayscale.value) ctx.filter = 'grayscale(1)';
    ctx.translate(targetW / 2, targetH / 2);
    ctx.rotate((rot * Math.PI) / 180);
    ctx.scale(flipH.value ? -1 : 1, flipV.value ? -1 : 1);
    ctx.drawImage(img, -scaledW / 2, -scaledH / 2, scaledW, scaledH);

    const mime =
      outFormat.value === 'png'
        ? 'image/png'
        : outFormat.value === 'jpeg'
          ? 'image/jpeg'
          : 'image/webp';
    const q = outFormat.value === 'png' ? undefined : quality.value / 100;

    canvas.toBlob(
      (blob) => {
        if (!blob) {
          toast.error('图片处理失败');
          processing.value = false;
          return;
        }
        if (resultUrl.value) URL.revokeObjectURL(resultUrl.value);
        resultUrl.value = URL.createObjectURL(blob);
        resultInfo.value = { w: targetW, h: targetH, size: blob.size };
        processing.value = false;
        toast.success('图片处理完成');
      },
      mime,
      q,
    );
  } catch (e) {
    toast.error('处理失败: ' + e);
    processing.value = false;
  }
}

function downloadResult() {
  if (!resultUrl.value) return;
  const ext = outFormat.value === 'jpeg' ? 'jpg' : outFormat.value;
  const a = document.createElement('a');
  a.href = resultUrl.value;
  a.download = `processed-${Date.now()}.${ext}`;
  a.click();
  toast.success('已下载');
}

async function copyBase64() {
  if (!resultUrl.value) return;
  try {
    const resp = await fetch(resultUrl.value);
    const blob = await resp.blob();
    const reader = new FileReader();
    reader.onload = async () => {
      await copyToClipboard(reader.result as string, '已复制 Base64');
    };
    reader.readAsDataURL(blob);
  } catch {
    toast.error('复制失败');
  }
}

function fromBase64() {
  let data = b64Input.value.trim();
  if (!data) return;
  if (!data.startsWith('data:')) {
    data = `data:image/png;base64,${data}`;
  }
  clearAll();
  previewUrl.value = data;
  const img = new Image();
  img.onload = () => {
    originalInfo.value = {
      name: 'from-base64',
      w: img.naturalWidth,
      h: img.naturalHeight,
      size: Math.ceil((data.length * 3) / 4),
      type: 'image/png',
    };
  };
  img.onerror = () => toast.error('无效的 Base64 图片数据');
  img.src = data;
}

function clearAll() {
  if (previewUrl.value.startsWith('blob:')) URL.revokeObjectURL(previewUrl.value);
  if (resultUrl.value) URL.revokeObjectURL(resultUrl.value);
  previewUrl.value = '';
  resultUrl.value = '';
  originalInfo.value = null;
  if (fileInput.value) fileInput.value.value = '';
}
</script>

<style scoped>
.img-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}
.img-upload-col {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.drop-zone {
  min-height: 260px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 2px dashed var(--xuya-border-strong);
  border-radius: var(--xuya-radius-lg);
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
  max-height: 300px;
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
.clear-img-btn {
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
.clear-img-btn:hover {
  background: var(--xuya-danger);
}

.img-info {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  font-size: 12px;
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
  padding: 0 4px;
}

.img-controls-col {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.control-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.ctrl-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.reset-btn {
  font-size: 10px;
  font-weight: 500;
  color: var(--xuya-text-tertiary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: color var(--xuya-duration-fast);
}
.reset-btn:hover {
  color: var(--xuya-accent);
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
.range {
  width: 100%;
  accent-color: var(--xuya-accent);
}

.control-actions {
  margin-top: 4px;
}

.result-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.result-info {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  font-size: 12px;
  font-family: var(--xuya-font-mono);
  color: var(--xuya-text-secondary);
}
.delta-down {
  color: var(--xuya-success);
  font-weight: 600;
}
.delta-up {
  color: var(--xuya-warn);
  font-weight: 600;
}
.result-img {
  max-width: 100%;
  max-height: 200px;
  object-fit: contain;
  border-radius: var(--xuya-radius-sm);
}
.result-actions {
  display: flex;
  gap: 8px;
}

.b64-section {
  margin-top: 8px;
  padding: 12px 14px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.b64-section summary {
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.b64-input {
  width: 100%;
  min-height: 70px;
  margin-top: 10px;
  padding: 8px 10px;
  font-family: var(--xuya-font-mono);
  font-size: 11px;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-card-bg);
  color: var(--xuya-text);
  resize: vertical;
  word-break: break-all;
}
.b64-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
}

.spin {
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 700px) {
  .img-grid {
    grid-template-columns: 1fr;
  }
}
</style>
