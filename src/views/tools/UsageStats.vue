<template>
  <ToolShell title="请求统计" :icon="BarChart3" description="查看反代记录的请求数据:成功率、延迟、Token 用量、费用与明细日志。">
    <template #actions>
      <BaseButton variant="ghost" @click="onRefresh">
        <RefreshCw :size="13" />
        刷新
      </BaseButton>
    </template>
    <div class="usage-manager-body">
      <UsagePanel />
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { BarChart3, RefreshCw } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import UsagePanel from '@/components/cli/UsagePanel.vue';
import { useCliConfig } from '@/composables/useCliConfig';

const { refreshUsageSummary, refreshRequestLogs, logsPage } = useCliConfig();

async function onRefresh() {
  await Promise.all([refreshUsageSummary(), refreshRequestLogs(logsPage.value)]);
}
</script>

<style scoped>
.usage-manager-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
</style>
