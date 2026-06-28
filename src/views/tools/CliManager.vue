<template>
  <div class="cli-manager-wrap">
    <CliPanel />
  </div>
</template>

<script setup lang="ts">
import CliPanel from '@/components/cli/CliPanel.vue';
</script>

<style scoped>
.cli-manager-wrap {
  height: 100%;
  padding: 20px 28px 32px;
  max-width: 1280px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.cli-manager-wrap :deep(.cli-panel) {
  flex: 1;
  min-height: 0;
}
</style>
