<template>
  <div class="titlebar" @mousedown="startDrag">
    <!-- 左侧:拖拽区 + 品牌 -->
    <div class="titlebar-left">
      <div class="brand">
        <img class="brand-logo" src="/Logo.png" alt="" aria-hidden="true" />
        <span class="brand-name">XuYa Tools</span>
      </div>
    </div>

    <!-- 中间:可拖拽留白 -->
    <div class="titlebar-center"></div>

    <!-- 右侧:主题切换 + 窗口控制 -->
    <div class="titlebar-right">
      <button
        class="ctrl-btn theme-btn"
        :title="themeLabel"
        @mousedown.stop
        @click="cycleMode"
      >
        <Sun v-if="mode === 'light'" :size="14" />
        <Moon v-else-if="mode === 'dark'" :size="14" />
        <Monitor v-else :size="14" />
      </button>

      <div class="win-controls" @mousedown.stop>
        <button class="ctrl-btn" title="最小化" @click="minimize">
          <svg width="11" height="11" viewBox="0 0 12 12"><rect x="1.5" y="5.5" width="9" height="1" fill="currentColor"/></svg>
        </button>
        <button class="ctrl-btn" :title="isMaximized ? '还原' : '最大化'" @click="toggleMaximize">
          <svg v-if="!isMaximized" width="11" height="11" viewBox="0 0 12 12"><rect x="2" y="2" width="8" height="8" stroke="currentColor" stroke-width="1" fill="none"/></svg>
          <svg v-else width="11" height="11" viewBox="0 0 12 12">
            <rect x="3.5" y="1.5" width="6.5" height="6.5" stroke="currentColor" stroke-width="1" fill="none"/>
            <rect x="1.5" y="3.5" width="6.5" height="6.5" stroke="currentColor" stroke-width="1" fill="var(--xuya-bg-elevated)"/>
          </svg>
        </button>
        <button class="ctrl-btn close" title="关闭" @click="close">
          <svg width="11" height="11" viewBox="0 0 12 12">
            <path d="M2 2 L10 10 M10 2 L2 10" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Sun, Moon, Monitor } from '@lucide/vue';
import { useWindow } from '@/composables/useWindow';
import { useTheme } from '@/composables/useTheme';

const { isMaximized, minimize, toggleMaximize, close, startDrag } = useWindow();
const { mode, cycleMode } = useTheme();

const themeLabel = computed(
  () => ({ light: '浅色 (点击切换)', dark: '深色 (点击切换)', system: '跟随系统 (点击切换)' }[mode.value])
);
</script>

<style scoped>
.titlebar {
  height: 42px;
  display: flex;
  align-items: center;
  padding: 0 8px 0 14px;
  background: var(--xuya-bg-elevated);
  border-bottom: 1px solid var(--xuya-border);
  -webkit-app-region: drag;
  flex-shrink: 0;
  /* 标题栏底部微弱分隔 (替代生硬边框) */
  box-shadow: var(--xuya-shadow-sm);
}

.titlebar-left,
.titlebar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.titlebar-center {
  flex: 1;
  height: 100%;
  -webkit-app-region: drag;
}

.brand {
  display: flex;
  align-items: center;
  gap: 9px;
  -webkit-app-region: no-drag;
}

.brand-logo {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  object-fit: cover;
  box-shadow: var(--xuya-shadow-hover);
}

.brand-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
  letter-spacing: 0.2px;
}

.titlebar-right {
  -webkit-app-region: no-drag;
}

.win-controls {
  display: flex;
  align-items: center;
  gap: 2px;
}

.ctrl-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: none;
  background: transparent;
  color: var(--xuya-text-secondary);
  border-radius: var(--xuya-radius-sm);
  transition: background var(--xuya-duration-fast) var(--xuya-ease),
    color var(--xuya-duration-fast) var(--xuya-ease);
}

.ctrl-btn:hover {
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
}

.ctrl-btn:active {
  transform: scale(0.92);
}

.theme-btn {
  margin-right: 4px;
}

.ctrl-btn.close:hover {
  background: var(--xuya-danger);
  color: var(--xuya-bg-elevated);
}
</style>
