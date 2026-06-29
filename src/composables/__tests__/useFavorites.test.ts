import { beforeEach, describe, expect, it } from 'vitest';
import { nextTick } from 'vue';
import { useFavorites } from '../useFavorites';

describe('useFavorites', () => {
  beforeEach(() => {
    localStorage.clear();
    // 重置内部模块级 ref 为默认值
    const { favorites } = useFavorites();
    favorites.value = ['json', 'regex', 'timestamp'];
  });

  it('默认收藏为 json/regex/timestamp', () => {
    const { favorites } = useFavorites();
    expect(favorites.value).toEqual(['json', 'regex', 'timestamp']);
  });

  it('toggleFavorite 可切换收藏状态', async () => {
    const { toggleFavorite, isFavorite, favorites } = useFavorites();
    toggleFavorite('jwt');
    await nextTick();
    expect(isFavorite('jwt')).toBe(true);
    expect(favorites.value).toContain('jwt');

    toggleFavorite('jwt');
    await nextTick();
    expect(isFavorite('jwt')).toBe(false);
    expect(favorites.value).not.toContain('jwt');
  });

  it('addFavorite 重复添加不会产生副本', async () => {
    const { addFavorite, favorites } = useFavorites();
    addFavorite('json');
    await nextTick();
    addFavorite('json');
    await nextTick();
    const count = favorites.value.filter((id) => id === 'json').length;
    expect(count).toBe(1);
  });

  it('removeFavorite 移除指定项', async () => {
    const { removeFavorite, isFavorite } = useFavorites();
    removeFavorite('regex');
    await nextTick();
    expect(isFavorite('regex')).toBe(false);
  });

  it('收藏列表持久化到 localStorage', async () => {
    const { toggleFavorite } = useFavorites();
    toggleFavorite('hashgen');
    await nextTick();
    const raw = localStorage.getItem('xuya_favorites');
    expect(raw).not.toBeNull();
    const parsed = JSON.parse(raw!);
    expect(parsed).toContain('hashgen');
  });
});
