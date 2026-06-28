<template>
  <input
    class="base-input"
    :value="modelValue"
    :type="type"
    :placeholder="placeholder"
    :spellcheck="false"
    @input="onInput"
  />
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue?: string;
    placeholder?: string;
    type?: 'text' | 'number' | 'password';
  }>(),
  { modelValue: '', placeholder: '', type: 'text' }
);

const emit = defineEmits<{ (e: 'update:modelValue', v: string): void }>();
function onInput(e: Event) {
  emit('update:modelValue', (e.target as HTMLInputElement).value);
}
</script>

<style scoped>
.base-input {
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  font-family: inherit;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  transition: border-color var(--xuya-duration-fast) var(--xuya-ease),
    box-shadow var(--xuya-duration-fast) var(--xuya-ease),
    background var(--xuya-duration-fast) var(--xuya-ease);
}
.base-input:hover:not(:focus) {
  border-color: var(--xuya-border-strong);
}
.base-input::placeholder {
  color: var(--xuya-text-tertiary);
}
.base-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
  background: var(--xuya-bg-elevated);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
</style>
