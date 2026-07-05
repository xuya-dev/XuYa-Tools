<template>
  <div class="md-layout" :class="'layout-' + layoutMode">
    <LayoutSwitcher :visible="switcherVisible" @close="switcherVisible = false" />

    <!-- ====== Sidebar 布局 (默认) ====== -->
    <template v-if="layoutMode === 'sidebar'">
      <!-- 左侧:工具列表(常驻,占满全高) -->
      <aside class="md-master">
        <!-- 品牌头 (拖拽区) -->
        <div class="md-brand" @mousedown="startDrag">
          <img class="md-brand-logo" src="/Logo.png" alt="" />
          <div class="md-brand-text">
            <span class="md-brand-name">XuYa Tools</span>
            <small class="md-brand-sub">开发工具箱 · v1.0.4</small>
          </div>
        </div>

        <!-- 搜索 -->
        <div class="md-master-head">
          <div class="md-search">
            <Search :size="15" />
            <input
              ref="searchInputEl"
              v-model="keyword"
              type="text"
              placeholder="搜索工具... (Ctrl/⌘+K)"
              spellcheck="false"
            />
            <button v-if="keyword" class="md-search-clear" @click="keyword = ''">
              <X :size="13" />
            </button>
          </div>
        </div>

        <!-- 导航 -->
        <nav class="md-list">
          <!-- 常用区 (按使用频率, 仅非搜索 + 有数据时) -->
          <div
            v-if="!keyword && recentTools.length"
            class="md-cat"
            :class="{ collapsed: recentCollapsed }"
          >
            <button class="md-cat-header" @click="recentCollapsed = !recentCollapsed">
              <Clock :size="11" class="md-cat-star" />
              <span class="md-cat-name">常用</span>
              <span class="md-cat-count">{{ recentTools.length }}</span>
              <ChevronDown :size="12" class="md-cat-arrow" />
            </button>
            <div class="md-cat-items">
              <button
                v-for="t in recentTools"
                :key="t.id"
                class="md-item"
                :class="{ active: isActive(t) }"
                @click="selectTool(t)"
              >
                <span class="md-item-icon">
                  <component :is="t.icon" :size="15" />
                </span>
                <span class="md-item-info">
                  <span class="md-item-name">{{ t.name }}</span>
                  <span class="md-item-desc">{{ t.visits }} 次访问</span>
                </span>
              </button>
            </div>
          </div>

          <!-- 收藏区 (仅在非搜索时显示) -->
          <div
            v-if="!keyword && favoriteTools.length"
            class="md-cat"
            :class="{ collapsed: favCollapsed }"
          >
            <button class="md-cat-header" @click="favCollapsed = !favCollapsed">
              <Star :size="11" fill="currentColor" class="md-cat-star" />
              <span class="md-cat-name">收藏</span>
              <span class="md-cat-count">{{ favoriteTools.length }}</span>
              <span v-if="!favCollapsed" class="md-cat-hint">1-9</span>
              <ChevronDown :size="12" class="md-cat-arrow" />
            </button>
            <div class="md-cat-items">
              <button
                v-for="t in favoriteTools"
                :key="t.id"
                class="md-item"
                :class="{ active: isActive(t) }"
                @click="selectTool(t)"
              >
                <span class="md-item-icon">
                  <component :is="t.icon" :size="15" />
                </span>
                <span class="md-item-info">
                  <span class="md-item-name">{{ t.name }}</span>
                  <span class="md-item-desc">{{ t.description }}</span>
                </span>
                <span class="md-star on" title="取消收藏" @click.stop="toggleFavorite(t.id)">
                  <Star :size="14" fill="currentColor" />
                </span>
              </button>
            </div>
          </div>

          <!-- 分类分组 (可折叠) -->
          <div
            v-for="(group, cat) in groupedTools"
            :key="cat"
            class="md-cat"
            :class="{ collapsed: keyword ? false : isCollapsed(cat) }"
          >
            <button class="md-cat-header" @click="toggle(cat)">
              <span class="md-cat-dot" :style="{ background: CATEGORY_COLORS[cat] }"></span>
              <span class="md-cat-name">{{ CATEGORY_LABELS[cat] }}</span>
              <span class="md-cat-count">{{ group.length }}</span>
              <ChevronDown :size="12" class="md-cat-arrow" />
            </button>
            <div class="md-cat-items">
              <button
                v-for="t in group"
                :key="t.id"
                class="md-item"
                :class="{ active: isActive(t) }"
                @click="selectTool(t)"
              >
                <span class="md-item-icon">
                  <component :is="t.icon" :size="15" />
                </span>
                <span class="md-item-info">
                  <span class="md-item-name">{{ t.name }}</span>
                  <span class="md-item-desc">{{ t.description }}</span>
                </span>
                <span
                  class="md-star"
                  :class="{ on: isFavorite(t.id) }"
                  :title="isFavorite(t.id) ? '取消收藏' : '添加收藏'"
                  @click.stop="toggleFavorite(t.id)"
                >
                  <Star :size="14" :fill="isFavorite(t.id) ? 'currentColor' : 'none'" />
                </span>
              </button>
            </div>
          </div>

          <div v-if="!filteredTools.length" class="md-empty">
            <SearchX :size="22" />
            <p>没有匹配 "{{ keyword }}" 的工具</p>
          </div>
        </nav>

        <!-- 底部状态栏 -->
        <div class="md-master-foot">
          <span class="md-status">
            <span class="md-status-dot"></span>
            本地运行
          </span>
          <span class="md-foot-count">{{ tools.length }} tools</span>
          <button class="md-settings-btn" title="设置" @click="goSettings">
            <Settings :size="14" />
          </button>
        </div>
      </aside>

      <!-- 右侧:主体 (工具条 + 工具详情) -->
      <section class="md-detail">
        <AppToolBar @switcher="switcherVisible = true" />
        <div class="md-detail-body">
          <ErrorBoundary>
            <router-view v-slot="{ Component }">
              <transition name="fade" mode="out-in">
                <component :is="Component" />
              </transition>
            </router-view>
          </ErrorBoundary>
        </div>
      </section>
    </template>

    <!-- ====== Tabbar 布局 ====== -->
    <template v-else-if="layoutMode === 'tabbar'">
      <section class="md-detail">
        <AppToolBar show-brand @switcher="switcherVisible = true" />
        <div class="tabbar-cats">
          <button
            v-for="cat in allCategories"
            :key="cat"
            class="tabbar-cat"
            :class="{ active: activeTabCategory === cat }"
            @click="activeTabCategory = cat"
          >
            <span class="cat-dot" :style="{ background: CATEGORY_COLORS[cat] }"></span>
            {{ CATEGORY_LABELS[cat] }}
          </button>
        </div>
        <div class="tabbar-body">
          <aside class="tabbar-tools">
            <button
              v-for="t in tabbarTools"
              :key="t.id"
              class="md-item"
              :class="{ active: isActive(t) }"
              @click="selectTool(t)"
            >
              <span class="md-item-icon"><component :is="t.icon" :size="15" /></span>
              <span class="md-item-info">
                <span class="md-item-name">{{ t.name }}</span>
                <span class="md-item-desc">{{ t.description }}</span>
              </span>
            </button>
          </aside>
          <div class="md-detail-body">
            <ErrorBoundary>
              <router-view v-slot="{ Component }">
                <transition name="fade" mode="out-in">
                  <component :is="Component" />
                </transition>
              </router-view>
            </ErrorBoundary>
          </div>
        </div>
      </section>
    </template>

    <!-- ====== Grid 布局 ====== -->
    <template v-else-if="layoutMode === 'grid'">
      <section class="md-detail">
        <AppToolBar
          :show-brand="gridHomeVisible"
          :show-back="!gridHomeVisible"
          @switcher="switcherVisible = true"
          @back="gridHomeVisible = true"
        />
        <div class="md-detail-body">
          <!-- 工具详情 (选中某个工具时) -->
          <template v-if="activeTool && !gridHomeVisible">
            <ErrorBoundary>
              <router-view v-slot="{ Component }">
                <transition name="fade" mode="out-in">
                  <component :is="Component" />
                </transition>
              </router-view>
            </ErrorBoundary>
          </template>
          <!-- 网格首页 -->
          <div v-else class="grid-home" style="overflow: auto">
            <div class="grid-brand">
              <img class="grid-brand-logo" src="/Logo.png" alt="" />
              <div>
                <span class="grid-brand-name">XuYa Tools</span>
                <small class="grid-brand-sub">开发工具箱 · {{ tools.length }} 个工具</small>
              </div>
            </div>
            <div class="md-search" style="max-width: 400px; margin: 0 auto 20px">
              <Search :size="15" />
              <input v-model="keyword" type="text" placeholder="搜索工具..." spellcheck="false" />
              <button v-if="keyword" class="md-search-clear" @click="keyword = ''">
                <X :size="13" />
              </button>
            </div>
            <div v-for="cat in allCategories" :key="cat" class="grid-section">
              <div class="grid-section-title">
                <span class="cat-dot" :style="{ background: CATEGORY_COLORS[cat] }"></span>
                {{ CATEGORY_LABELS[cat] }}
              </div>
              <div class="grid-tiles">
                <button
                  v-for="t in groupedTools[cat]"
                  :key="t.id"
                  class="grid-tile"
                  :class="{ active: isActive(t) }"
                  @click="selectTool(t)"
                >
                  <span class="grid-tile-icon"><component :is="t.icon" :size="18" /></span>
                  <span class="grid-tile-name">{{ t.name }}</span>
                  <span class="grid-tile-desc">{{ t.description }}</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>
    </template>

    <!-- ====== Dual 布局 (公共头 + 分类列 + 工具列) ====== -->
    <template v-else-if="layoutMode === 'dual'">
      <div class="dual-sidebar">
        <!-- 公共头部: 和侧栏布局一样的品牌头 + 搜索 -->
        <div class="md-brand" @mousedown="startDrag">
          <img class="md-brand-logo" src="/Logo.png" alt="" />
          <div class="md-brand-text">
            <span class="md-brand-name">XuYa Tools</span>
            <small class="md-brand-sub">开发工具箱 · v1.0.4</small>
          </div>
        </div>
        <div class="md-master-head">
          <div class="md-search">
            <Search :size="15" />
            <input
              ref="searchInputEl"
              v-model="keyword"
              type="text"
              placeholder="搜索工具... (Ctrl/⌘+K)"
              spellcheck="false"
            />
            <button v-if="keyword" class="md-search-clear" @click="keyword = ''">
              <X :size="13" />
            </button>
          </div>
        </div>
        <!-- 双列 -->
        <div class="dual-columns">
          <aside class="dual-cats">
            <nav class="dual-cat-list">
              <button
                v-for="cat in allCategories"
                :key="cat"
                class="dual-cat-item"
                :class="{ active: activeTabCategory === cat }"
                @click="activeTabCategory = cat"
              >
                <component :is="categoryIcons[cat]" :size="18" class="dual-cat-icon" />
                <span class="dual-cat-name">{{ CATEGORY_LABELS[cat] }}</span>
                <span class="dual-cat-count">{{ groupedTools[cat]?.length ?? 0 }}</span>
              </button>
            </nav>
          </aside>
          <aside class="dual-tools">
            <nav class="md-list dual-tools-list">
              <button
                v-for="t in dualTools"
                :key="t.id"
                class="md-item"
                :class="{ active: isActive(t) }"
                @click="selectTool(t)"
              >
                <span class="md-item-icon"><component :is="t.icon" :size="15" /></span>
                <span class="md-item-info">
                  <span class="md-item-name">{{ t.name }}</span>
                  <span class="md-item-desc">{{ t.description }}</span>
                </span>
                <span
                  class="md-star"
                  :class="{ on: isFavorite(t.id) }"
                  @click.stop="toggleFavorite(t.id)"
                >
                  <Star :size="14" :fill="isFavorite(t.id) ? 'currentColor' : 'none'" />
                </span>
              </button>
            </nav>
          </aside>
        </div>
        <!-- 底部 -->
        <div class="md-master-foot">
          <span class="md-status">
            <span class="md-status-dot"></span>
            {{ tools.length }} tools
          </span>
          <button class="md-settings-btn" title="设置" @click="goSettings">
            <Settings :size="14" />
          </button>
        </div>
      </div>
      <section class="md-detail">
        <AppToolBar @switcher="switcherVisible = true" />
        <div class="md-detail-body">
          <ErrorBoundary>
            <router-view v-slot="{ Component }">
              <transition name="fade" mode="out-in">
                <component :is="Component" />
              </transition>
            </router-view>
          </ErrorBoundary>
        </div>
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import {
  Search,
  SearchX,
  X,
  Star,
  Settings,
  ChevronDown,
  ArrowLeftRight,
  Binary,
  Lock,
  Clock,
  Type,
  Globe,
  Terminal,
  Sparkles,
} from '@lucide/vue';
import {
  tools,
  CATEGORY_LABELS,
  CATEGORY_COLORS,
  type ToolMeta,
  type ToolCategory,
} from '@/config/tools';
import { useFavorites } from '@/composables/useFavorites';
import { useCollapsedCategories } from '@/composables/useCollapsedCategories';
import { useWindow } from '@/composables/useWindow';
import { useLayout } from '@/composables/useLayout';
import { useUsageStats } from '@/composables/useUsageStats';
import AppToolBar from './AppToolBar.vue';
import ErrorBoundary from './ErrorBoundary.vue';
import LayoutSwitcher from './LayoutSwitcher.vue';

const router = useRouter();
const route = useRoute();
const { favorites, isFavorite, toggleFavorite } = useFavorites();
const { isCollapsed, toggle } = useCollapsedCategories();
const { startDrag } = useWindow();
const { layoutMode } = useLayout();
const { recordVisit } = useUsageStats();
const switcherVisible = ref(false);

const categoryIcons: Record<ToolCategory, typeof ArrowLeftRight> = {
  converter: ArrowLeftRight,
  encoder: Binary,
  crypto: Lock,
  datetime: Clock,
  text: Type,
  network: Globe,
  devops: Terminal,
  ai: Sparkles,
};

const keyword = ref('');
const searchInputEl = ref<HTMLInputElement | null>(null);
const favCollapsed = ref(false);
const recentCollapsed = ref(false);

const { stats: usageStats } = useUsageStats();

const recentTools = computed(() =>
  Object.entries(usageStats.value)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 5)
    .map(([id, visits]) => {
      const t = tools.find((item) => item.id === id);
      return t ? { ...t, visits } : null;
    })
    .filter((t): t is ToolMeta & { visits: number } => !!t),
);

function onGlobalKeydown(e: KeyboardEvent) {
  const tag = (e.target as HTMLElement)?.tagName;
  const inInput =
    tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable;

  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    searchInputEl.value?.focus();
    searchInputEl.value?.select();
    return;
  }

  if (e.key === 'Escape') {
    if (switcherVisible.value) {
      switcherVisible.value = false;
      return;
    }
    if (keyword.value) {
      keyword.value = '';
      return;
    }
  }

  if (e.altKey && e.key === 'ArrowLeft') {
    e.preventDefault();
    router.back();
    return;
  }
  if (e.altKey && e.key === 'ArrowRight') {
    e.preventDefault();
    router.forward();
    return;
  }

  if (!inInput && !e.ctrlKey && !e.metaKey && !e.altKey) {
    const num = parseInt(e.key, 10);
    if (num >= 1 && num <= 9 && favorites.value.length >= num) {
      const favId = favorites.value[num - 1];
      const t = tools.find((item) => item.id === favId);
      if (t) {
        e.preventDefault();
        selectTool(t);
      }
    }
  }
}

onMounted(() => window.addEventListener('keydown', onGlobalKeydown));
onUnmounted(() => window.removeEventListener('keydown', onGlobalKeydown));

const filteredTools = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  if (!kw) return tools;
  return tools.filter((t) =>
    [t.name, t.description, ...t.keywords].join(' ').toLowerCase().includes(kw),
  );
});

const favoriteTools = computed(() =>
  favorites.value.map((id) => tools.find((t) => t.id === id)).filter((t): t is ToolMeta => !!t),
);

const groupedTools = computed(() => {
  const groups = {} as Record<ToolCategory, ToolMeta[]>;
  for (const t of filteredTools.value) {
    if (!groups[t.category]) groups[t.category] = [];
    groups[t.category].push(t);
  }
  return groups;
});

function isActive(t: ToolMeta) {
  return route.path === t.route;
}

async function selectTool(t: ToolMeta) {
  await router.push(t.route);
  gridHomeVisible.value = false;
  recordVisit(t.id);
}

const gridHomeVisible = ref(true);

const activeTool = computed(() => tools.find((t) => t.route === route.path));

function goSettings() {
  router.push('/settings');
}

const allCategories = computed(() => {
  const order: ToolCategory[] = [
    'converter',
    'encoder',
    'crypto',
    'datetime',
    'text',
    'network',
    'devops',
    'ai',
  ];
  return order.filter((c) => groupedTools.value[c]?.length);
});

const activeTabCategory = ref<ToolCategory | ''>('');
watch(
  [() => route.path, layoutMode],
  () => {
    if (layoutMode.value === 'tabbar' || layoutMode.value === 'dual') {
      const t = tools.find((item) => item.route === route.path);
      activeTabCategory.value = t?.category ?? '';
    }
  },
  { immediate: true },
);

const tabbarTools = computed(() => {
  if (!activeTabCategory.value) return filteredTools.value;
  return filteredTools.value.filter((t) => t.category === activeTabCategory.value);
});

const dualTools = computed(() => {
  if (!activeTabCategory.value) {
    const t = tools.find((item) => item.route === route.path);
    if (t) return filteredTools.value.filter((item) => item.category === t.category);
    return filteredTools.value;
  }
  return filteredTools.value.filter((t) => t.category === activeTabCategory.value);
});
</script>

<style scoped>
.md-layout {
  display: flex;
  flex: 1;
  min-width: 0;
  height: 100%;
  min-height: 0;
}

/* ===== 左侧 master ===== */
.md-master {
  width: 268px;
  min-width: 268px;
  flex-shrink: 0;
  background: var(--xuya-bg-elevated);
  border-right: 1px solid var(--xuya-border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 品牌头 (拖拽区) */
.md-brand {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 16px 18px 12px;
  -webkit-app-region: drag;
  user-select: none;
  flex-shrink: 0;
}
.md-brand-logo {
  width: 34px;
  height: 34px;
  border-radius: 9px;
  object-fit: cover;
  flex-shrink: 0;
  box-shadow: 0 3px 12px var(--xuya-accent-glow);
}
.md-brand-text {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}
.md-brand-name {
  font-size: 14.5px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--xuya-text);
}
.md-brand-sub {
  font-size: 10px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  letter-spacing: 0.1em;
  text-transform: uppercase;
  margin-top: 3px;
}

/* 搜索 */
.md-master-head {
  flex-shrink: 0;
  padding: 0 14px 10px;
}
.md-search {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px 0 12px;
  height: 36px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  color: var(--xuya-text-tertiary);
  transition:
    border-color var(--xuya-duration-fast) var(--xuya-ease),
    box-shadow var(--xuya-duration-fast) var(--xuya-ease),
    background var(--xuya-duration-fast) var(--xuya-ease);
}
.md-search:hover {
  border-color: var(--xuya-border-strong);
}
.md-search:focus-within {
  border-color: var(--xuya-accent);
  background: var(--xuya-bg-elevated);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.md-search input {
  flex: 1;
  border: none;
  background: none;
  color: var(--xuya-text);
  font-size: 13px;
  outline: none;
}
.md-search input::placeholder {
  color: var(--xuya-text-tertiary);
}
.md-search-clear {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  color: var(--xuya-text-tertiary);
  border-radius: 50%;
  transition: background var(--xuya-duration-fast) var(--xuya-ease);
}
.md-search-clear:hover {
  background: var(--xuya-border);
  color: var(--xuya-text);
}

/* 导航列表 */
.md-list {
  flex: 1;
  overflow: auto;
  padding: 6px 10px 12px;
}

/* 分类组 (可折叠) */
.md-cat {
  margin-bottom: 2px;
}
.md-cat-header {
  display: flex;
  align-items: center;
  gap: 7px;
  width: 100%;
  padding: 10px 10px 5px;
  border: none;
  background: none;
  cursor: pointer;
  user-select: none;
  transition: color var(--xuya-duration-fast) var(--xuya-ease);
  color: var(--xuya-text-tertiary);
}
.md-cat-header:hover {
  color: var(--xuya-text);
}
.md-cat-star {
  color: var(--xuya-warn);
  flex-shrink: 0;
}
.md-cat-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}
.md-cat-name {
  flex: 1;
  text-align: left;
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.12em;
}
.md-cat-count {
  font-family: var(--xuya-font-mono);
  font-size: 9.5px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: 5px;
  padding: 0 6px;
  line-height: 16px;
  min-width: 16px;
  text-align: center;
}
.md-cat-arrow {
  opacity: 0.6;
  flex-shrink: 0;
  transition: transform var(--xuya-duration) var(--xuya-ease);
}
.md-cat-hint {
  font-size: 9px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: 3px;
  padding: 0 4px;
  line-height: 14px;
  font-family: var(--xuya-font-mono);
}
.md-cat.collapsed .md-cat-arrow {
  transform: rotate(-90deg);
}
.md-cat.collapsed .md-cat-items {
  display: none;
}

/* 导航项 */
.md-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  text-align: left;
  padding: 6px 10px;
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition:
    background var(--xuya-duration-fast) var(--xuya-ease),
    color var(--xuya-duration-fast) var(--xuya-ease);
  border: none;
  background: none;
  color: var(--xuya-text);
  margin: 1px 0;
  position: relative;
}
.md-item:hover {
  background: var(--xuya-input-bg);
}
.md-item.active {
  background: var(--xuya-accent-soft);
}
.md-item-icon {
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  flex-shrink: 0;
  border-radius: var(--xuya-radius-sm);
  background: var(--xuya-input-bg);
  color: var(--xuya-text-secondary);
  border: 1px solid var(--xuya-border);
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
}
.md-item:hover .md-item-icon {
  border-color: var(--xuya-accent);
  color: var(--xuya-accent);
}
.md-item.active .md-item-icon {
  background: var(--xuya-accent);
  border-color: var(--xuya-accent);
  color: var(--xuya-on-accent);
}
.md-item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.md-item-name {
  font-size: 13px;
  color: var(--xuya-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.md-item.active .md-item-name {
  font-weight: 600;
}
.md-item-desc {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
/* 收藏星标 */
.md-star {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  color: var(--xuya-text-tertiary);
  opacity: 0;
  transition:
    opacity var(--xuya-duration-fast) var(--xuya-ease),
    color var(--xuya-duration-fast) var(--xuya-ease),
    background var(--xuya-duration-fast) var(--xuya-ease);
}
.md-item:hover .md-star,
.md-star.on {
  opacity: 1;
}
.md-star:hover {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}
.md-star.on {
  color: var(--xuya-warn);
}

.md-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 40px 16px;
  color: var(--xuya-text-tertiary);
}
.md-empty p {
  font-size: 12px;
}

/* 底部状态栏 */
.md-master-foot {
  flex-shrink: 0;
  padding: 10px 16px;
  border-top: 1px solid var(--xuya-border);
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
  display: flex;
  align-items: center;
  gap: 8px;
}
.md-status {
  display: flex;
  align-items: center;
  gap: 5px;
}
.md-status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--xuya-success);
  display: inline-block;
  box-shadow: 0 0 6px var(--xuya-success);
}
.md-foot-count {
  flex: 1;
  text-align: right;
}
.md-settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  background: transparent;
  color: var(--xuya-text-tertiary);
  border-radius: var(--xuya-radius-sm);
  transition:
    background var(--xuya-duration-fast) var(--xuya-ease),
    color var(--xuya-duration-fast) var(--xuya-ease);
  cursor: pointer;
}
.md-settings-btn:hover {
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
}

/* ===== 右侧 detail ===== */
.md-detail {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.md-detail-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* ===== Grid 布局 ===== */
.grid-home {
  overflow: auto;
  padding: 20px 24px;
  flex: 1;
}
.grid-section {
  margin-bottom: 24px;
}
.grid-section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--xuya-text-tertiary);
  margin-bottom: 10px;
}
.grid-tiles {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 10px;
}
.grid-tile {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 16px 10px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-lg);
  cursor: pointer;
  text-align: center;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
}
.grid-tile:hover {
  border-color: var(--xuya-accent);
  transform: translateY(-2px);
  box-shadow: var(--xuya-shadow-hover);
}
.grid-tile.active {
  border-color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
}
.grid-tile-icon {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: var(--xuya-input-bg);
  color: var(--xuya-text-secondary);
  border: 1px solid var(--xuya-border);
}
.grid-tile:hover .grid-tile-icon,
.grid-tile.active .grid-tile-icon {
  background: var(--xuya-accent);
  border-color: var(--xuya-accent);
  color: var(--xuya-on-accent);
}
.grid-tile-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text);
}
.grid-tile-desc {
  font-size: 10px;
  color: var(--xuya-text-tertiary);
  line-height: 1.3;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.cat-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

/* ===== Tabbar 布局 ===== */
.tabbar-cats {
  display: flex;
  gap: 0;
  padding: 0 16px;
  background: var(--xuya-bg-elevated);
  border-bottom: 1px solid var(--xuya-border);
  flex-shrink: 0;
}
.tabbar-cat {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
  cursor: pointer;
  border: none;
  background: transparent;
  border-bottom: 2px solid transparent;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
}
.tabbar-cat:hover {
  color: var(--xuya-text);
}
.tabbar-cat.active {
  color: var(--xuya-accent);
  border-bottom-color: var(--xuya-accent);
}
.tabbar-body {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
}
.tabbar-tools {
  width: 210px;
  min-width: 210px;
  background: var(--xuya-bg-elevated);
  border-right: 1px solid var(--xuya-border);
  padding: 8px;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.tabbar-brand {
  margin-left: auto;
  padding-right: 16px;
  font-size: 12px;
  font-weight: 700;
  color: var(--xuya-text-tertiary);
  letter-spacing: -0.01em;
}

/* ===== Grid 品牌 ===== */
.grid-brand {
  display: flex;
  align-items: center;
  gap: 12px;
  justify-content: center;
  margin-bottom: 16px;
}
.grid-brand-logo {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  object-fit: cover;
  box-shadow: 0 3px 12px var(--xuya-accent-glow);
}
.grid-brand-name {
  display: block;
  font-size: 18px;
  font-weight: 700;
  color: var(--xuya-text);
}
.grid-brand-sub {
  display: block;
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  margin-top: 2px;
}

/* ===== Dual 布局 ===== */
.dual-sidebar {
  display: flex;
  flex-direction: column;
  background: var(--xuya-bg-elevated);
  border-right: 1px solid var(--xuya-border);
  flex-shrink: 0;
}

/* 双列容器 */
.dual-columns {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
}

/* 第一列: 分类 */
.dual-cats {
  width: 110px;
  min-width: 110px;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--xuya-border);
}
.dual-cat-list {
  flex: 1;
  overflow: auto;
  padding: 6px 4px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.dual-cat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 5px;
  padding: 10px 2px;
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  border: none;
  background: transparent;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  text-align: center;
  width: 100%;
}
.dual-cat-item:hover {
  background: var(--xuya-input-bg);
}
.dual-cat-item.active {
  background: var(--xuya-accent-soft);
}
.dual-cat-icon {
  color: var(--xuya-text-secondary);
  transition: color var(--xuya-duration-fast);
}
.dual-cat-item.active .dual-cat-icon {
  color: var(--xuya-accent);
}
.dual-cat-name {
  font-size: 10.5px;
  font-weight: 600;
  color: var(--xuya-text);
  line-height: 1.3;
}
.dual-cat-item.active .dual-cat-name {
  color: var(--xuya-accent);
}
.dual-cat-count {
  font-size: 9px;
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
}

/* 第二列: 工具 */
.dual-tools {
  width: 200px;
  min-width: 200px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.dual-tools-list {
  flex: 1;
  overflow: auto;
  padding: 6px 8px;
}

.dual-sidebar .md-master-foot {
  flex-shrink: 0;
}

.md-item:hover :deep(.brand-icon),
.md-item.active :deep(.brand-icon),
.grid-tile:hover :deep(.brand-icon),
.grid-tile.active :deep(.brand-icon) {
  color: inherit !important;
}
</style>
