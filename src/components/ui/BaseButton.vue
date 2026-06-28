<template>
  <button
    class="base-btn"
    :class="[`variant-${variant}`, { 'is-block': block }]"
    :type="type"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    type?: 'button' | 'submit';
    disabled?: boolean;
    block?: boolean;
  }>(),
  { variant: 'secondary', type: 'button', disabled: false, block: false }
);
</script>

<style scoped>
.base-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 7px 14px;
  font-size: 13px;
  font-weight: 500;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  transition: background 0.12s, border-color 0.12s, transform 0.08s, opacity 0.12s;
  white-space: nowrap;
}
.base-btn:hover:not(:disabled) {
  background: var(--xuya-border-light);
}
.base-btn:active:not(:disabled) {
  transform: scale(0.97);
}
.base-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.is-block {
  width: 100%;
}

.variant-primary {
  background: var(--xuya-accent-gradient);
  color: #fff;
  border: none;
  box-shadow: 0 2px 6px var(--xuya-accent-ring);
}
.variant-primary:hover:not(:disabled) {
  filter: brightness(1.06);
  background: var(--xuya-accent-gradient);
}

.variant-ghost {
  background: transparent;
  border-color: var(--xuya-border);
  color: var(--xuya-text-secondary);
}
.variant-ghost:hover:not(:disabled) {
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
}

.variant-danger {
  background: var(--xuya-danger);
  color: #fff;
  border: none;
}
.variant-danger:hover:not(:disabled) {
  filter: brightness(1.08);
}
</style>
