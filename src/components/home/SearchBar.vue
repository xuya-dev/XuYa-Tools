<template>
  <div class="search-bar">
    <Search :size="16" class="search-icon" />
    <input
      :value="modelValue"
      class="search-input"
      type="text"
      placeholder="搜索工具... (Ctrl+K)"
      spellcheck="false"
      @input="onInput"
    />
    <button v-if="modelValue" class="clear-btn" @click="clear">
      <X :size="14" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { Search, X } from '@lucide/vue';

defineProps<{ modelValue: string }>();
const emit = defineEmits<{ (e: 'update:modelValue', v: string): void }>();

function onInput(e: Event) {
  emit('update:modelValue', (e.target as HTMLInputElement).value);
}
function clear() {
  emit('update:modelValue', '');
}
</script>

<style scoped>
.search-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 16px;
  height: 46px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  transition: border-color 0.12s, box-shadow 0.12s;
}
.search-bar:focus-within {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.search-icon {
  color: var(--xuya-text-tertiary);
  flex-shrink: 0;
}
.search-input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--xuya-text);
  font-size: 14px;
  outline: none;
}
.search-input::placeholder {
  color: var(--xuya-text-tertiary);
}
.clear-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: none;
  background: transparent;
  color: var(--xuya-text-tertiary);
  border-radius: 50%;
  transition: background 0.12s, color 0.12s;
}
.clear-btn:hover {
  background: var(--xuya-border);
  color: var(--xuya-text);
}
</style>
