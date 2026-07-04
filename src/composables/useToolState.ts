import { ref, watch, type Ref } from 'vue';

/**
 * 工具状态持久化:把任意可序列化状态按工具 id 存到 localStorage,
 * 切换工具或重启应用后仍保留输入内容。
 *
 * 用法:
 *   const input = useToolState('json', 'input', '{\n  "a": 1\n}');
 *   // input 是一个普通 ref,读写都自动持久化
 */

const PREFIX = 'xuya_state_';

const cache = new Map<string, Ref<unknown>>();

export function useToolState<T>(toolId: string, key: string, defaultValue: T): Ref<T> {
  const storageKey = `${PREFIX}${toolId}_${key}`;

  // 同一 key 复用同一 ref (跨组件共享)
  if (cache.has(storageKey)) {
    return cache.get(storageKey) as Ref<T>;
  }

  let initial = defaultValue;
  try {
    const raw = localStorage.getItem(storageKey);
    if (raw !== null) {
      initial = JSON.parse(raw) as T;
    }
  } catch {
    /* 解析失败用默认值 */
  }

  const state = ref(initial) as Ref<T>;

  watch(
    state,
    (val) => {
      try {
        localStorage.setItem(storageKey, JSON.stringify(val));
      } catch {
        /* 存储满或隐私模式,静默忽略 */
      }
    },
    { deep: true },
  );

  cache.set(storageKey, state as Ref<unknown>);
  return state;
}

/** 读取某个工具的全部持久化状态 (调试/导出用) */
export function getToolStateAll(toolId: string): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  const prefix = `${PREFIX}${toolId}_`;
  for (let i = 0; i < localStorage.length; i++) {
    const k = localStorage.key(i);
    if (k && k.startsWith(prefix)) {
      const subKey = k.slice(prefix.length);
      try {
        out[subKey] = JSON.parse(localStorage.getItem(k) ?? 'null');
      } catch {
        /* ignore */
      }
    }
  }
  return out;
}

/** 清除某个工具的持久化状态 */
export function clearToolState(toolId: string) {
  const prefix = `${PREFIX}${toolId}_`;
  const toRemove: string[] = [];
  for (let i = 0; i < localStorage.length; i++) {
    const k = localStorage.key(i);
    if (k && k.startsWith(prefix)) toRemove.push(k);
  }
  toRemove.forEach((k) => {
    localStorage.removeItem(k);
    cache.delete(k);
  });
}

/** 清除全部工具状态 */
export function clearAllToolState() {
  const toRemove: string[] = [];
  for (let i = 0; i < localStorage.length; i++) {
    const k = localStorage.key(i);
    if (k && k.startsWith(PREFIX)) toRemove.push(k);
  }
  toRemove.forEach((k) => {
    localStorage.removeItem(k);
    cache.delete(k);
  });
}
