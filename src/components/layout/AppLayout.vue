<template>
  <div class="app-shell">
    <ToolMasterDetail />
    <ToastHost />
  </div>
</template>

<script setup lang="ts">
import ToolMasterDetail from '@/components/layout/ToolMasterDetail.vue';
import ToastHost from '@/components/ui/ToastHost.vue';
</script>

<style scoped>
.app-shell {
  display: flex;
  height: 100%;
  background: var(--xuya-bg);
  overflow: hidden;
}
</style>
