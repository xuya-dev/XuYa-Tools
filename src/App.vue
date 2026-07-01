<template>
  <Suspense>
    <WidgetView v-if="isWidget" />
    <AppLayout v-else />
  </Suspense>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import AppLayout from '@/components/layout/AppLayout.vue';
import WidgetView from '@/views/WidgetView.vue';

const route = useRoute();
const isWidget = computed(() => route.path === '/widget');
</script>
