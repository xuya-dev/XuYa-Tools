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
  transition: background var(--xuya-duration-fast) var(--xuya-ease),
    border-color var(--xuya-duration-fast) var(--xuya-ease),
    transform var(--xuya-duration-fast) var(--xuya-ease),
    box-shadow var(--xuya-duration-fast) var(--xuya-ease),
    opacity var(--xuya-duration-fast) var(--xuya-ease);
  white-space: nowrap;
  position: relative;
}
.base-btn:hover:not(:disabled) {
  background: var(--xuya-bg-elevated);
  border-color: var(--xuya-border-strong);
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
  box-shadow: 0 2px 8px var(--xuya-accent-glow), inset 0 1px 0 rgba(255, 255, 255, 0.15);
}
.variant-primary:hover:not(:disabled) {
  background: var(--xuya-accent-gradient-hover);
  box-shadow: 0 4px 14px var(--xuya-accent-glow), inset 0 1px 0 rgba(255, 255, 255, 0.2);
  transform: translateY(-1px);
}
.variant-primary:active:not(:disabled) {
  transform: translateY(0) scale(0.98);
}

.variant-ghost {
  background: transparent;
  border-color: var(--xuya-border);
  color: var(--xuya-text-secondary);
}
.variant-ghost:hover:not(:disabled) {
  background: var(--xuya-input-bg);
  border-color: var(--xuya-border-strong);
  color: var(--xuya-text);
}

.variant-danger {
  background: var(--xuya-danger);
  color: #fff;
  border: none;
  box-shadow: 0 2px 8px rgba(239, 68, 68, 0.3);
}
.variant-danger:hover:not(:disabled) {
  filter: brightness(1.08);
  box-shadow: 0 4px 14px rgba(239, 68, 68, 0.4);
  transform: translateY(-1px);
}
</style>
