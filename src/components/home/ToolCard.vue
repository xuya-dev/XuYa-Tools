<template>
  <button class="tool-card" @click="onClick">
    <div class="card-icon-wrap">
      <component :is="tool.icon" :size="22" />
    </div>
    <div class="card-body">
      <h3 class="card-name">{{ tool.name }}</h3>
      <p class="card-desc">{{ tool.description }}</p>
    </div>
    <ChevronRight :size="16" class="card-arrow" />
  </button>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { ChevronRight } from '@lucide/vue';
import type { ToolMeta } from '@/config/tools';

const props = defineProps<{ tool: ToolMeta }>();
const router = useRouter();

function onClick() {
  router.push(props.tool.route);
}
</script>

<style scoped>
.tool-card {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  width: 100%;
  text-align: left;
  padding: 18px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-lg);
  box-shadow: var(--xuya-shadow);
  transition: transform 0.14s ease, box-shadow 0.14s ease, border-color 0.14s ease;
}
.tool-card:hover {
  transform: translateY(-2px);
  border-color: var(--xuya-accent);
  box-shadow: var(--xuya-shadow-hover);
}
.tool-card:hover .card-arrow {
  transform: translateX(2px);
  color: var(--xuya-accent);
}

.card-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 42px;
  height: 42px;
  flex-shrink: 0;
  border-radius: var(--xuya-radius);
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent);
}

.card-body {
  flex: 1;
  min-width: 0;
}
.card-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--xuya-text);
  margin-bottom: 4px;
}
.card-desc {
  font-size: 12.5px;
  color: var(--xuya-text-secondary);
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.card-arrow {
  color: var(--xuya-text-tertiary);
  flex-shrink: 0;
  margin-top: 2px;
  transition: transform 0.14s, color 0.14s;
}
</style>
