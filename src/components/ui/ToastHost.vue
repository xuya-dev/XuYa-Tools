<template>
  <div class="toast-host">
    <TransitionGroup name="toast">
      <div
        v-for="t in toasts"
        :key="t.id"
        class="toast"
        :class="`type-${t.type}`"
        @click="remove(t.id)"
      >
        <CheckCircle2 v-if="t.type === 'success'" :size="15" />
        <XCircle v-else-if="t.type === 'error'" :size="15" />
        <Info v-else :size="15" />
        <span>{{ t.message }}</span>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import { CheckCircle2, XCircle, Info } from '@lucide/vue';
import { useToast } from '@/composables/useToast';

const { toasts, remove } = useToast();
</script>

<style scoped>
.toast-host {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column-reverse;
  gap: 8px;
  z-index: 9999;
  pointer-events: none;
}

.toast {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 9px 16px;
  border-radius: var(--xuya-radius);
  font-size: 13px;
  color: var(--xuya-on-accent);
  box-shadow: var(--xuya-shadow-hover);
  pointer-events: auto;
  cursor: pointer;
  backdrop-filter: blur(6px);
}
.type-success {
  background: var(--xuya-success);
}
.type-error {
  background: var(--xuya-danger);
}
.type-info {
  background: var(--xuya-accent);
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.25s ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(12px);
}
</style>
