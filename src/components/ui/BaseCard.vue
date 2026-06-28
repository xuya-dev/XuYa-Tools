<template>
  <div class="base-card" :class="{ 'no-pad': noPad }">
    <slot />
  </div>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ noPad?: boolean }>(), { noPad: false });
</script>

<style scoped>
.base-card {
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-lg);
  padding: 20px;
  box-shadow: var(--xuya-shadow);
}
.no-pad {
  padding: 0;
}
</style>
