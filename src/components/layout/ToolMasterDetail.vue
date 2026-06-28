<template>
  <div class="md-layout">
    <!-- 左侧:工具列表(常驻) -->
    <aside class="md-master">
      <div class="md-master-head">
        <div class="md-search">
          <Search :size="15" />
          <input
            v-model="keyword"
            type="text"
            placeholder="搜索工具..."
            spellcheck="false"
          />
        </div>
      </div>

      <nav class="md-list">
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
          </button>
        </template>
        <div v-if="!filteredTools.length" class="md-empty">
          <SearchX :size="22" />
          <p>没有匹配 "{{ keyword }}" 的工具</p>
        </div>
      </nav>

      <div class="md-master-foot">
        <span class="md-foot-text">v0.1.0 · {{ tools.length }} 个工具</span>
      </div>
    </aside>

    <!-- 右侧:工具详情(路由) -->
    <section class="md-detail">
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { Search, SearchX } from '@lucide/vue';
import { tools, CATEGORY_LABELS, type ToolMeta, type ToolCategory } from '@/config/tools';

const router = useRouter();
const route = useRoute();

const keyword = ref('');

const filteredTools = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  if (!kw) return tools;
  return tools.filter((t) =>
    [t.name, t.description, ...t.keywords].join(' ').toLowerCase().includes(kw)
  );
});

/** 按分类分组 (保持注册表顺序) */
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
  padding: 0 12px;
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
  color: #fff;
  box-shadow: 0 2px 6px var(--xuya-accent-glow);
}
.md-item.active .md-item-name {
  color: var(--xuya-accent);
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
  padding: 10px 16px;
  border-top: 1px solid var(--xuya-border);
}
.md-foot-text {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  letter-spacing: 0.2px;
}

/* ===== 右侧 detail ===== */
.md-detail {
  flex: 1;
  min-width: 0;
  overflow: auto;
  display: flex;
  flex-direction: column;
}
</style>
