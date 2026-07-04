<template>
  <ToolShell
    title="本地反代 / 统计"
    :icon="Network"
    description="启动本地代理接管 Claude Code / Codex CLI,实时查看请求数据、Token 用量与费用。"
  >
    <template #actions>
      <BaseButton variant="ghost" @click="onRefreshAll">
        <RefreshCw :size="13" :class="{ spin: proxyBusy }" />
        刷新
      </BaseButton>
    </template>

    <div class="proxy-dashboard">
      <!-- 反代控制 -->
      <ProxyPanel />

      <!-- 统计 (仅在反代开启后才有数据) -->
      <div id="stats" class="stats-anchor"></div>
      <UsagePanel />
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { Network, RefreshCw } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import ProxyPanel from '@/components/cli/ProxyPanel.vue';
import UsagePanel from '@/components/cli/UsagePanel.vue';
import { useCliConfig } from '@/composables/useCliConfig';

const { proxyBusy, refreshProxyStatus, refreshUsageSummary, refreshRequestLogs, logsPage } =
  useCliConfig();

async function onRefreshAll() {
  await Promise.all([
    refreshProxyStatus(),
    refreshUsageSummary(),
    refreshRequestLogs(logsPage.value),
  ]);
}
</script>

<style scoped>
.proxy-dashboard {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.stats-anchor {
  scroll-margin-top: 80px;
}
:deep(.spin) {
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
