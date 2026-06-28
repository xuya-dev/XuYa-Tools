import { ref, onMounted, onUnmounted } from 'vue';

/**
 * 封装 Tauri 窗口控制。
 * 在非 Tauri 环境 (如纯浏览器 `npm run dev` 预览) 下安全降级为空操作。
 */

interface TauriWindow {
  minimize(): Promise<void>;
  toggleMaximize(): Promise<void>;
  close(): Promise<void>;
  startDragging(): Promise<void>;
  isMaximized(): Promise<boolean>;
  onResized(cb: () => void): Promise<() => void>;
}

let win: TauriWindow | null = null;
let cached = false;

async function getWindow(): Promise<TauriWindow | null> {
  if (cached) return win;
  cached = true;
  try {
    // 动态导入,避免非 Tauri 环境下报错
    const mod = await import('@tauri-apps/api/window');
    win = (mod.getCurrentWindow?.() ?? (mod as unknown as { getCurrentWindow: () => TauriWindow }).getCurrentWindow()) as TauriWindow;
  } catch {
    win = null;
  }
  return win;
}

export function useWindow() {
  const isMaximized = ref(false);
  let unlisten: (() => void) | null = null;

  onMounted(async () => {
    const w = await getWindow();
    if (!w) return;
    try {
      isMaximized.value = await w.isMaximized();
      unlisten = await w.onResized(async () => {
        try {
          isMaximized.value = await w.isMaximized();
        } catch {
          /* ignore */
        }
      });
    } catch {
      /* ignore */
    }
  });

  onUnmounted(() => {
    unlisten?.();
  });

  async function minimize() {
    const w = await getWindow();
    if (w) {
      try {
        await w.minimize();
      } catch {
        /* ignore */
      }
    }
  }

  async function toggleMaximize() {
    const w = await getWindow();
    if (w) {
      try {
        await w.toggleMaximize();
        isMaximized.value = await w.isMaximized();
      } catch {
        /* ignore */
      }
    }
  }

  async function close() {
    const w = await getWindow();
    if (w) {
      try {
        await w.close();
      } catch {
        /* ignore */
      }
    }
  }

  async function startDrag() {
    const w = await getWindow();
    if (w) {
      try {
        await w.startDragging();
      } catch {
        /* ignore */
      }
    }
  }

  /** 是否运行在 Tauri 桌面环境中 */
  const isTauri = ref(false);
  getWindow().then((w) => {
    isTauri.value = !!w;
  });

  return { isMaximized, isTauri, minimize, toggleMaximize, close, startDrag };
}
