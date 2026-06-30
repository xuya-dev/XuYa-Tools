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
  padding: 22px 30px 30px;
  animation: tool-enter var(--xuya-duration) var(--xuya-ease-out);
}
@keyframes tool-enter {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.tool-header {
  flex-shrink: 0;
  padding-bottom: 18px;
  margin-bottom: 20px;
  border-bottom: 1px solid var(--xuya-border);
}

.tool-title {
  display: flex;
  align-items: center;
  gap: 14px;
}
.tool-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 42px;
  height: 42px;
  flex-shrink: 0;
  border-radius: var(--xuya-radius);
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent);
  box-shadow: inset 0 0 0 1px var(--xuya-accent-soft-strong);
}
.tool-title-text {
  flex: 1;
  min-width: 0;
}
.tool-title h1 {
  font-size: 19px;
  font-weight: 700;
  color: var(--xuya-text);
  line-height: 1.3;
  letter-spacing: -0.2px;
}
.tool-title p {
  margin-top: 3px;
  font-size: 12.5px;
  color: var(--xuya-text-secondary);
  line-height: 1.45;
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
  overflow: auto;
  display: flex;
  flex-direction: column;
}
</style>
