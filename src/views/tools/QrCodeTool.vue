<template>
  <ToolShell title="二维码生成" :icon="QrCode" description="文本 / URL 转 QR 码,可调尺寸与纠错级别,下载 PNG。">
    <div class="qr-grid">
      <div class="qr-controls">
        <div class="control-block">
          <label class="ctrl-label">输入内容</label>
          <textarea v-model="text" class="text-input" placeholder="输入文本或 URL, 如 https://example.com" spellcheck="false"></textarea>
          <div class="char-count">{{ text.length }} 字符</div>
        </div>

        <div class="control-row">
          <div class="ctrl-item">
            <label class="ctrl-label">纠错级别</label>
            <div class="seg">
              <button v-for="l in LEVELS" :key="l.v" :class="{ active: level === l.v }" @click="level = l.v">{{ l.label }}</button>
            </div>
          </div>
          <div class="ctrl-item">
            <label class="ctrl-label">尺寸 {{ size }}px</label>
            <input v-model.number="size" type="range" min="160" max="480" step="20" class="range" />
          </div>
        </div>

        <div class="control-row">
          <div class="ctrl-item">
            <label class="ctrl-label">前景色</label>
            <input v-model="fgColor" type="color" class="color-pick" />
          </div>
          <div class="ctrl-item">
            <label class="ctrl-label">背景色</label>
            <input v-model="bgColor" type="color" class="color-pick" />
          </div>
        </div>

        <BaseButton variant="primary" block @click="download" :disabled="!text">
          <Download :size="14" /> 下载 PNG
        </BaseButton>
        <BaseButton variant="ghost" block @click="copyText" :disabled="!text" style="margin-top:8px">
          <Copy :size="14" /> 复制文本
        </BaseButton>
      </div>

      <div class="qr-preview">
        <div class="qr-frame">
          <canvas ref="canvasRef" :width="size" :height="size"></canvas>
          <div v-if="!text" class="qr-placeholder">
            <QrCode :size="48" />
            <p>输入内容后生成二维码</p>
          </div>
        </div>
        <div class="qr-meta">
          <span>纠错: {{ levelLabel }}</span>
          <span>·</span>
          <span>版本: {{ qrVersion || '—' }}</span>
        </div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { QrCode, Download, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';
// 内嵌的最小 QR 编码器 (byte 模式, ISO-8859-1 近似为 UTF-8 字节)
import { encodeQR } from '@/utils/qrcode';

const toast = useToast();
type ECLevel = 'L' | 'M' | 'Q' | 'H';
const LEVELS: { v: ECLevel; label: string }[] = [
  { v: 'L', label: 'L (7%)' },
  { v: 'M', label: 'M (15%)' },
  { v: 'Q', label: 'Q (25%)' },
  { v: 'H', label: 'H (30%)' },
];
const levelLabel = ref<ECLevel>('M');

const text = ref('https://github.com');
const level = ref<ECLevel>('M');
const size = ref(280);
const fgColor = ref('#1a1d23');
const bgColor = ref('#ffffff');
const qrVersion = ref(0);

const canvasRef = ref<HTMLCanvasElement | null>(null);

function draw() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  ctx.fillStyle = bgColor.value;
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  if (!text.value.trim()) { qrVersion.value = 0; return; }
  try {
    const matrix = encodeQR(text.value, level.value);
    qrVersion.value = (matrix.size - 21) / 4 + 1;
    const count = matrix.size;
    const margin = 2;
    const total = count + margin * 2;
    const cell = canvas.width / total;
    ctx.fillStyle = fgColor.value;
    for (let y = 0; y < count; y++) {
      for (let x = 0; x < count; x++) {
        if (matrix.get(x, y)) {
          ctx.fillRect((x + margin) * cell, (y + margin) * cell, cell, cell);
        }
      }
    }
  } catch (e) {
    toast.error('二维码生成失败: ' + (e instanceof Error ? e.message : String(e)));
  }
}

watch([text, level, size, fgColor, bgColor], draw);
watch(level, (l) => { levelLabel.value = l; });
onMounted(draw);

function download() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const link = document.createElement('a');
  link.download = `qrcode-${Date.now()}.png`;
  link.href = canvas.toDataURL('image/png');
  link.click();
  toast.success('已下载二维码 PNG');
}
async function copyText() {
  if (text.value) await copyToClipboard(text.value, '已复制文本');
}
</script>

<style scoped>
.qr-grid { display: grid; grid-template-columns: 1fr 320px; gap: 20px; height: 100%; }
.qr-controls { display: flex; flex-direction: column; gap: 14px; }
.control-block { display: flex; flex-direction: column; gap: 6px; }
.ctrl-label { font-size: 12px; font-weight: 600; color: var(--xuya-text-secondary); }
.text-input { width: 100%; min-height: 100px; padding: 12px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); font-size: 13px; line-height: 1.5; resize: vertical; font-family: var(--xuya-font-mono); transition: border-color .12s, box-shadow .12s; }
.text-input:focus { outline: none; border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }
.char-count { font-size: 11px; color: var(--xuya-text-tertiary); text-align: right; }
.control-row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.ctrl-item { display: flex; flex-direction: column; gap: 6px; }
.seg { display: inline-flex; background: var(--xuya-input-bg); border-radius: var(--xuya-radius-sm); padding: 2px; gap: 2px; }
.seg button { flex: 1; padding: 5px 8px; font-size: 11.5px; color: var(--xuya-text-secondary); background: transparent; border: none; border-radius: 4px; transition: .1s; }
.seg button.active { background: var(--xuya-bg-elevated); color: var(--xuya-accent); font-weight: 600; box-shadow: var(--xuya-shadow); }
.range { width: 100%; accent-color: var(--xuya-accent); }
.color-pick { width: 100%; height: 34px; border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-sm); background: none; cursor: pointer; }

.qr-preview { display: flex; flex-direction: column; align-items: center; gap: 12px; justify-content: center; }
.qr-frame { position: relative; padding: 14px; background: var(--xuya-card-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius-lg); }
.qr-frame canvas { display: block; border-radius: var(--xuya-radius-sm); image-rendering: pixelated; }
.qr-placeholder { position: absolute; inset: 14px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; color: var(--xuya-text-tertiary); background: var(--xuya-card-bg); border-radius: var(--xuya-radius-sm); }
.qr-placeholder p { font-size: 12px; }
.qr-meta { font-size: 11px; color: var(--xuya-text-tertiary); display: flex; gap: 6px; }
</style>
