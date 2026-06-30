<template>
  <div class="md-layout">
    <!-- 左侧:工具列表(常驻) -->
    <aside class="md-master">
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

      <nav class="md-list">
        <!-- 收藏区 (仅在非搜索时显示) -->
        <template v-if="!keyword && favoriteTools.length">
          <div class="md-group-label">
            <Star :size="11" /> 收藏
          </div>
          <button
            v-for="t in favoriteTools"
            :key="t.id"
            class="md-item"
            :class="{ active: isActive(t) }"
            @click="selectTool(t)"
          >
            <span class="md-item-icon">
              <component :is="t.icon" :size="16" />
            </span>
            <span class="md-item-info">
              <span class="md-item-name">{{ t.name }}</span>
              <span class="md-item-desc">{{ t.description }}</span>
            </span>
            <span
              class="md-star"
              :class="{ on: true }"
              title="取消收藏"
              @click.stop="toggleFavorite(t.id)"
            >
              <Star :size="14" fill="currentColor" />
            </span>
          </button>
        </template>

        <!-- 分类分组 -->
        <template v-for="(group, cat) in groupedTools" :key="cat">
          <div class="md-group-label">{{ CATEGORY_LABELS[cat] }}</div>
          <button
            v-for="t in group"
            :key="t.id"
            class="md-item"
            :class="{ active: isActive(t) }"
            @click="selectTool(t)"
          >
            <span class="md-item-icon">
              <component :is="t.icon" :size="16" />
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
        </template>
        <div v-if="!filteredTools.length" class="md-empty">
          <SearchX :size="22" />
          <p>没有匹配 "{{ keyword }}" 的工具</p>
        </div>
      </nav>

      <div class="md-master-foot">
        <span class="md-foot-text">v0.1.0 · {{ tools.length }} 个工具</span>
        <button class="md-settings-btn" title="设置" @click="goSettings">
          <Settings :size="14" />
        </button>
      </div>
    </aside>

    <!-- 右侧:工具详情(路由) -->
    <section class="md-detail">
      <ErrorBoundary>
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </ErrorBoundary>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { Search, SearchX, X, Star, Settings } from '@lucide/vue';
import { tools, CATEGORY_LABELS, type ToolMeta, type ToolCategory } from '@/config/tools';
import { useFavorites } from '@/composables/useFavorites';
import ErrorBoundary from './ErrorBoundary.vue';

const router = useRouter();
const route = useRoute();
const { favorites, isFavorite, toggleFavorite } = useFavorites();

const keyword = ref('');
const searchInputEl = ref<HTMLInputElement | null>(null);

function focusSearch(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    searchInputEl.value?.focus();
    searchInputEl.value?.select();
  }
}

onMounted(() => window.addEventListener('keydown', focusSearch));
onUnmounted(() => window.removeEventListener('keydown', focusSearch));

const filteredTools = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  if (!kw) return tools;
  return tools.filter((t) =>
    [t.name, t.description, ...t.keywords].join(' ').toLowerCase().includes(kw)
  );
});

/** 收藏工具 (按收藏顺序) */
const favoriteTools = computed(() =>
  favorites.value
    .map((id) => tools.find((t) => t.id === id))
    .filter((t): t is ToolMeta => !!t)
);

/** 按分类分组 (排除已收藏的,避免重复显示; 搜索时显示全部) */
const groupedTools = computed(() => {
  const groups: Partial<Record<ToolCategory, ToolMeta[]>> = {};
  for (const t of filteredTools.value) {
    if (!groups[t.category]) groups[t.category] = [];
    groups[t.category]!.push(t);
  }
  return groups;
});

function isActive(t: ToolMeta) {
  return route.path === t.route;
}

function selectTool(t: ToolMeta) {
  router.push(t.route);
}

function goSettings() {
  router.push('/settings');
}
</script>

<style scoped>
.md-layout {
  display: flex;
  height: 100%;
  min-height: 0;
}

/* ===== 左侧 master ===== */
.md-master {
  width: 286px;
  flex-shrink: 0;
  background: var(--xuya-bg-elevated);
  border-right: 1px solid var(--xuya-border);
  display: flex;
  flex-direction: column;
}

.md-master-head {
  flex-shrink: 0;
  padding: 14px 14px 10px;
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
  transition: border-color var(--xuya-duration-fast) var(--xuya-ease),
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

.md-list {
  flex: 1;
  overflow: auto;
  padding: 2px 10px 12px;
}
.md-group-label {
  font-size: 10.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.7px;
  color: var(--xuya-text-tertiary);
  padding: 14px 10px 5px;
  display: flex;
  align-items: center;
  gap: 5px;
}
.md-group-label:first-child {
  padding-top: 4px;
}
.md-item {
  display: flex;
  align-items: center;
  gap: 11px;
  width: 100%;
  text-align: left;
  padding: 7px 10px;
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: background var(--xuya-duration-fast) var(--xuya-ease),
    color var(--xuya-duration-fast) var(--xuya-ease),
    transform var(--xuya-duration-fast) var(--xuya-ease);
  border: none;
  background: none;
  color: var(--xuya-text);
  position: relative;
}
.md-item:hover {
  background: var(--xuya-input-bg);
}
.md-item.active {
  background: var(--xuya-accent-soft);
}
/* 激活态左侧指示条 */
.md-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 18px;
  background: var(--xuya-accent-gradient);
  border-radius: 0 2px 2px 0;
}
.md-item.active .md-item-icon {
  background: var(--xuya-accent-gradient);
  color: var(--xuya-on-accent);
  box-shadow: 0 2px 6px var(--xuya-accent-glow);
}
.md-item.active .md-item-name {
  color: var(--xuya-text);
  font-weight: 600;
}
.md-item-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  flex-shrink: 0;
  border-radius: 8px;
  background: var(--xuya-input-bg);
  color: var(--xuya-text-secondary);
  transition: background var(--xuya-duration-fast) var(--xuya-ease),
    color var(--xuya-duration-fast) var(--xuya-ease),
    box-shadow var(--xuya-duration-fast) var(--xuya-ease),
    transform var(--xuya-duration-fast) var(--xuya-ease);
}
.md-item:hover .md-item-icon {
  color: var(--xuya-text);
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
  transition: opacity var(--xuya-duration-fast) var(--xuya-ease),
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

.md-master-foot {
  flex-shrink: 0;
  padding: 8px 12px 8px 16px;
  border-top: 1px solid var(--xuya-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.md-foot-text {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  letter-spacing: 0.2px;
}
.md-settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--xuya-text-tertiary);
  border-radius: var(--xuya-radius-sm);
  transition: background var(--xuya-duration-fast) var(--xuya-ease),
    color var(--xuya-duration-fast) var(--xuya-ease);
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
</style>
