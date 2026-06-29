<template>
  <AppLayout />
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import AppLayout from '@/components/layout/AppLayout.vue';

onMounted(async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().setIcon('/Logo.png');
  } catch {
    // 非 Tauri 环境下忽略
  }
});
</script>
