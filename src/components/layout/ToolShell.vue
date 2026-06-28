<template>
  <div class="tool-shell">
    <header class="tool-header">
      <button class="back-btn" @click="goBack">
        <ArrowLeft :size="16" />
        <span>返回</span>
      </button>
      <div class="tool-title">
        <component :is="icon" :size="18" class="tool-icon" />
        <h1>{{ title }}</h1>
      </div>
      <p v-if="description" class="tool-desc">{{ description }}</p>
    </header>

    <div class="tool-body">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue';
import { useRouter } from 'vue-router';
import { ArrowLeft } from '@lucide/vue';

defineProps<{
  title: string;
  description?: string;
  icon: Component;
}>();

const router = useRouter();
function goBack() {
  router.push('/');
}
</script>

<style scoped>
.tool-shell {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px 32px 40px;
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.tool-header {
  flex-shrink: 0;
  margin-bottom: 18px;
}

.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px 4px 4px;
  margin-bottom: 14px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-radius: var(--xuya-radius-sm);
  transition: color 0.12s, background 0.12s;
}
.back-btn:hover {
  color: var(--xuya-text);
  background: var(--xuya-input-bg);
}

.tool-title {
  display: flex;
  align-items: center;
  gap: 10px;
}
.tool-title h1 {
  font-size: 20px;
  font-weight: 700;
  color: var(--xuya-text);
}
.tool-icon {
  color: var(--xuya-accent);
  flex-shrink: 0;
}

.tool-desc {
  margin-top: 6px;
  margin-left: 28px;
  font-size: 13px;
  color: var(--xuya-text-secondary);
}

.tool-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
</style>
