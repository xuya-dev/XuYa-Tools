<template>
  <ToolShell
    title="二维码生成"
    :icon="QrCode"
    description="文本 / URL / WiFi 转 QR 码，支持中间 LOGO、圆点样式、自定义颜色，导出 PNG / SVG。"
  >
    <div class="qr-grid">
      <div class="qr-controls">
        <div class="type-row">
          <span class="ctrl-label">类型</span>
          <div class="type-pills">
            <button
              v-for="t in TYPES"
              :key="t.id"
              class="type-pill"
              :class="{ active: activeType === t.id }"
              @click="applyType(t.id)"
            >
              {{ t.label }}
            </button>
          </div>
        </div>

        <div class="input-block">
          <label class="ctrl-label">
            内容
            <span class="char-count">{{ text.length }} 字符</span>
          </label>
          <textarea
            v-model="text"
            class="text-input"
            :placeholder="inputPlaceholder"
            spellcheck="false"
          ></textarea>
        </div>

        <div class="settings-row">
          <div class="setting">
            <label class="ctrl-label">纠错级别</label>
            <div class="seg">
              <button
                v-for="l in LEVELS"
                :key="l.v"
                :class="{ active: level === l.v }"
                @click="level = l.v"
              >
                {{ l.v }}
              </button>
            </div>
          </div>
          <div class="setting">
            <label class="ctrl-label">点样式</label>
            <div class="seg">
              <button
                v-for="d in DOT_STYLES"
                :key="d.v"
                :class="{ active: dotStyle === d.v }"
                @click="dotStyle = d.v"
              >
                {{ d.label }}
              </button>
            </div>
          </div>
        </div>

        <div class="settings-row">
          <div class="setting">
            <label class="ctrl-label">尺寸 {{ size }}px</label>
            <input v-model.number="size" type="range" min="128" max="512" step="16" class="range" />
          </div>
          <div class="setting">
            <label class="ctrl-label">边距 {{ margin }}</label>
            <input v-model.number="margin" type="range" min="0" max="8" step="1" class="range" />
          </div>
        </div>

        <div class="settings-row">
          <div class="color-item">
            <label class="ctrl-label">前景</label>
            <input v-model="fgColor" type="color" class="color-pick" />
          </div>
          <div class="color-item">
            <label class="ctrl-label">背景</label>
            <input v-model="bgColor" type="color" class="color-pick" />
          </div>
        </div>

        <div class="logo-section">
          <label class="ctrl-label">中间 LOGO</label>
          <div class="logo-controls">
            <button class="logo-btn" @click="logoInput?.click()">
              <ImageIcon :size="14" />
              {{ logoSrc ? '更换图片' : '上传图片' }}
            </button>
            <button v-if="logoSrc" class="logo-btn danger" @click="removeLogo">移除</button>
            <input
              ref="logoInput"
              type="file"
              accept="image/png,image/jpeg,image/svg+xml,image/webp"
              hidden
              @change="onLogoSelected"
            />
          </div>
          <div v-if="logoSrc" class="logo-size-row">
            <label class="ctrl-label">LOGO 大小 {{ logoSize }}%</label>
            <input
              v-model.number="logoSize"
              type="range"
              min="10"
              max="30"
              step="1"
              class="range"
            />
          </div>
          <p v-if="logoSrc" class="logo-hint">建议纠错级别设为 H 以保证可识别性</p>
        </div>

        <div class="action-row">
          <BaseButton variant="primary" :disabled="!text.trim()" @click="downloadPng">
            <Download :size="14" />
            PNG
          </BaseButton>
          <BaseButton variant="ghost" :disabled="!text.trim()" @click="downloadSvg">
            <Download :size="14" />
            SVG
          </BaseButton>
          <BaseButton variant="ghost" :disabled="!text.trim()" @click="copyImage">
            <Copy :size="14" />
            复制
          </BaseButton>
          <BaseButton variant="ghost" @click="text = ''">清空</BaseButton>
        </div>
      </div>

      <div class="qr-preview">
        <div class="qr-frame">
          <canvas
            ref="canvasRef"
            :style="{ width: displaySize + 'px', height: displaySize + 'px' }"
          ></canvas>
          <div v-if="!text.trim()" class="qr-placeholder">
            <QrCode :size="48" />
            <p>输入内容后实时生成</p>
          </div>
          <div v-if="errorMsg" class="qr-error">{{ errorMsg }}</div>
        </div>
        <div class="qr-meta">
          <span>纠错: {{ level }}</span>
          <span>·</span>
          <span>版本: {{ qrVersion || '—' }}</span>
          <span>·</span>
          <span>{{ size }}px</span>
        </div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import QRCode from 'qrcode';
import { QrCode, Download, Copy, Image as ImageIcon } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToast } from '@/composables/useToast';
import { useToolState } from '@/composables/useToolState';

const toast = useToast();

type ECLevel = 'L' | 'M' | 'Q' | 'H';
const LEVELS: { v: ECLevel; label: string }[] = [
  { v: 'L', label: '低 7%' },
  { v: 'M', label: '中 15%' },
  { v: 'Q', label: '高 25%' },
  { v: 'H', label: '最高 30%' },
];

type DotStyle = 'square' | 'rounded' | 'dots';
const DOT_STYLES: { v: DotStyle; label: string }[] = [
  { v: 'square', label: '方块' },
  { v: 'rounded', label: '圆角' },
  { v: 'dots', label: '圆点' },
];

type QrType = 'text' | 'url' | 'wifi' | 'email' | 'phone' | 'sms';
const TYPES: { id: QrType; label: string }[] = [
  { id: 'text', label: '文本' },
  { id: 'url', label: 'URL' },
  { id: 'wifi', label: 'WiFi' },
  { id: 'email', label: '邮箱' },
  { id: 'phone', label: '电话' },
  { id: 'sms', label: '短信' },
];

const text = useToolState('qrcode', 'text', '');
const level = useToolState<ECLevel>('qrcode', 'level', 'M');
const size = useToolState('qrcode', 'size', 320);
const margin = useToolState('qrcode', 'margin', 2);
const fgColor = useToolState('qrcode', 'fg', '#1a1d23');
const bgColor = useToolState('qrcode', 'bg', '#ffffff');
const dotStyle = useToolState<DotStyle>('qrcode', 'dotStyle', 'square');
const logoSrc = ref('');
const logoSize = useToolState('qrcode', 'logoSize', 20);

const activeType = ref<QrType>('text');
const errorMsg = ref('');
const qrVersion = ref(0);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const logoInput = ref<HTMLInputElement | null>(null);

const displaySize = computed(() => Math.min(size.value, 320));
const inputPlaceholder = computed(() => {
  const map: Record<QrType, string> = {
    text: '输入任意文本…',
    url: 'https://example.com',
    wifi: '点击「WiFi」按钮生成模板…',
    email: 'mailto:example@email.com',
    phone: 'tel:+8613800138000',
    sms: 'sms:+8613800138000',
  };
  return map[activeType.value];
});

function applyType(t: QrType) {
  activeType.value = t;
  switch (t) {
    case 'url':
      if (!text.value.startsWith('http')) text.value = 'https://';
      break;
    case 'wifi':
      text.value = 'WIFI:T:WPA;S:网络名称;P:密码;;';
      break;
    case 'email':
      text.value = 'mailto:';
      break;
    case 'phone':
      text.value = 'tel:';
      break;
    case 'sms':
      text.value = 'sms:';
      break;
  }
}

function drawRoundedRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
) {
  const radius = Math.min(r, w / 2, h / 2);
  ctx.beginPath();
  ctx.moveTo(x + radius, y);
  ctx.lineTo(x + w - radius, y);
  ctx.quadraticCurveTo(x + w, y, x + w, y + radius);
  ctx.lineTo(x + w, y + h - radius);
  ctx.quadraticCurveTo(x + w, y + h, x + w - radius, y + h);
  ctx.lineTo(x + radius, y + h);
  ctx.quadraticCurveTo(x, y + h, x, y + h - radius);
  ctx.lineTo(x, y + radius);
  ctx.quadraticCurveTo(x, y, x + radius, y);
  ctx.closePath();
  ctx.fill();
}

function isFinderPattern(x: number, y: number, count: number): boolean {
  const inArea = (cx: number, cy: number) => x >= cx && x < cx + 7 && y >= cy && y < cy + 7;
  return inArea(0, 0) || inArea(count - 7, 0) || inArea(0, count - 7);
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = src;
  });
}

async function generate() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  errorMsg.value = '';

  if (!text.value.trim()) {
    const ctx = canvas.getContext('2d');
    if (ctx) ctx.clearRect(0, 0, canvas.width, canvas.height);
    qrVersion.value = 0;
    return;
  }

  try {
    const qrData = QRCode.create(text.value, { errorCorrectionLevel: level.value });
    qrVersion.value = qrData.version;
    const matrix = qrData.modules;
    const count = matrix.size;

    const renderSize = size.value * 2;
    canvas.width = renderSize;
    canvas.height = renderSize;
    const ctx = canvas.getContext('2d')!;
    ctx.imageSmoothingEnabled = false;

    const marginModules = margin.value;
    const totalModules = count + marginModules * 2;
    const cell = renderSize / totalModules;

    ctx.fillStyle = bgColor.value;
    ctx.fillRect(0, 0, renderSize, renderSize);

    ctx.fillStyle = fgColor.value;
    for (let y = 0; y < count; y++) {
      for (let x = 0; x < count; x++) {
        if (!matrix.get(x, y)) continue;
        const px = (x + marginModules) * cell;
        const py = (y + marginModules) * cell;
        const isFinder = isFinderPattern(x, y, count);

        if (dotStyle.value === 'square' || isFinder) {
          ctx.fillRect(px, py, cell + 0.5, cell + 0.5);
        } else if (dotStyle.value === 'rounded') {
          drawRoundedRect(ctx, px, py, cell + 0.5, cell + 0.5, cell * 0.3);
        } else {
          ctx.beginPath();
          ctx.arc(px + cell / 2, py + cell / 2, cell * 0.42, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    }

    if (logoSrc.value) {
      try {
        const img = await loadImage(logoSrc.value);
        const logoPx = renderSize * (logoSize.value / 100);
        const cx = (renderSize - logoPx) / 2;
        const cy = (renderSize - logoPx) / 2;
        const pad = logoPx * 0.12;
        ctx.fillStyle = bgColor.value;
        drawRoundedRect(ctx, cx - pad, cy - pad, logoPx + pad * 2, logoPx + pad * 2, pad * 0.5);
        ctx.drawImage(img, cx, cy, logoPx, logoPx);
      } catch {
        /* logo load failed */
      }
    }
  } catch (e) {
    errorMsg.value = e instanceof Error ? e.message : String(e);
    qrVersion.value = 0;
  }
}

function onLogoSelected(e: Event) {
  const target = e.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;
  if (file.size > 2 * 1024 * 1024) {
    toast.error('LOGO 图片过大 (>2MB)');
    target.value = '';
    return;
  }
  const reader = new FileReader();
  reader.onload = () => {
    logoSrc.value = reader.result as string;
    generate();
    toast.success('LOGO 已添加');
  };
  reader.readAsDataURL(file);
  target.value = '';
}

function removeLogo() {
  logoSrc.value = '';
  generate();
}

function downloadPng() {
  const canvas = canvasRef.value;
  if (!canvas || !text.value.trim()) return;
  const link = document.createElement('a');
  link.download = `qrcode-${Date.now()}.png`;
  link.href = canvas.toDataURL('image/png');
  link.click();
  toast.success('已下载 PNG');
}

async function downloadSvg() {
  if (!text.value.trim()) return;
  try {
    const svg = await QRCode.toString(text.value, {
      type: 'svg',
      margin: margin.value,
      errorCorrectionLevel: level.value,
      color: { dark: fgColor.value, light: bgColor.value },
    });
    const blob = new Blob([svg], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `qrcode-${Date.now()}.svg`;
    a.click();
    URL.revokeObjectURL(url);
    toast.success('已下载 SVG');
  } catch {
    toast.error('SVG 导出失败');
  }
}

async function copyImage() {
  const canvas = canvasRef.value;
  if (!canvas || !text.value.trim()) return;
  try {
    const blob = await new Promise<Blob | null>((resolve) =>
      canvas.toBlob((b) => resolve(b), 'image/png'),
    );
    if (!blob) throw new Error('生成图片失败');
    await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
    toast.success('已复制二维码图片');
  } catch {
    toast.error('复制图片失败，浏览器可能不支持');
  }
}

watch([text, level, size, margin, fgColor, bgColor, dotStyle, logoSize], generate, {
  immediate: true,
});
watch(logoSrc, () => generate());
</script>

<style scoped>
.qr-grid {
  display: grid;
  grid-template-columns: 1fr 340px;
  gap: 24px;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.qr-controls {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-width: 0;
  overflow-y: auto;
  padding-right: 4px;
}

.type-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.type-pills {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}
.type-pill {
  padding: 4px 12px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: 99px;
  cursor: pointer;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  white-space: nowrap;
}
.type-pill:hover {
  border-color: var(--xuya-accent);
  color: var(--xuya-text);
}
.type-pill.active {
  background: var(--xuya-accent-soft);
  border-color: var(--xuya-accent);
  color: var(--xuya-accent);
  font-weight: 600;
}

.ctrl-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.char-count {
  font-size: 10.5px;
  font-weight: 400;
  opacity: 0.7;
}

.input-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.text-input {
  width: 100%;
  min-height: 80px;
  max-height: 160px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-size: 13px;
  line-height: 1.5;
  resize: vertical;
  font-family: var(--xuya-font-mono);
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.text-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.text-input::placeholder {
  color: var(--xuya-text-tertiary);
}

.settings-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
.setting {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}
.seg {
  display: inline-flex;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  padding: 2px;
  gap: 2px;
}
.seg button {
  flex: 1;
  padding: 5px 8px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-radius: var(--xuya-radius-sm);
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  white-space: nowrap;
}
.seg button:hover {
  color: var(--xuya-text);
}
.seg button.active {
  background: var(--xuya-bg-elevated);
  color: var(--xuya-accent);
  font-weight: 700;
  box-shadow: var(--xuya-shadow-sm);
}
.range {
  width: 100%;
  accent-color: var(--xuya-accent);
}

.color-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.color-pick {
  width: 100%;
  height: 34px;
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  background: none;
  cursor: pointer;
  padding: 2px;
}

.logo-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
}
.logo-controls {
  display: flex;
  gap: 8px;
}
.logo-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 6px 12px;
  font-size: 12px;
  color: var(--xuya-text);
  background: var(--xuya-bg-elevated);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.logo-btn:hover {
  border-color: var(--xuya-accent);
  color: var(--xuya-accent);
}
.logo-btn.danger:hover {
  border-color: var(--xuya-danger);
  color: var(--xuya-danger);
}
.logo-size-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.logo-hint {
  font-size: 11px;
  color: var(--xuya-warn);
}

.action-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.qr-preview {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  justify-content: center;
  min-width: 0;
}
.qr-frame {
  position: relative;
  padding: 14px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-lg);
  box-shadow: var(--xuya-shadow);
}
.qr-frame canvas {
  display: block;
  border-radius: var(--xuya-radius-sm);
  image-rendering: pixelated;
}
.qr-placeholder {
  position: absolute;
  inset: 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--xuya-text-tertiary);
  background: var(--xuya-card-bg);
  border-radius: var(--xuya-radius-sm);
}
.qr-placeholder p {
  font-size: 12px;
}
.qr-error {
  position: absolute;
  inset: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 20px;
  font-size: 12px;
  color: var(--xuya-danger);
  background: var(--xuya-danger-soft);
  border-radius: var(--xuya-radius-sm);
}
.qr-meta {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  display: flex;
  gap: 6px;
  font-family: var(--xuya-font-mono);
}

@media (max-width: 600px) {
  .qr-grid {
    grid-template-columns: 1fr;
  }
}
</style>
