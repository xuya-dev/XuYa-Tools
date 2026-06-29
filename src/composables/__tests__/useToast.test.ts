import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useToast } from '../useToast';

describe('useToast', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    const { toasts } = useToast();
    toasts.value = [];
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('success 推入一条 success 类型 toast', () => {
    const { success, toasts } = useToast();
    success('操作成功');
    expect(toasts.value).toHaveLength(1);
    expect(toasts.value[0]).toMatchObject({ message: '操作成功', type: 'success' });
  });

  it('error 默认持续时间长于 success', () => {
    const { success, error } = useToast();
    const spy = vi.spyOn(globalThis, 'setTimeout');
    success('ok');
    error('boom');
    const successDuration = spy.mock.calls[0]?.[1];
    const errorDuration = spy.mock.calls[1]?.[1];
    expect(errorDuration).toBeGreaterThan(successDuration!);
  });

  it('duration 到期后自动移除', () => {
    const { success, toasts } = useToast();
    success('flash', 1000);
    expect(toasts.value).toHaveLength(1);
    vi.advanceTimersByTime(1000);
    expect(toasts.value).toHaveLength(0);
  });

  it('remove 主动移除指定 id', () => {
    const { success, remove, toasts } = useToast();
    success('x');
    const id = toasts.value[0]!.id;
    remove(id);
    expect(toasts.value).toHaveLength(0);
  });
});
