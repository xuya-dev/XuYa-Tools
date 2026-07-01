import { describe, expect, it } from 'vitest';
import { encodeQR } from '../qrcode';

describe('encodeQR', () => {
  it('空内容使用占位符不报错', () => {
    const m = encodeQR('', 'M');
    expect(m.size).toBeGreaterThan(0);
  });

  it('短文本生成 v1 (21x21) 矩阵', () => {
    const m = encodeQR('hi', 'M');
    expect(m.size).toBe(21);
  });

  it('矩阵只返回 boolean', () => {
    const m = encodeQR('hello', 'L');
    for (let y = 0; y < m.size; y++) {
      for (let x = 0; x < m.size; x++) {
        expect(typeof m.get(x, y)).toBe('boolean');
      }
    }
  });

  it('三个定位图案在角落', () => {
    const m = encodeQR('qr', 'M');
    const s = m.size;
    expect(m.get(3, 3)).toBe(true);
    expect(m.get(s - 4, 3)).toBe(true);
    expect(m.get(3, s - 4)).toBe(true);
  });

  it('长内容自动升级版本', () => {
    const short = encodeQR('a', 'M').size;
    const long = encodeQR('a'.repeat(100), 'M').size;
    expect(long).toBeGreaterThan(short);
  });

  it('所有纠错等级都能生成', () => {
    for (const lvl of ['L', 'M', 'Q', 'H'] as const) {
      const m = encodeQR('xuya', lvl);
      expect(m.size).toBeGreaterThan(0);
    }
  });

  it('中文内容能正常生成', () => {
    const m = encodeQR('许亚工具箱', 'M');
    expect(m.size).toBeGreaterThan(0);
  });
});
