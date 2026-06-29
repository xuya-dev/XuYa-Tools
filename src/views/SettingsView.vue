<template>
  <ToolShell title="设置" :icon="Settings" description="应用偏好、主题、自启动与数据管理。">
    <!-- 外观 -->
    <div class="settings-section">
      <div class="settings-section-title">外观</div>
      <div class="settings-row">
        <div class="settings-info">
          <div class="settings-name">主题模式</div>
          <div class="settings-desc">浅色 / 深色 / 跟随系统</div>
        </div>
        <div class="seg">
          <button :class="{ active: mode === 'light' }" @click="setMode('light')">
            <Sun :size="14" /> 浅色
          </button>
          <button :class="{ active: mode === 'dark' }" @click="setMode('dark')">
            <Moon :size="14" /> 深色
          </button>
          <button :class="{ active: mode === 'system' }" @click="setMode('system')">
            <Monitor :size="14" /> 系统
          </button>
        </div>
      </div>
    </div>

    <!-- 系统 -->
    <div class="settings-section">
      <div class="settings-section-title">系统</div>
      <div class="settings-row">
        <div class="settings-info">
          <div class="settings-name">开机自启动</div>
          <div class="settings-desc">系统登录时自动启动 XuYa Tools</div>
        </div>
        <label class="switch">
          <input type="checkbox" :checked="autostartEnabled" @change="toggleAutostart" />
          <span class="slider"></span>
        </label>
      </div>
      <div class="settings-row">
        <div class="settings-info">
          <div class="settings-name">数据目录</div>
          <div class="settings-desc mono small">{{ dataDir || '加载中…' }}</div>
        </div>
        <BaseButton variant="ghost" :disabled="!dataDir" @click="openDataDir">
          <FolderOpen :size="14" /> 打开
        </BaseButton>
      </div>
    </div>

    <!-- 数据 -->
    <div class="settings-section">
      <div class="settings-section-title">数据管理</div>
      <div class="settings-row">
        <div class="settings-info">
          <div class="settings-name">清除工具输入缓存</div>
          <div class="settings-desc">清除所有工具的输入记忆 (不影响配置与收藏)</div>
        </div>
        <BaseButton variant="danger" @click="clearCache">清除</BaseButton>
      </div>
      <div class="settings-row">
        <div class="settings-info">
          <div class="settings-name">重置收藏列表</div>
          <div class="settings-desc">恢复收藏为默认 (JSON / 正则 / 时间戳)</div>
        </div>
        <BaseButton variant="ghost" @click="resetFavorites">重置</BaseButton>
      </div>
    </div>

    <!-- 关于 -->
    <div class="settings-section">
      <div class="settings-section-title">关于</div>
      <div class="about-card">
        <img class="about-logo" src="/Logo.png" alt="" aria-hidden="true" />
        <div class="about-info">
          <div class="about-name">XuYa Tools</div>
          <div class="about-version">v0.1.0 · 程序员日常开发工具箱</div>
          <div class="about-tech">Vite 8 · Vue 3 · Tauri 2</div>
        </div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Settings, Sun, Moon, Monitor, FolderOpen } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useTheme } from '@/composables/useTheme';
import { useToast } from '@/composables/useToast';
import { clearAllToolState } from '@/composables/useToolState';
import { useFavorites } from '@/composables/useFavorites';

const toast = useToast();
const { mode, setMode } = useTheme();
const { favorites } = useFavorites();

const autostartEnabled = ref(false);
const dataDir = ref('');

async function toggleAutostart() {
  try {
    const { isEnabled, enable, disable } = await import('@tauri-apps/plugin-autostart');
    const enabled = await isEnabled();
    if (enabled) {
      await disable();
      autostartEnabled.value = false;
      toast.success('已关闭开机自启');
    } else {
      await enable();
      autostartEnabled.value = true;
      toast.success('已开启开机自启');
    }
  } catch (e) {
    toast.error('操作失败: ' + e);
  }
}

async function openDataDir() {
  try {
    const opener = await import('@tauri-apps/plugin-opener');
    await opener.openPath(dataDir.value);
  } catch (e) {
    toast.error('打开失败: ' + e);
  }
}

function clearCache() {
  if (!confirm('确定清除所有工具的输入缓存?此操作不可撤销。')) return;
  clearAllToolState();
  toast.success('已清除工具输入缓存');
}

function resetFavorites() {
  favorites.value = ['json', 'regex', 'timestamp'];
  toast.success('已重置收藏列表');
}

onMounted(async () => {
  try {
    const { isEnabled } = await import('@tauri-apps/plugin-autostart');
    autostartEnabled.value = await isEnabled();
  } catch { /* 非 Tauri 环境 */ }
  try {
    const dataDirPath = (await import('@tauri-apps/api/path')).appDataDir;
    dataDir.value = await dataDirPath();
  } catch { /* 非 Tauri 环境 */ }
});
</script>

<style scoped>
.settings-section { margin-bottom: 28px; }
.settings-section-title { font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.7px; color: var(--xuya-text-tertiary); margin-bottom: 10px; padding-left: 4px; }
.settings-row { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 14px 16px; background: var(--xuya-card-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius); margin-bottom: 8px; }
.settings-info { flex: 1; min-width: 0; }
.settings-name { font-size: 14px; font-weight: 600; color: var(--xuya-text); }
.settings-desc { font-size: 12px; color: var(--xuya-text-tertiary); margin-top: 3px; line-height: 1.4; word-break: break-all; }
.small { font-size: 11.5px; }

.seg { display: inline-flex; background: var(--xuya-input-bg); border-radius: var(--xuya-radius); padding: 3px; gap: 2px; }
.seg button { display: inline-flex; align-items: center; gap: 5px; padding: 6px 14px; font-size: 12px; color: var(--xuya-text-secondary); background: transparent; border: none; border-radius: var(--xuya-radius-sm); transition: .1s; }
.seg button.active { background: var(--xuya-bg-elevated); color: var(--xuya-accent); font-weight: 600; box-shadow: var(--xuya-shadow-sm); }

.switch { position: relative; display: inline-block; width: 44px; height: 24px; flex-shrink: 0; }
.switch input { opacity: 0; width: 0; height: 0; }
.slider { position: absolute; cursor: pointer; inset: 0; background: var(--xuya-border-strong); border-radius: 24px; transition: var(--xuya-duration) var(--xuya-ease); }
.slider::before { content: ''; position: absolute; height: 18px; width: 18px; left: 3px; top: 3px; background: #fff; border-radius: 50%; transition: var(--xuya-duration) var(--xuya-ease); box-shadow: var(--xuya-shadow-sm); }
.switch input:checked + .slider { background: var(--xuya-accent); }
.switch input:checked + .slider::before { transform: translateX(20px); }

.about-card { display: flex; align-items: center; gap: 14px; padding: 20px; background: var(--xuya-card-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius); }
.about-logo { width: 48px; height: 48px; border-radius: var(--xuya-radius); object-fit: cover; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.18); flex-shrink: 0; }
.about-name { font-size: 16px; font-weight: 700; color: var(--xuya-text); }
.about-version { font-size: 12.5px; color: var(--xuya-text-secondary); margin-top: 3px; }
.about-tech { font-size: 11px; color: var(--xuya-text-tertiary); margin-top: 6px; font-family: var(--xuya-font-mono); }
</style>
