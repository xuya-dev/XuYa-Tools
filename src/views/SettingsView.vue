<template>
  <ToolShell title="设置" :icon="Settings" description="应用偏好、主题、自启动、快捷键与数据管理。">
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
            <Sun :size="14" />
            浅色
          </button>
          <button :class="{ active: mode === 'dark' }" @click="setMode('dark')">
            <Moon :size="14" />
            深色
          </button>
          <button :class="{ active: mode === 'system' }" @click="setMode('system')">
            <Monitor :size="14" />
            系统
          </button>
        </div>
      </div>
      <div class="settings-row">
        <div class="settings-info">
          <div class="settings-name">主题强调色</div>
          <div class="settings-desc">
            点击标题栏
            <SlidersHorizontal :size="11" />
            按钮可切换全部外观
          </div>
        </div>
        <div class="accent-row">
          <button
            v-for="a in accents"
            :key="a.id"
            class="accent-dot"
            :class="{ active: accentId === a.id }"
            :style="{ background: a.gradient }"
            :title="a.label"
            @click="setAccent(a.id)"
          />
        </div>
      </div>
      <div class="settings-row">
        <div class="settings-info">
          <div class="settings-name">菜单布局</div>
          <div class="settings-desc">侧栏分组 / 双栏导航 / 顶部 Tab / 网格首页</div>
        </div>
        <select
          :value="layoutMode"
          class="layout-select"
          @change="setLayout(($event.target as HTMLSelectElement).value as LayoutMode)"
        >
          <option value="sidebar">侧栏分组</option>
          <option value="dual">双栏导航</option>
          <option value="tabbar">顶部 Tab</option>
          <option value="grid">网格首页</option>
        </select>
      </div>
    </div>

    <!-- 快捷键 -->
    <div class="settings-section">
      <div class="settings-section-title">快捷键</div>
      <div class="shortcut-list">
        <div class="shortcut-row">
          <span>搜索工具</span>
          <kbd>Ctrl</kbd>
          <kbd>K</kbd>
        </div>
        <div class="shortcut-row">
          <span>切换收藏 1-9</span>
          <kbd>1</kbd>
          ~
          <kbd>9</kbd>
        </div>
        <div class="shortcut-row">
          <span>后退</span>
          <kbd>Alt</kbd>
          <kbd>←</kbd>
        </div>
        <div class="shortcut-row">
          <span>前进</span>
          <kbd>Alt</kbd>
          <kbd>→</kbd>
        </div>
        <div class="shortcut-row">
          <span>关闭弹窗 / 清除搜索</span>
          <kbd>Esc</kbd>
        </div>
        <div class="shortcut-row">
          <span>JSON 美化 (JSON 工具内)</span>
          <kbd>Ctrl</kbd>
          <kbd>Enter</kbd>
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
          <FolderOpen :size="14" />
          打开
        </BaseButton>
      </div>
    </div>

    <!-- 数据 -->
    <div class="settings-section">
      <div class="settings-section-title">数据管理</div>
      <div class="settings-row">
        <div class="settings-info">
          <div class="settings-name">导出全部数据</div>
          <div class="settings-desc">收藏、工具输入、使用统计、布局偏好导出为 JSON</div>
        </div>
        <BaseButton variant="ghost" @click="exportData">
          <Download :size="14" />
          导出
        </BaseButton>
      </div>
      <div class="settings-row">
        <div class="settings-info">
          <div class="settings-name">导入数据</div>
          <div class="settings-desc">从导出的 JSON 文件恢复数据</div>
        </div>
        <BaseButton variant="ghost" @click="importData">
          <Upload :size="14" />
          导入
        </BaseButton>
        <input ref="importInput" type="file" accept=".json" hidden @change="onImportFile" />
      </div>
      <div class="settings-row">
        <div class="settings-info">
          <div class="settings-name">清除工具输入缓存</div>
          <div class="settings-desc">清除所有工具的输入记忆 (不影响配置与收藏)</div>
        </div>
        <BaseButton variant="danger" @click="clearCache">清除</BaseButton>
      </div>
      <div class="settings-row">
        <div class="settings-info">
          <div class="settings-name">清除使用统计</div>
          <div class="settings-desc">重置工具访问次数统计</div>
        </div>
        <BaseButton variant="ghost" @click="clearUsageStats">清除</BaseButton>
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
          <div class="about-version">v1.0.1 · 程序员日常开发工具箱</div>
          <div class="about-tech">Vite 8 · Vue 3 · Tauri 2 · {{ toolCount }} 个工具</div>
        </div>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  Settings,
  Sun,
  Moon,
  Monitor,
  FolderOpen,
  Download,
  Upload,
  SlidersHorizontal,
} from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useTheme } from '@/composables/useTheme';
import { useToast } from '@/composables/useToast';
import { clearAllToolState } from '@/composables/useToolState';
import { useFavorites } from '@/composables/useFavorites';
import { useLayout, type LayoutMode } from '@/composables/useLayout';
import { useUsageStats } from '@/composables/useUsageStats';
import { tools } from '@/config/tools';

const toast = useToast();
const { mode, setMode } = useTheme();
const { favorites } = useFavorites();
const { layoutMode, accentId, accents, setLayout, setAccent } = useLayout();
const { clearStats } = useUsageStats();
const importInput = ref<HTMLInputElement | null>(null);
const toolCount = tools.length;

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

function clearUsageStats() {
  clearStats();
  toast.success('已清除使用统计');
}

function resetFavorites() {
  favorites.value = ['json', 'regex', 'timestamp'];
  toast.success('已重置收藏列表');
}

function exportData() {
  const data: Record<string, unknown> = {};
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key && key.startsWith('xuya_')) {
      data[key] = localStorage.getItem(key);
    }
  }
  data['__exportTime'] = new Date().toISOString();
  data['__version'] = '1.0.0';
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `xuya-backup-${Date.now()}.json`;
  a.click();
  URL.revokeObjectURL(url);
  toast.success('已导出数据');
}

function importData() {
  importInput.value?.click();
}

function onImportFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => {
    try {
      const data = JSON.parse(reader.result as string);
      let count = 0;
      for (const [key, val] of Object.entries(data)) {
        if (key.startsWith('xuya_') && typeof val === 'string') {
          localStorage.setItem(key, val);
          count++;
        }
      }
      toast.success(`已导入 ${count} 条数据，刷新后生效`);
      setTimeout(() => location.reload(), 1500);
    } catch {
      toast.error('导入失败：文件格式无效');
    }
  };
  reader.readAsText(file);
  (e.target as HTMLInputElement).value = '';
}

onMounted(async () => {
  try {
    const { isEnabled } = await import('@tauri-apps/plugin-autostart');
    autostartEnabled.value = await isEnabled();
  } catch {
    /* 非 Tauri 环境 */
  }
  try {
    const dataDirPath = (await import('@tauri-apps/api/path')).appDataDir;
    dataDir.value = await dataDirPath();
  } catch {
    /* 非 Tauri 环境 */
  }
});
</script>

<style scoped>
.settings-section {
  margin-bottom: 28px;
}
.settings-section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.7px;
  color: var(--xuya-text-tertiary);
  margin-bottom: 10px;
  padding-left: 4px;
}
.settings-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 16px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  margin-bottom: 8px;
}
.settings-info {
  flex: 1;
  min-width: 0;
}
.settings-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--xuya-text);
}
.settings-desc {
  font-size: 12px;
  color: var(--xuya-text-tertiary);
  margin-top: 3px;
  line-height: 1.4;
  word-break: break-all;
}
.small {
  font-size: 11.5px;
}

.seg {
  display: inline-flex;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius);
  padding: 3px;
  gap: 2px;
}
.seg button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 6px 14px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-radius: var(--xuya-radius-sm);
  transition: 0.1s;
  cursor: pointer;
}
.seg button.active {
  background: var(--xuya-bg-elevated);
  color: var(--xuya-accent);
  font-weight: 600;
  box-shadow: var(--xuya-shadow-sm);
}

.accent-row {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.accent-dot {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.accent-dot:hover {
  transform: scale(1.1);
}
.accent-dot.active {
  border-color: var(--xuya-text);
  box-shadow: 0 0 0 2px var(--xuya-card-bg);
}

.layout-select {
  padding: 6px 12px;
  font-size: 12px;
  color: var(--xuya-text);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  outline: none;
}

.shortcut-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px 16px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.shortcut-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--xuya-text-secondary);
}
.shortcut-row span {
  flex: 1;
}
kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  height: 22px;
  padding: 0 6px;
  font-family: var(--xuya-font-mono);
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: 4px;
  box-shadow: 0 1px 0 var(--xuya-border);
}

.switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  flex-shrink: 0;
}
.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}
.slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background: var(--xuya-border-strong);
  border-radius: 24px;
  transition: var(--xuya-duration) var(--xuya-ease);
}
.slider::before {
  content: '';
  position: absolute;
  height: 18px;
  width: 18px;
  left: 3px;
  top: 3px;
  background: #fff;
  border-radius: 50%;
  transition: var(--xuya-duration) var(--xuya-ease);
  box-shadow: var(--xuya-shadow-sm);
}
.switch input:checked + .slider {
  background: var(--xuya-accent);
}
.switch input:checked + .slider::before {
  transform: translateX(20px);
}

.about-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 20px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.about-logo {
  width: 48px;
  height: 48px;
  border-radius: var(--xuya-radius);
  object-fit: cover;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.18);
  flex-shrink: 0;
}
.about-name {
  font-size: 16px;
  font-weight: 700;
  color: var(--xuya-text);
}
.about-version {
  font-size: 12.5px;
  color: var(--xuya-text-secondary);
  margin-top: 3px;
}
.about-tech {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  margin-top: 6px;
  font-family: var(--xuya-font-mono);
}
</style>
