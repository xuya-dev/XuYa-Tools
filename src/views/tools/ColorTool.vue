<template>
  <ToolShell title="颜色工具" :icon="Palette" description="HEX / RGB / HSL 互转、色板生成、WCAG 对比度检测。">
    <!-- 预览 + 选色 -->
    <div class="color-top">
      <div class="swatch" :style="{ background: hex }">
        <input type="color" :value="hex" @input="setHex(($event.target as HTMLInputElement).value)" class="picker" />
      </div>
      <div class="formats">
        <div class="fmt-row" v-for="f in FMTS" :key="f.key">
          <span class="fmt-label">{{ f.label }}</span>
          <input class="fmt-input mono" :value="f.value" spellcheck="false" @change="setFrom(f.key, ($event.target as HTMLInputElement).value)" />
          <button class="mini-btn" @click="copy(f.value)"><Copy :size="13" /></button>
        </div>
      </div>
    </div>

    <!-- 对比度检测 -->
    <div class="contrast-section">
      <div class="section-label">对比度检测 (WCAG)</div>
      <div class="contrast-grid">
        <div class="contrast-card fg" :style="{ background: hex, color: '#fff' }">
          <div>前景 {{ hex }}</div>
          <div class="ctext" style="font-size:18px">Aa 文字</div>
          <div class="cbadge" :class="contrastWhite.pass ? 'pass' : 'fail'">{{ contrastWhite.ratio }} : 1</div>
        </div>
        <div class="contrast-card fg" :style="{ background: hex, color: '#000' }">
          <div>前景 {{ hex }} (黑字)</div>
          <div class="ctext" style="font-size:18px">Aa 文字</div>
          <div class="cbadge" :class="contrastBlack.pass ? 'pass' : 'fail'">{{ contrastBlack.ratio }} : 1</div>
        </div>
      </div>
      <p class="hint">建议使用 {{ contrastWhite.ratio >= contrastBlack.ratio ? '白字' : '黑字' }}(对比度更高)。AA 标准需 ≥ 4.5:1,AAA 需 ≥ 7:1。</p>
    </div>

    <!-- 调色方案 -->
    <div class="scheme-section" v-if="schemes.length">
      <div class="section-label">配色方案</div>
      <div class="scheme-list">
        <div v-for="(sc, si) in schemes" :key="si" class="scheme-row">
          <span class="scheme-name">{{ sc.name }}</span>
          <div class="swatches">
            <button
              v-for="(c, ci) in sc.colors"
              :key="ci"
              class="swatch-btn"
              :style="{ background: c }"
              :title="c"
              @click="setHex(c)"
            >{{ c.toUpperCase() }}</button>
          </div>
        </div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { Palette, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';

const hex = ref('#3b82f6');

function setHex(h: string) {
  if (/^#?[0-9a-fA-F]{6}$/.test(h)) {
    hex.value = h.startsWith('#') ? h : '#' + h;
  } else if (/^#?[0-9a-fA-F]{3}$/.test(h)) {
    const s = h.replace('#', '');
    hex.value = '#' + s.split('').map((c) => c + c).join('');
  }
}

function hexToRgb(h: string) {
  const m = h.replace('#', '').match(/.{2}/g) || [];
  return { r: parseInt(m[0] || '0', 16), g: parseInt(m[1] || '0', 16), b: parseInt(m[2] || '0', 16) };
}
function rgbToHex(r: number, g: number, b: number) {
  const c = (n: number) => Math.max(0, Math.min(255, Math.round(n))).toString(16).padStart(2, '0');
  return '#' + c(r) + c(g) + c(b);
}
function rgbToHsl(r: number, g: number, b: number) {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  let h = 0, s = 0; const l = (max + min) / 2;
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
function hslToRgb(h: number, s: number, l: number) {
  h /= 360; s /= 100; l /= 100;
  let r, g, b;
  if (s === 0) { r = g = b = l; }
  else {
    const hue2rgb = (p: number, q: number, t: number) => {
      if (t < 0) t += 1; if (t > 1) t -= 1;
      if (t < 1 / 6) return p + (q - p) * 6 * t;
      if (t < 1 / 2) return q;
      if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
      return p;
    };
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1 / 3); g = hue2rgb(p, q, h); b = hue2rgb(p, q, h - 1 / 3);
  }
  return { r: Math.round(r * 255), g: Math.round(g * 255), b: Math.round(b * 255) };
}

const rgb = computed(() => hexToRgb(hex.value));
const hsl = computed(() => rgbToHsl(rgb.value.r, rgb.value.g, rgb.value.b));

const FMTS = computed(() => [
  { key: 'hex', label: 'HEX', value: hex.value.toUpperCase() },
  { key: 'rgb', label: 'RGB', value: `rgb(${rgb.value.r}, ${rgb.value.g}, ${rgb.value.b})` },
  { key: 'hsl', label: 'HSL', value: `hsl(${hsl.value.h}, ${hsl.value.s}%, ${hsl.value.l}%)` },
]);

function setFrom(key: string, val: string) {
  val = val.trim();
  if (key === 'hex') setHex(val);
  else if (key === 'rgb') {
    const m = val.match(/(\d+)\D+(\d+)\D+(\d+)/);
    if (m) setHex(rgbToHex(+m[1]!, +m[2]!, +m[3]!));
  } else if (key === 'hsl') {
    const m = val.match(/(\d+)\D+(\d+)\D+(\d+)/);
    if (m) {
      const c = hslToRgb(+m[1]!, +m[2]!, +m[3]!);
      setHex(rgbToHex(c.r, c.g, c.b));
    }
  }
}

// ===== 对比度 =====
function relLum(r: number, g: number, b: number) {
  const f = (c: number) => { c /= 255; return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4); };
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}
function contrast(c1: string, c2: string) {
  const a = hexToRgb(c1), b = hexToRgb(c2);
  const l1 = relLum(a.r, a.g, a.b), l2 = relLum(b.r, b.g, b.b);
  const ratio = (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
  return { ratio: Math.round(ratio * 100) / 100, pass: ratio >= 4.5 };
}
const contrastWhite = computed(() => contrast(hex.value, '#ffffff'));
const contrastBlack = computed(() => contrast(hex.value, '#000000'));

// ===== 配色方案 =====
const schemes = computed(() => {
  const { h } = hsl.value;
  const mk = (deg: number) => {
    const c = hslToRgb((h + deg + 360) % 360, 70, 55);
    return rgbToHex(c.r, c.g, c.b);
  };
  return [
    { name: '互补色', colors: [mk(0), mk(180)] },
    { name: '类似色', colors: [mk(-30), mk(0), mk(30)] },
    { name: '三角色', colors: [mk(0), mk(120), mk(240)] },
    { name: '明度阶', colors: [0, 1, 2, 3, 4].map((i) => { const c = hslToRgb(h, 60, 20 + i * 16); return rgbToHex(c.r, c.g, c.b); }) },
  ];
});

async function copy(v: string) { await copyToClipboard(v, '已复制 ' + v); }
</script>

<style scoped>
.color-top { display: flex; gap: 16px; margin-bottom: 24px; }
.swatch { width: 120px; height: 120px; border-radius: var(--xuya-radius-lg); border: 1px solid var(--xuya-border); position: relative; overflow: hidden; flex-shrink: 0; }
.picker { position: absolute; inset: 0; opacity: 0; cursor: pointer; }
.formats { flex: 1; display: flex; flex-direction: column; gap: 8px; justify-content: center; }
.fmt-row { display: flex; align-items: center; gap: 8px; }
.fmt-label { width: 40px; font-size: 12px; color: var(--xuya-text-secondary); font-weight: 600; }
.fmt-input { flex: 1; padding: 7px 12px; font-size: 13px; border-radius: var(--xuya-radius-sm); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); transition: border-color .12s; }
.fmt-input:focus { outline: none; border-color: var(--xuya-accent); }
.mini-btn { width: 30px; height: 30px; display: flex; align-items: center; justify-content: center; border-radius: var(--xuya-radius-sm); color: var(--xuya-text-secondary); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); transition: .1s; }
.mini-btn:hover { color: var(--xuya-text); border-color: var(--xuya-accent); }

.contrast-section, .scheme-section { margin-bottom: 24px; }
.section-label { font-size: 11px; text-transform: uppercase; letter-spacing: .5px; color: var(--xuya-text-tertiary); margin-bottom: 10px; }
.contrast-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.contrast-card { padding: 18px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); font-size: 12px; }
.ctext { margin: 12px 0; font-weight: 600; }
.cbadge { display: inline-block; padding: 3px 10px; border-radius: 99px; font-size: 12px; font-weight: 700; }
.cbadge.pass { background: rgba(0,0,0,.25); color: #fff; }
.cbadge.fail { background: rgba(0,0,0,.25); color: #fff; }
.hint { margin-top: 10px; font-size: 12px; color: var(--xuya-text-tertiary); }

.scheme-list { display: flex; flex-direction: column; gap: 10px; }
.scheme-row { display: flex; align-items: center; gap: 12px; }
.scheme-name { width: 60px; font-size: 12px; color: var(--xuya-text-secondary); flex-shrink: 0; }
.swatches { display: flex; gap: 6px; flex: 1; }
.swatch-btn { flex: 1; height: 44px; border-radius: var(--xuya-radius-sm); border: 1px solid var(--xuya-border); font-size: 9px; color: rgba(255,255,255,.9); text-shadow: 0 1px 2px rgba(0,0,0,.5); transition: transform .1s; font-family: var(--xuya-font-mono); }
.swatch-btn:hover { transform: translateY(-2px); }
</style>
