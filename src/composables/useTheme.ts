import { ref, watch, onMounted, onUnmounted } from 'vue';

export type ThemeMode = 'light' | 'dark' | 'system';

const STORAGE_KEY = 'xuya_theme_mode';

const mode = ref<ThemeMode>('dark');
const systemDark = ref(true);
let media: MediaQueryList | null = null;

function resolveDark(m: ThemeMode): boolean {
  if (m === 'dark') return true;
  if (m === 'light') return false;
  return systemDark.value;
}

function applyToDom(isDark: boolean) {
  const root = document.documentElement;
  root.classList.toggle('dark-theme', isDark);
}

/** 初始化主题 (由 main.ts 调用一次) */
export function initTheme() {
  // 读取已持久化的偏好,默认 system
  const saved = localStorage.getItem(STORAGE_KEY) as ThemeMode | null;
  mode.value = saved ?? 'system';

  media = window.matchMedia('(prefers-color-scheme: dark)');
  systemDark.value = media.matches;

  // 监听系统主题变化
  const onSys = (e: MediaQueryListEvent) => {
    systemDark.value = e.matches;
  };
  media.addEventListener('change', onSys);

  applyToDom(resolveDark(mode.value));
}

/** 供组件使用的 composable */
export function useTheme() {
  const isDark = ref(resolveDark(mode.value));

  // 同步 mode → isDark + DOM
  watch(
    mode,
    (m) => {
      isDark.value = resolveDark(m);
      applyToDom(isDark.value);
    },
    { immediate: true }
  );

  // 系统变化时,若当前为 system 则同步
  const onSys = () => {
    if (mode.value === 'system') {
      isDark.value = systemDark.value;
      applyToDom(isDark.value);
    }
  };

  onMounted(() => media?.addEventListener('change', onSys));
  onUnmounted(() => media?.removeEventListener('change', onSys));

  function setMode(m: ThemeMode) {
    mode.value = m;
    localStorage.setItem(STORAGE_KEY, m);
  }

  function cycleMode() {
    const order: ThemeMode[] = ['light', 'dark', 'system'];
    const idx = order.indexOf(mode.value);
    setMode(order[(idx + 1) % order.length] as ThemeMode);
  }

  return { mode, isDark, setMode, cycleMode };
}
