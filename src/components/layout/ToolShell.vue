<template>
  <div class="tool-shell">
    <header class="tool-header">
      <div class="tool-title">
        <component :is="icon" :size="20" class="tool-icon" />
        <div class="tool-title-text">
          <h1>{{ title }}</h1>
          <p v-if="description">{{ description }}</p>
        </div>
        <div v-if="$slots.actions" class="tool-actions">
          <slot name="actions" />
        </div>
      </div>
    </header>

    <div class="tool-body">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue';

defineProps<{
  title: string;
  description?: string;
  icon: Component;
}>();
</script>

<style scoped>
.tool-shell {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  padding: 20px 28px 28px;
}

.tool-header {
  flex-shrink: 0;
  padding-bottom: 16px;
  margin-bottom: 18px;
  border-bottom: 1px solid var(--xuya-border);
}

.tool-title {
  display: flex;
  align-items: center;
  gap: 12px;
}
.tool-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  border-radius: var(--xuya-radius);
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent);
}
.tool-title-text {
  flex: 1;
  min-width: 0;
}
.tool-title h1 {
  font-size: 18px;
  font-weight: 700;
  color: var(--xuya-text);
  line-height: 1.3;
}
.tool-title p {
  margin-top: 3px;
  font-size: 12.5px;
  color: var(--xuya-text-secondary);
}
.tool-actions {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.tool-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
</style>
