<template>
  <ToolShell
    title="CLI 配置 / 反代"
    description="管理 Claude Code / Codex CLI 供应商配置,本地反代与请求统计。"
    :icon="Terminal"
  >
    <template #actions>
      <BaseButton variant="ghost" :disabled="loading" @click="refreshAll">
        <RefreshCw :size="13" :class="{ spin: loading }" />
        {{ loading ? '刷新中...' : '刷新' }}
      </BaseButton>
    </template>

    <CliPanel />
  </ToolShell>
</template>

<script setup lang="ts">
import { Terminal, RefreshCw } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import CliPanel from '@/components/cli/CliPanel.vue';
import { useCliConfig } from '@/composables/useCliConfig';

// 单例 store: 与 CliPanel 内部共享同一份状态
const { loading, refreshAll } = useCliConfig();
</script>

<style scoped>
:deep(.spin) {
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
