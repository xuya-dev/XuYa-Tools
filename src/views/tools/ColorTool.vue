<template>
  <ToolShell
    title="颜色工具"
    :icon="Palette"
    description="HEX / RGB / HSL / HSV / CMYK 互转、屏幕取色、配色方案、WCAG 对比度检测。"
  >
    <!-- 预览 + 格式 -->
    <div class="color-top">
      <div class="swatch-area">
        <div class="swatch" :style="{ background: hex }">
          <input
            type="color"
            :value="hex"
            class="native-picker"
            @input="setHex(($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="swatch-actions">
          <button v-if="hasEyeDropper" class="action-btn primary" @click="pickFromScreen">
            <Pipette :size="15" />
            屏幕取色
          </button>
          <button class="action-btn" @click="setHex(randomHex())">
            <Shuffle :size="14" />
            随机
          </button>
        </div>
      </div>

      <div class="formats">
        <div v-for="f in formatList" :key="f.key" class="fmt-row">
          <span class="fmt-label">{{ f.label }}</span>
          <input
            class="fmt-input"
            :value="f.value"
            spellcheck="false"
            @change="setFromFormat(f.key, ($event.target as HTMLInputElement).value)"
          />
          <button class="mini-btn" :title="`复制 ${f.label}`" @click="copy(f.value)">
            <Copy :size="13" />
          </button>
        </div>
      </div>
    </div>

    <!-- 明暗变化 -->
    <div class="section">
      <div class="section-label">明暗渐变</div>
      <div class="shade-row">
        <button
          v-for="(c, i) in shades"
          :key="'s' + i"
          class="swatch-btn"
          :style="{ background: c }"
          :title="c.toUpperCase()"
          @click="setHex(c)"
        >
          <span class="swatch-label">{{ c.toUpperCase() }}</span>
        </button>
      </div>
    </div>

    <!-- 配色方案 -->
    <div class="section">
      <div class="section-label">配色方案</div>
      <div class="scheme-list">
        <div v-for="(sc, si) in schemes" :key="si" class="scheme-row">
          <span class="scheme-name">{{ sc.name }}</span>
          <div class="scheme-swatches">
            <button
              v-for="(c, ci) in sc.colors"
              :key="ci"
              class="swatch-btn sm"
              :style="{ background: c }"
              :title="c.toUpperCase()"
              @click="setHex(c)"
            >
              <span class="swatch-label">{{ c.toUpperCase() }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 对比度检测 -->
    <div class="section">
      <div class="section-label">对比度检测 (WCAG)</div>
      <div class="contrast-grid">
        <div class="contrast-card" :style="{ background: hex, color: '#ffffff' }">
          <span class="c-label">白字</span>
          <span class="c-text">Aa 文字示例</span>
          <span class="c-badge" :class="contrastWhite.aa ? 'pass' : 'fail'">
            {{ contrastWhite.ratio }}:1 · {{ contrastWhite.aa ? 'AA' : '不达标' }}
          </span>
        </div>
        <div class="contrast-card" :style="{ background: hex, color: '#000000' }">
          <span class="c-label">黑字</span>
          <span class="c-text">Aa 文字示例</span>
          <span class="c-badge" :class="contrastBlack.aa ? 'pass' : 'fail'">
            {{ contrastBlack.ratio }}:1 · {{ contrastBlack.aa ? 'AA' : '不达标' }}
          </span>
        </div>
      </div>
      <p class="hint">
        推荐使用{{ contrastWhite.ratio >= contrastBlack.ratio ? '白字' : '黑字' }}。AA ≥ 4.5:1 · AAA
        ≥ 7:1
      </p>
    </div>

    <!-- 历史颜色 -->
    <div v-if="history.length" class="section">
      <div class="section-label">
        最近使用
        <button class="clear-history" @click="history = []">清除</button>
      </div>
      <div class="history-row">
        <button
          v-for="(c, i) in history"
          :key="i"
          class="history-swatch"
          :style="{ background: c }"
          :title="c"
          @click="setHex(c)"
        ></button>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Palette, Copy, Pipette, Shuffle } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

interface RGB {
  r: number;
  g: number;
  b: number;
}
interface HSL {
  h: number;
  s: number;
  l: number;
}
interface HSV {
  h: number;
  s: number;
  v: number;
}
interface CMYK {
  c: number;
  m: number;
  y: number;
  k: number;
}

const hex = useToolState('color', 'hex', '#2563eb');
const history = useToolState<string[]>('color', 'history', []);

const EyeDropperAPI = (
  window as unknown as { EyeDropper?: new () => { open: () => Promise<{ sRGBHex: string }> } }
).EyeDropper;
const hasEyeDropper = !!EyeDropperAPI;

function setHex(h: string) {
  let s = h.trim();
  if (!s.startsWith('#')) s = '#' + s;
  if (/^#[0-9a-fA-F]{6}$/.test(s)) {
    hex.value = s.toLowerCase();
    addToHistory(s);
  } else if (/^#[0-9a-fA-F]{3}$/.test(s)) {
    const e = s.slice(1);
    hex.value = '#' + e[0]! + e[0]! + e[1]! + e[1]! + e[2]! + e[2]!;
    addToHistory(hex.value);
  }
}

function addToHistory(h: string) {
  const upper = h.toUpperCase();
  const next = [upper, ...history.value.filter((c) => c !== upper)].slice(0, 14);
  history.value = next;
}

async function pickFromScreen() {
  if (!EyeDropperAPI) return;
  try {
    const ed = new EyeDropperAPI();
    const result = await ed.open();
    setHex(result.sRGBHex);
  } catch {
    /* cancelled */
  }
}

function randomHex(): string {
  const r = Math.floor(Math.random() * 256);
  const g = Math.floor(Math.random() * 256);
  const b = Math.floor(Math.random() * 256);
  return rgbToHex(r, g, b);
}

function hexToRgb(h: string): RGB {
  const m = h.replace('#', '').match(/.{2}/g) || [];
  return {
    r: parseInt(m[0] || '0', 16),
    g: parseInt(m[1] || '0', 16),
    b: parseInt(m[2] || '0', 16),
  };
}
function rgbToHex(r: number, g: number, b: number): string {
  const c = (n: number) =>
    Math.max(0, Math.min(255, Math.round(n)))
      .toString(16)
      .padStart(2, '0');
  return '#' + c(r) + c(g) + c(b);
}
function rgbToHsl(r: number, g: number, b: number): HSL {
  r /= 255;
  g /= 255;
  b /= 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0;
  let s = 0;
  const l = (max + min) / 2;
  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    if (max === r) h = (g - b) / d + (g < b ? 6 : 0);
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
    h /= 6;
  }
  return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) };
}
function hslToRgb(h: number, s: number, l: number): RGB {
  h /= 360;
  s /= 100;
  l /= 100;
  let r: number, g: number, b: number;
  if (s === 0) {
    r = g = b = l;
  } else {
    const hue2rgb = (p: number, q: number, t: number) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1 / 6) return p + (q - p) * 6 * t;
      if (t < 1 / 2) return q;
      if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
      return p;
    };
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1 / 3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1 / 3);
  }
  return { r: Math.round(r * 255), g: Math.round(g * 255), b: Math.round(b * 255) };
}
function rgbToHsv(r: number, g: number, b: number): HSV {
  r /= 255;
  g /= 255;
  b /= 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const d = max - min;
  let h = 0;
  if (d !== 0) {
    if (max === r) h = ((g - b) / d) % 6;
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
    h *= 60;
    if (h < 0) h += 360;
  }
  const s = max === 0 ? 0 : d / max;
  return { h: Math.round(h), s: Math.round(s * 100), v: Math.round(max * 100) };
}
function rgbToCmyk(r: number, g: number, b: number): CMYK {
  r /= 255;
  g /= 255;
  b /= 255;
  const k = 1 - Math.max(r, g, b);
  if (k >= 1) return { c: 0, m: 0, y: 0, k: 100 };
  return {
    c: Math.round(((1 - r - k) / (1 - k)) * 100),
    m: Math.round(((1 - g - k) / (1 - k)) * 100),
    y: Math.round(((1 - b - k) / (1 - k)) * 100),
    k: Math.round(k * 100),
  };
}

const rgb = computed(() => hexToRgb(hex.value));
const hsl = computed(() => rgbToHsl(rgb.value.r, rgb.value.g, rgb.value.b));
const hsv = computed(() => rgbToHsv(rgb.value.r, rgb.value.g, rgb.value.b));
const cmyk = computed(() => rgbToCmyk(rgb.value.r, rgb.value.g, rgb.value.b));

const formatList = computed(() => [
  { key: 'hex', label: 'HEX', value: hex.value.toUpperCase() },
  { key: 'rgb', label: 'RGB', value: `rgb(${rgb.value.r}, ${rgb.value.g}, ${rgb.value.b})` },
  { key: 'hsl', label: 'HSL', value: `hsl(${hsl.value.h}, ${hsl.value.s}%, ${hsl.value.l}%)` },
  { key: 'hsv', label: 'HSV', value: `hsv(${hsv.value.h}, ${hsv.value.s}%, ${hsv.value.v}%)` },
  {
    key: 'cmyk',
    label: 'CMYK',
    value: `cmyk(${cmyk.value.c}%, ${cmyk.value.m}%, ${cmyk.value.y}%, ${cmyk.value.k}%)`,
  },
]);

function setFromFormat(key: string, val: string) {
  val = val.trim();
  if (key === 'hex') {
    setHex(val);
  } else if (key === 'rgb') {
    const m = val.match(/(\d+)\D+(\d+)\D+(\d+)/);
    if (m) setHex(rgbToHex(+m[1]!, +m[2]!, +m[3]!));
  } else if (key === 'hsl') {
    const m = val.match(/(\d+)\D+(\d+)\D+(\d+)/);
    if (m) {
      const c = hslToRgb(+m[1]!, +m[2]!, +m[3]!);
      setHex(rgbToHex(c.r, c.g, c.b));
    }
  } else if (key === 'hsv') {
    const m = val.match(/(\d+)\D+(\d+)\D+(\d+)/);
    if (m) {
      const h = +m[1]!;
      const s = +m[2]! / 100;
      const v = +m[3]! / 100;
      const c = hsvToRgb(h, s, v);
      setHex(rgbToHex(c.r, c.g, c.b));
    }
  } else if (key === 'cmyk') {
    const m = val.match(/(\d+)\D+(\d+)\D+(\d+)\D+(\d+)/);
    if (m) {
      const c = cmykToRgb(+m[1]! / 100, +m[2]! / 100, +m[3]! / 100, +m[4]! / 100);
      setHex(rgbToHex(c.r, c.g, c.b));
    }
  }
}

function hsvToRgb(h: number, s: number, v: number): RGB {
  const c = v * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = v - c;
  let r = 0,
    g = 0,
    b = 0;
  if (h < 60) {
    r = c;
    g = x;
    b = 0;
  } else if (h < 120) {
    r = x;
    g = c;
    b = 0;
  } else if (h < 180) {
    r = 0;
    g = c;
    b = x;
  } else if (h < 240) {
    r = 0;
    g = x;
    b = c;
  } else if (h < 300) {
    r = x;
    g = 0;
    b = c;
  } else {
    r = c;
    g = 0;
    b = x;
  }
  return {
    r: Math.round((r + m) * 255),
    g: Math.round((g + m) * 255),
    b: Math.round((b + m) * 255),
  };
}
function cmykToRgb(c: number, m: number, y: number, k: number): RGB {
  return {
    r: Math.round(255 * (1 - c) * (1 - k)),
    g: Math.round(255 * (1 - m) * (1 - k)),
    b: Math.round(255 * (1 - y) * (1 - k)),
  };
}

function relLum(r: number, g: number, b: number) {
  const f = (c: number) => {
    c /= 255;
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  };
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}
function contrast(c1: string, c2: string) {
  const a = hexToRgb(c1);
  const b = hexToRgb(c2);
  const l1 = relLum(a.r, a.g, a.b);
  const l2 = relLum(b.r, b.g, b.b);
  const ratio = (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
  return { ratio: Math.round(ratio * 100) / 100, aa: ratio >= 4.5, aaa: ratio >= 7 };
}
const contrastWhite = computed(() => contrast(hex.value, '#ffffff'));
const contrastBlack = computed(() => contrast(hex.value, '#000000'));

const shades = computed(() => {
  const { h, s } = hsl.value;
  const points = [95, 85, 75, 65, 55, 45, 35, 25, 15];
  return points.map((l) => {
    const c = hslToRgb(h, s, l);
    return rgbToHex(c.r, c.g, c.b);
  });
});

const schemes = computed(() => {
  const { h, s } = hsl.value;
  const mk = (deg: number, sat = s, lit = 55) => {
    const c = hslToRgb((h + deg + 360) % 360, sat, lit);
    return rgbToHex(c.r, c.g, c.b);
  };
  return [
    { name: '互补色', colors: [mk(0), mk(180)] },
    { name: '类似色', colors: [mk(-30), mk(0), mk(30)] },
    { name: '三角色', colors: [mk(0), mk(120), mk(240)] },
    { name: '分裂互补', colors: [mk(0), mk(150), mk(210)] },
    { name: '矩形配色', colors: [mk(0), mk(60), mk(180), mk(240)] },
  ];
});

async function copy(v: string) {
  await copyToClipboard(v, '已复制 ' + v);
}
</script>

<style scoped>
.color-top {
  display: flex;
  gap: 20px;
  margin-bottom: 24px;
}
.swatch-area {
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex-shrink: 0;
}
.swatch {
  width: 120px;
  height: 120px;
  border-radius: var(--xuya-radius-lg);
  border: 1px solid var(--xuya-border);
  position: relative;
  overflow: hidden;
  box-shadow: var(--xuya-shadow);
  cursor: pointer;
}
.native-picker {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
}
.swatch-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 6px 10px;
  font-size: 12px;
  font-weight: 500;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  cursor: pointer;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
}
.action-btn:hover {
  border-color: var(--xuya-accent);
  color: var(--xuya-accent);
}
.action-btn.primary {
  background: var(--xuya-accent-gradient);
  color: var(--xuya-on-accent);
  border: none;
}
.action-btn.primary:hover {
  filter: brightness(1.08);
}

.formats {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  justify-content: center;
}
.fmt-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.fmt-label {
  width: 44px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  font-weight: 600;
  flex-shrink: 0;
}
.fmt-input {
  flex: 1;
  padding: 7px 12px;
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
.fmt-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.mini-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text-secondary);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
  flex-shrink: 0;
}
.mini-btn:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}

.section {
  margin-bottom: 24px;
}
.section-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
  margin-bottom: 10px;
}
.clear-history {
  font-size: 10px;
  color: var(--xuya-text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
  text-transform: none;
  letter-spacing: 0;
}
.clear-history:hover {
  color: var(--xuya-danger);
}

.shade-row {
  display: flex;
  gap: 6px;
}
.swatch-btn {
  flex: 1;
  height: 48px;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  cursor: pointer;
  transition: transform var(--xuya-duration-fast) var(--xuya-ease);
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding-bottom: 3px;
  overflow: hidden;
}
.swatch-btn.sm {
  height: 40px;
}
.swatch-btn:hover {
  transform: translateY(-3px);
  box-shadow: var(--xuya-shadow-hover);
}
.swatch-label {
  font-size: 8px;
  font-family: var(--xuya-font-mono);
  color: rgba(255, 255, 255, 0.85);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
}

.scheme-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.scheme-row {
  display: flex;
  align-items: center;
  gap: 12px;
}
.scheme-name {
  width: 72px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  flex-shrink: 0;
}
.scheme-swatches {
  display: flex;
  gap: 6px;
  flex: 1;
}

.contrast-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
.contrast-card {
  padding: 18px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: flex-start;
}
.c-label {
  font-size: 12px;
  opacity: 0.8;
}
.c-text {
  font-size: 18px;
  font-weight: 600;
}
.c-badge {
  display: inline-block;
  padding: 3px 10px;
  border-radius: 99px;
  font-size: 11px;
  font-weight: 700;
  background: rgba(0, 0, 0, 0.2);
  backdrop-filter: blur(4px);
}
.c-badge.pass {
  background: rgba(0, 0, 0, 0.25);
}
.c-badge.fail {
  background: rgba(255, 0, 0, 0.3);
}
.hint {
  margin-top: 10px;
  font-size: 12px;
  color: var(--xuya-text-tertiary);
}

.history-row {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.history-swatch {
  width: 32px;
  height: 32px;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  cursor: pointer;
  transition: transform var(--xuya-duration-fast) var(--xuya-ease);
}
.history-swatch:hover {
  transform: scale(1.15);
  box-shadow: var(--xuya-shadow-hover);
}
</style>
