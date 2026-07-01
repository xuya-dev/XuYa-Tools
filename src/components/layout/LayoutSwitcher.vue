<template>
  <Transition name="switcher-fade">
    <div v-if="visible" class="switcher-overlay" @click.self="close">
      <div class="switcher-panel">
        <div class="switcher-head">
          <span class="switcher-title">布局与主题</span>
          <button class="switcher-close" @click="close"><X :size="16" /></button>
        </div>

        <!-- 布局选择 -->
        <div class="switcher-section">
          <span class="section-label">菜单布局</span>
          <div class="layout-options">
            <button
              v-for="l in LAYOUT_OPTIONS"
              :key="l.id"
              class="layout-option"
              :class="{ active: layoutMode === l.id }"
              @click="setLayout(l.id)"
            >
              <div class="layout-thumb" :class="l.id">
                <div v-if="l.id === 'sidebar'" class="thumb-sidebar">
                  <div class="ts-rail"></div>
                  <div class="ts-body"></div>
                </div>
                <div v-if="l.id === 'tabbar'" class="thumb-tabbar">
                  <div class="tt-top"></div>
                  <div class="tt-side"></div>
                  <div class="tt-body"></div>
                </div>
                <div v-if="l.id === 'grid'" class="thumb-grid">
                  <div class="tg-cell"></div>
                  <div class="tg-cell"></div>
                  <div class="tg-cell"></div>
                  <div class="tg-cell"></div>
                  <div class="tg-cell"></div>
                  <div class="tg-cell"></div>
                </div>
                <div v-if="l.id === 'dual'" class="thumb-dual">
                  <div class="td-cat"></div>
                  <div class="td-tools"></div>
                  <div class="td-body"></div>
                </div>
              </div>
              <span class="layout-name">{{ l.label }}</span>
            </button>
          </div>
        </div>

        <!-- 主题色 -->
        <div class="switcher-section">
          <span class="section-label">主题强调色</span>
          <div class="accent-options">
            <button
              v-for="a in accents"
              :key="a.id"
              class="accent-option"
              :class="{ active: accentId === a.id }"
              :style="{ background: a.gradient }"
              :title="a.label"
              @click="setAccent(a.id)"
            >
              <Check v-if="accentId === a.id" :size="16" />
            </button>
          </div>
        </div>

        <!-- 明暗模式 -->
        <div class="switcher-section">
          <span class="section-label">明暗模式</span>
          <div class="theme-modes">
            <button
              v-for="t in THEME_MODES"
              :key="t.id"
              class="theme-mode-btn"
              :class="{ active: currentTheme === t.id }"
              @click="setThemeMode(t.id)"
            >
              <component :is="t.icon" :size="15" />
              {{ t.label }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { Sun, Moon, Monitor, Check, X } from '@lucide/vue';
import { useLayout, type LayoutMode } from '@/composables/useLayout';
import { useTheme, type ThemeMode } from '@/composables/useTheme';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();

const { layoutMode, accentId, accents, setLayout, setAccent } = useLayout();
const { mode: currentTheme, setMode: setThemeMode } = useTheme();

function close() {
  emit('close');
}

const LAYOUT_OPTIONS: { id: LayoutMode; label: string }[] = [
  { id: 'sidebar', label: '侧栏分组' },
  { id: 'dual', label: '双栏导航' },
  { id: 'tabbar', label: '顶部 Tab' },
  { id: 'grid', label: '网格首页' },
];

const THEME_MODES: { id: ThemeMode; label: string; icon: typeof Sun }[] = [
  { id: 'light', label: '浅色', icon: Sun },
  { id: 'dark', label: '深色', icon: Moon },
  { id: 'system', label: '跟随', icon: Monitor },
];

void props;
</script>

<style scoped>
.switcher-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: var(--xuya-overlay);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
  padding: 52px 14px 0 0;
  -webkit-app-region: no-drag;
}

.switcher-panel {
  width: 340px;
  background: var(--xuya-bg-elevated);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-xl);
  box-shadow: var(--xuya-shadow-xl);
  overflow: hidden;
}

.switcher-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--xuya-border);
}
.switcher-title {
  font-size: 14px;
  font-weight: 700;
}
.switcher-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  background: transparent;
  color: var(--xuya-text-tertiary);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.switcher-close:hover {
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
}

.switcher-section {
  padding: 14px 18px;
  border-bottom: 1px solid var(--xuya-border-light);
}
.switcher-section:last-child {
  border-bottom: none;
}
.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--xuya-text-tertiary);
  display: block;
  margin-bottom: 10px;
}

/* 布局选择 */
.layout-options {
  display: flex;
  gap: 10px;
}
.layout-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 8px;
  border: 2px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  background: var(--xuya-input-bg);
  cursor: pointer;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  flex: 1;
}
.layout-option:hover {
  border-color: var(--xuya-border-strong);
}
.layout-option.active {
  border-color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
}
.layout-thumb {
  width: 100%;
  height: 52px;
  border-radius: 4px;
  overflow: hidden;
  position: relative;
  background: var(--xuya-bg);
}
.layout-name {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text);
}
.layout-option.active .layout-name {
  color: var(--xuya-accent);
}

/* 缩略图: sidebar */
.thumb-sidebar {
  display: flex;
  height: 100%;
}
.thumb-sidebar .ts-rail {
  width: 28%;
  height: 100%;
  background: var(--xuya-border);
}
.thumb-sidebar .ts-body {
  flex: 1;
}

/* 缩略图: tabbar */
.thumb-tabbar {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.thumb-tabbar .tt-top {
  height: 28%;
  background: var(--xuya-border);
}
.thumb-tabbar .tt-side {
  width: 22%;
  background: var(--xuya-border-light);
  position: absolute;
  bottom: 0;
  left: 0;
  height: 72%;
}
.thumb-tabbar .tt-body {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 78%;
  height: 72%;
}

/* 缩略图: grid */
.thumb-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 3px;
  padding: 4px;
}
.thumb-grid .tg-cell {
  background: var(--xuya-border);
  border-radius: 3px;
  aspect-ratio: 1;
}

/* 缩略图: dual */
.thumb-dual {
  display: flex;
  height: 100%;
}
.thumb-dual .td-cat {
  width: 26%;
  background: var(--xuya-border);
}
.thumb-dual .td-tools {
  width: 26%;
  background: var(--xuya-border-light);
}
.thumb-dual .td-body {
  flex: 1;
}

.layout-options {
  flex-wrap: wrap;
}

/* 主题色 */
.accent-options {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.accent-option {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}
.accent-option:hover {
  transform: scale(1.1);
}
.accent-option.active {
  border-color: var(--xuya-text);
  box-shadow:
    0 0 0 2px var(--xuya-bg-elevated),
    0 0 0 4px currentColor;
}

/* 明暗 */
.theme-modes {
  display: flex;
  gap: 8px;
}
.theme-mode-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 6px 14px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.theme-mode-btn:hover {
  border-color: var(--xuya-border-strong);
}
.theme-mode-btn.active {
  background: var(--xuya-accent-soft);
  border-color: var(--xuya-accent);
  color: var(--xuya-accent);
  font-weight: 600;
}

/* 动画 */
.switcher-fade-enter-active,
.switcher-fade-leave-active {
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
}
.switcher-fade-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}
.switcher-fade-leave-to {
  opacity: 0;
}
</style>
