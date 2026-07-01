<template>
  <div class="tool-bar" @mousedown="startDrag">
    <template v-if="showBack">
      <button class="tb-back-btn" title="返回首页" @click="emitBack" @mousedown.stop>
        <ChevronDown :size="18" style="transform: rotate(90deg)" />
      </button>
    </template>
    <template v-else-if="showBrand">
      <img class="tb-brand-logo" src="/Logo.png" alt="" />
      <span class="tb-brand-name">XuYa Tools</span>
    </template>
    <div class="tb-spacer"></div>

    <span v-if="categoryName" class="tb-cat-badge">
      <span class="tb-cat-dot" :style="{ background: categoryColor }"></span>
      {{ categoryName }}
    </span>

    <div class="tb-controls" @mousedown.stop>
      <button class="tb-btn theme-btn" title="桌面悬浮窗" @click="onToggleWidget">
        <LayoutPanelLeft :size="15" />
      </button>
      <button class="tb-btn theme-btn" title="布局与主题" @click="emitSwitcher">
        <SlidersHorizontal :size="15" />
      </button>
      <button class="tb-btn theme-btn" :title="themeLabel" @click="cycleMode">
        <Sun v-if="mode === 'light'" :size="15" />
        <Moon v-else-if="mode === 'dark'" :size="15" />
        <Monitor v-else :size="15" />
      </button>
      <span class="tb-ctrl-sep"></span>
      <button class="tb-btn" title="最小化" @click="minimize">
        <svg width="11" height="11" viewBox="0 0 12 12">
          <rect x="1.5" y="5.5" width="9" height="1" fill="currentColor" />
        </svg>
      </button>
      <button class="tb-btn" :title="isMaximized ? '向下还原' : '最大化'" @click="toggleMaximize">
        <svg v-if="!isMaximized" width="11" height="11" viewBox="0 0 12 12">
          <rect
            x="2.2"
            y="2.2"
            width="7.6"
            height="7.6"
            rx="1"
            stroke="currentColor"
            stroke-width="1"
            fill="none"
          />
        </svg>
        <svg v-else width="11" height="11" viewBox="0 0 12 12">
          <rect
            x="1.5"
            y="4"
            width="6.5"
            height="6.5"
            rx="1"
            stroke="currentColor"
            stroke-width="1"
            fill="var(--xuya-bg-elevated)"
          />
          <path d="M4 4V2.5h6.5V9H9" stroke="currentColor" stroke-width="1" fill="none" />
        </svg>
      </button>
      <button class="tb-btn close" title="关闭" @click="close">
        <svg width="11" height="11" viewBox="0 0 12 12">
          <path
            d="M2.2 2.2L9.8 9.8M9.8 2.2L2.2 9.8"
            stroke="currentColor"
            stroke-width="1.15"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { Sun, Moon, Monitor, SlidersHorizontal, ChevronDown, LayoutPanelLeft } from '@lucide/vue';
import { tools, CATEGORY_LABELS, CATEGORY_COLORS } from '@/config/tools';
import { useWindow } from '@/composables/useWindow';
import { useTheme } from '@/composables/useTheme';
import { useWidget } from '@/composables/useWidget';

const route = useRoute();
const { isMaximized, minimize, toggleMaximize, close, startDrag } = useWindow();
const { mode, cycleMode } = useTheme();
const { toggleWidget } = useWidget();

function onToggleWidget() {
  toggleWidget();
}

const categoryName = computed(() => {
  const cat = tools.find((t) => route.path === t.route)?.category;
  return cat ? CATEGORY_LABELS[cat] : '';
});

const categoryColor = computed(() => {
  const cat = tools.find((t) => route.path === t.route)?.category;
  return cat ? CATEGORY_COLORS[cat] : 'var(--xuya-accent)';
});

const props = withDefaults(
  defineProps<{
    showBrand?: boolean;
    showBack?: boolean;
  }>(),
  { showBrand: false, showBack: false },
);
void props;

const emit = defineEmits<{ switcher: []; back: [] }>();
function emitSwitcher() {
  emit('switcher');
}
function emitBack() {
  emit('back');
}

const themeLabel = computed(
  () =>
    ({
      light: '浅色 (点击切换)',
      dark: '深色 (点击切换)',
      system: '跟随系统 (点击切换)',
    })[mode.value],
);
</script>

<style scoped>
.tool-bar {
  height: 44px;
  min-height: 44px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 0 0 20px;
  background: var(--xuya-bg-elevated);
  border-bottom: 1px solid var(--xuya-border);
  -webkit-app-region: drag;
  flex-shrink: 0;
  user-select: none;
}

.tb-back-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--xuya-text-secondary);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
  -webkit-app-region: no-drag;
  flex-shrink: 0;
}
.tb-back-btn:hover {
  background: var(--xuya-input-bg);
  color: var(--xuya-accent);
}

.tb-brand-logo {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  object-fit: cover;
  flex-shrink: 0;
  box-shadow: 0 2px 8px var(--xuya-accent-glow);
  -webkit-app-region: no-drag;
}
.tb-brand-name {
  font-size: 13px;
  font-weight: 700;
  color: var(--xuya-text);
  letter-spacing: -0.02em;
  white-space: nowrap;
  flex-shrink: 0;
}

.tb-spacer {
  flex: 1;
}

.tb-cat-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 2px 9px;
  border-radius: var(--xuya-radius-sm);
  font-size: 11px;
  font-weight: 600;
  font-family: var(--xuya-font-mono);
  letter-spacing: 0.03em;
  background: var(--xuya-input-bg);
  color: var(--xuya-text-secondary);
  border: 1px solid var(--xuya-border);
  -webkit-app-region: no-drag;
}
.tb-cat-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

.tb-controls {
  display: flex;
  align-items: center;
  gap: 0;
  height: 100%;
  -webkit-app-region: no-drag;
}

.tb-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--xuya-text-secondary);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition:
    background var(--xuya-duration-fast) var(--xuya-ease),
    color var(--xuya-duration-fast) var(--xuya-ease);
}
.tb-btn:hover {
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
}
.tb-btn:active {
  transform: scale(0.92);
}

.theme-btn {
  width: 32px;
  margin-right: 2px;
}

.tb-ctrl-sep {
  width: 1px;
  height: 18px;
  background: var(--xuya-border);
  margin: 0 6px;
  flex-shrink: 0;
}

.tb-btn.close:hover {
  background: #e81123;
  color: #fff;
}
.tb-btn.close:active {
  background: #c50f1f;
}
</style>
