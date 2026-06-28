<template>
  <div class="app-shell">
    <AppTitleBar />
    <main class="app-main">
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>
    <ToastHost />
  </div>
</template>

<script setup lang="ts">
import AppTitleBar from './AppTitleBar.vue';
import ToastHost from '@/components/ui/ToastHost.vue';
</script>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--xuya-bg);
  overflow: hidden;
}

.app-main {
  flex: 1;
  min-height: 0;
  overflow: auto;
}
</style>
