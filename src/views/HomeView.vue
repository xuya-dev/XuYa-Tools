<template>
  <div class="home">
    <div class="home-inner">
      <!-- Hero -->
      <header class="hero">
        <div class="hero-badge">
          <Sparkles :size="13" />
          <span>程序员日常开发工具箱</span>
        </div>
        <h1 class="hero-title">XuYa Tools</h1>
        <p class="hero-subtitle">精选高频开发工具,开箱即用,数据本地处理,不上传服务器。</p>
      </header>

      <!-- 搜索 -->
      <SearchBar v-model="keyword" class="search" />

      <!-- 分类筛选 -->
      <div class="filters">
        <button
          class="filter-chip"
          :class="{ active: activeCategory === 'all' }"
          @click="activeCategory = 'all'"
        >
          全部 ({{ tools.length }})
        </button>
        <button
          v-for="(label, key) in CATEGORY_LABELS"
          :key="key"
          class="filter-chip"
          :class="{ active: activeCategory === key }"
          @click="activeCategory = key"
        >
          {{ label }} ({{ countByCategory(key) }})
        </button>
      </div>

      <!-- 工具网格 -->
      <div v-if="filtered.length" class="grid">
        <ToolCard v-for="t in filtered" :key="t.id" :tool="t" />
      </div>

      <!-- 空状态 -->
      <div v-else class="empty">
        <SearchX :size="32" />
        <p>没有找到匹配 "{{ keyword }}" 的工具</p>
      </div>

      <!-- 页脚 -->
      <footer class="home-footer">
        <span>XuYa Tools · v0.1.0</span>
        <span class="dot">·</span>
        <span>更多工具持续添加中</span>
      </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Sparkles, SearchX } from '@lucide/vue';
import { tools, CATEGORY_LABELS, type ToolCategory } from '@/config/tools';
import SearchBar from '@/components/home/SearchBar.vue';
import ToolCard from '@/components/home/ToolCard.vue';

const keyword = ref('');
const activeCategory = ref<ToolCategory | 'all'>('all');

function countByCategory(c: ToolCategory) {
  return tools.filter((t) => t.category === c).length;
}

const filtered = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  return tools.filter((t) => {
    const matchCat = activeCategory.value === 'all' || t.category === activeCategory.value;
    if (!matchCat) return false;
    if (!kw) return true;
    const haystack = [t.name, t.description, ...t.keywords].join(' ').toLowerCase();
    return haystack.includes(kw);
  });
});

// 快捷键 Ctrl/Cmd + K 聚焦搜索
function onKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    const input = document.querySelector<HTMLInputElement>('.search-input');
    input?.focus();
  }
}
onMounted(() => window.addEventListener('keydown', onKeydown));
onUnmounted(() => window.removeEventListener('keydown', onKeydown));
</script>

<style scoped>
.home {
  max-width: 1200px;
  margin: 0 auto;
  padding: 48px 32px 32px;
}
.home-inner {
  display: flex;
  flex-direction: column;
}

/* Hero */
.hero {
  text-align: center;
  margin-bottom: 32px;
}
.hero-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  font-size: 12px;
  color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
  border-radius: 999px;
  margin-bottom: 16px;
}
.hero-title {
  font-size: 34px;
  font-weight: 800;
  letter-spacing: -0.5px;
  background: var(--xuya-accent-gradient);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
  margin-bottom: 10px;
}
.hero-subtitle {
  font-size: 14px;
  color: var(--xuya-text-secondary);
}

/* 搜索 */
.search {
  margin-bottom: 18px;
}

/* 分类 */
.filters {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 24px;
}
.filter-chip {
  padding: 5px 13px;
  font-size: 12.5px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: 999px;
  transition: all 0.12s;
}
.filter-chip:hover {
  color: var(--xuya-text);
}
.filter-chip.active {
  color: #fff;
  background: var(--xuya-accent);
  border-color: var(--xuya-accent);
}

/* 网格 */
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}

/* 空状态 */
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 0;
  color: var(--xuya-text-tertiary);
}
.empty p {
  font-size: 14px;
}

/* 页脚 */
.home-footer {
  margin-top: 48px;
  text-align: center;
  font-size: 12px;
  color: var(--xuya-text-tertiary);
}
.dot {
  margin: 0 6px;
}
</style>
