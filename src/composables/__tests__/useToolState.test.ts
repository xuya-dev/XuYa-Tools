import { beforeEach, describe, expect, it } from 'vitest';
import { nextTick } from 'vue';
import { useToolState, clearToolState, clearAllToolState, getToolStateAll } from '../useToolState';

describe('useToolState', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('未存过时返回默认值', () => {
    const v = useToolState('test-tool', 'input', '默认');
    expect(v.value).toBe('默认');
  });

  it('写入后立即持久化到 localStorage', async () => {
    const v = useToolState('test-tool', 'input', '');
    v.value = 'hello';
    await nextTick();
    const raw = localStorage.getItem('xuya_state_test-tool_input');
    expect(raw).toBe(JSON.stringify('hello'));
  });

  it('同一 (toolId, key) 复用同一 ref (跨调用共享)', async () => {
    const a = useToolState('shared-tool', 'v', 1);
    const b = useToolState('shared-tool', 'v', 999);
    a.value = 42;
    await nextTick();
    expect(b.value).toBe(42);
  });

  it('从 localStorage 恢复已持久化的值', () => {
    localStorage.setItem('xuya_state_restore_x', JSON.stringify({ nested: true }));
    const v = useToolState('restore', 'x', null);
    expect(v.value).toEqual({ nested: true });
  });

  it('损坏的 JSON 回退为默认值', () => {
    localStorage.setItem('xuya_state_corrupt_x', 'not-a-json');
    const v = useToolState('corrupt', 'x', 'fallback');
    expect(v.value).toBe('fallback');
  });

  it('clearToolState 仅清指定工具', async () => {
    const a = useToolState('tool-a', 'k', 'a');
    const b = useToolState('tool-b', 'k', 'b');
    a.value = 'A';
    b.value = 'B';
    await nextTick();
    clearToolState('tool-a');
    expect(localStorage.getItem('xuya_state_tool-a_k')).toBeNull();
    expect(localStorage.getItem('xuya_state_tool-b_k')).not.toBeNull();
  });

  it('clearAllToolState 清除全部工具状态', async () => {
    const a = useToolState('tool-a', 'k', 'a');
    const b = useToolState('tool-b', 'k', 'b');
    a.value = 'A';
    b.value = 'B';
    await nextTick();
    clearAllToolState();
    expect(localStorage.getItem('xuya_state_tool-a_k')).toBeNull();
    expect(localStorage.getItem('xuya_state_tool-b_k')).toBeNull();
  });

  it('getToolStateAll 返回该工具的全部状态', async () => {
    const a = useToolState('all-tool', 'a', 1);
    const b = useToolState('all-tool', 'b', 2);
    a.value = 10;
    b.value = 20;
    await nextTick();
    const all = getToolStateAll('all-tool');
    expect(all).toEqual({ a: 10, b: 20 });
  });
});
