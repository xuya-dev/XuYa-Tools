<script setup lang="ts">
import { computed } from 'vue';
import { iconSvg, brandColor } from '@/config/providerPresets';

const props = defineProps<{
  iconKey: string;
  size?: number;
  colorSvg?: string;
}>();

const svg = computed(() => props.colorSvg ?? iconSvg(props.iconKey) ?? '');
const color = computed(() => brandColor(props.iconKey));
const size = computed(() => props.size ?? 16);
</script>

<template>
  <span
    v-if="svg"
    class="brand-icon"
    :style="{
      color: props.colorSvg ? undefined : color,
      width: `${size}px`,
      height: `${size}px`,
    }"
    v-html="svg"
  ></span>
  <span
    v-else
    class="brand-icon fallback"
    :style="{
      width: `${size}px`,
      height: `${size}px`,
      fontSize: `${size * 0.6}px`,
    }"
  >
    {{ iconKey.charAt(0).toUpperCase() }}
  </span>
</template>

<style scoped>
.brand-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.brand-icon :deep(svg) {
  width: 100%;
  height: 100%;
}

.fallback {
  font-weight: 600;
  line-height: 1;
}
</style>
