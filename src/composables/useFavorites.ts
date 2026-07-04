import { ref, watch } from 'vue';

/**
 * 工具收藏:常用工具置顶显示。
 * 收藏列表持久化到 localStorage。
 */

const STORAGE_KEY = 'xuya_favorites';
const favorites = ref<string[]>(load());

function load(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw) as string[];
  } catch {
    /* ignore */
  }
  // 默认收藏:JSON、正则、时间戳 (最高频)
  return ['json', 'regex', 'timestamp'];
}

watch(
  favorites,
  (val) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(val));
    } catch {
      /* ignore */
    }
  },
  { deep: true },
);

export function useFavorites() {
  function isFavorite(id: string): boolean {
    return favorites.value.includes(id);
  }

  function toggleFavorite(id: string) {
    if (favorites.value.includes(id)) {
      favorites.value = favorites.value.filter((f) => f !== id);
    } else {
      favorites.value = [...favorites.value, id];
    }
  }

  function addFavorite(id: string) {
    if (!favorites.value.includes(id)) {
      favorites.value = [...favorites.value, id];
    }
  }

  function removeFavorite(id: string) {
    favorites.value = favorites.value.filter((f) => f !== id);
  }

  return {
    favorites,
    isFavorite,
    toggleFavorite,
    addFavorite,
    removeFavorite,
  };
}
