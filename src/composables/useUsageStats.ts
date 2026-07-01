import { ref, watch } from 'vue';
import { tools, type ToolMeta } from '@/config/tools';

const STORAGE_KEY = 'xuya_usage_stats';

interface UsageStats {
  [toolId: string]: number;
}

const stats = ref<UsageStats>(load());

function load(): UsageStats {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw) as UsageStats;
  } catch {
    /* ignore */
  }
  return {};
}

watch(
  stats,
  (val) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(val));
    } catch {
      /* ignore */
    }
  },
  { deep: true },
);

export function useUsageStats() {
  function recordVisit(toolId: string) {
    stats.value[toolId] = (stats.value[toolId] ?? 0) + 1;
  }

  function getVisits(toolId: string): number {
    return stats.value[toolId] ?? 0;
  }

  function getTopTools(limit: number): ToolMeta[] {
    return [...tools]
      .map((t) => ({ ...t, _count: stats.value[t.id] ?? 0 }))
      .sort((a, b) => b._count - a._count)
      .slice(0, limit);
  }

  function clearStats() {
    stats.value = {};
  }

  return {
    stats,
    recordVisit,
    getVisits,
    getTopTools,
    clearStats,
  };
}
