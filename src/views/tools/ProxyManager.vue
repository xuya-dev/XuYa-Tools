<template>
  <ToolShell title="本地反代" :icon="Network" description="启动本地代理,接管 Claude Code / Codex CLI,将请求导向配置的上游供应商。">
    <template #actions>
      <BaseButton variant="ghost" :disabled="proxyBusy" @click="refreshProxyStatus">
        <RefreshCw :size="13" :class="{ spin: proxyBusy }" />
        刷新
      </BaseButton>
    </template>
    <div class="proxy-manager-body">
      <ProxyPanel />
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { Network, RefreshCw } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import ProxyPanel from '@/components/cli/ProxyPanel.vue';
import { useCliConfig } from '@/composables/useCliConfig';

const { proxyBusy, refreshProxyStatus } = useCliConfig();
</script>

<style scoped>
.proxy-manager-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
:deep(.spin) { animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
