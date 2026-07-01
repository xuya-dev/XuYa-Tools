import { ref, watch } from 'vue';
import type { ToolCategory } from '@/config/tools';

const STORAGE_KEY = 'xuya_collapsed_categories';

function load(): Set<ToolCategory> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return new Set(JSON.parse(raw) as ToolCategory[]);
  } catch {
    /* ignore */
  }
  return new Set();
}

const collapsed = ref<Set<ToolCategory>>(load());

watch(
  collapsed,
  (val) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify([...val]));
    } catch {
      /* ignore */
    }
  },
  { deep: true },
);

export function useCollapsedCategories() {
  function isCollapsed(cat: ToolCategory): boolean {
    return collapsed.value.has(cat);
  }

  function toggle(cat: ToolCategory) {
    const next = new Set(collapsed.value);
    if (next.has(cat)) next.delete(cat);
    else next.add(cat);
    collapsed.value = next;
  }

  return { collapsed, isCollapsed, toggle };
}
