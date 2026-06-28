import { ref } from 'vue';

export type ToastType = 'success' | 'error' | 'info';

interface ToastItem {
  id: number;
  message: string;
  type: ToastType;
}

const toasts = ref<ToastItem[]>([]);
let seq = 0;

function push(message: string, type: ToastType = 'success', duration = 2000) {
  const id = ++seq;
  toasts.value.push({ id, message, type });
  setTimeout(() => {
    remove(id);
  }, duration);
}

function remove(id: number) {
  const idx = toasts.value.findIndex((t) => t.id === id);
  if (idx !== -1) toasts.value.splice(idx, 1);
}

export function useToast() {
  return {
    toasts,
    success: (m: string, d?: number) => push(m, 'success', d),
    error: (m: string, d?: number) => push(m, 'error', d ?? 3000),
    info: (m: string, d?: number) => push(m, 'info', d),
    remove,
  };
}
