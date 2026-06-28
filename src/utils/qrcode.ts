/**
 * 最小化 QR Code 编码器 (byte 模式)。
 * 支持 1-40 版本、L/M/Q/H 纠错,输出 { size, get(x,y) } 矩阵。
 * 仅用于离线工具生成,实现紧凑无依赖。
 */

// ---- 安全数组访问辅助 (避免 noUncheckedIndexedAccess 干扰) ----
function atN(arr: ArrayLike<number> | Record<string, number>, i: number | string): number {
  return (arr as Record<number | string, number>)[i] ?? 0;
}
function atRow(arr: number[][], i: number): number[] {
  return arr[i] ?? [];
}

// ---- Galois 域 GF(256) ----
const EXP = new Uint8Array(512);
const LOG = new Uint8Array(256);
(() => {
  let x = 1;
  for (let i = 0; i < 255; i++) {
    EXP[i] = x;
    LOG[x] = i;
    x <<= 1;
    if (x & 0x100) x ^= 0x11d;
  }
  for (let i = 255; i < 512; i++) EXP[i] = EXP[i - 255]!;
})();
function gfMul(a: number, b: number): number {
  if (a === 0 || b === 0) return 0;
  return atN(EXP, atN(LOG, a) + atN(LOG, b));
}

// ---- Reed-Solomon ----
function rsGenPoly(degree: number): number[] {
  let poly: number[] = [1];
  for (let i = 0; i < degree; i++) {
    const np: number[] = new Array(poly.length + 1).fill(0);
    for (let j = 0; j < poly.length; j++) {
      np[j] = (np[j] ?? 0) ^ atN(poly, j);
      np[j + 1] = (np[j + 1] ?? 0) ^ gfMul(atN(poly, j), atN(EXP, i));
    }
    poly = np;
  }
  return poly;
}
function rsEncode(data: number[], ecLen: number): number[] {
  const gen = rsGenPoly(ecLen);
  const buf: number[] = data.concat(new Array<number>(ecLen).fill(0));
  for (let i = 0; i < data.length; i++) {
    const coef = atN(buf, i);
    if (coef === 0) continue;
    for (let j = 0; j < gen.length; j++) buf[i + j] = (buf[i + j] ?? 0) ^ gfMul(atN(gen, j), coef);
  }
  return buf.slice(data.length);
}

// ---- 纠错等级 ----
const EC_BITS: Record<string, number> = { L: 1, M: 0, Q: 3, H: 2 };

const CAPACITY: Record<string, number[]> = {
  L: [17, 32, 53, 78, 106, 134, 154, 192, 230, 271, 321, 367, 425, 458, 520, 586, 644, 718, 792, 858, 929, 1003, 1091, 1171, 1273, 1367, 1465, 1528, 1628, 1732, 1840, 1952, 2068, 2188, 2303, 2431, 2563, 2699, 2809, 2953],
  M: [14, 26, 42, 62, 84, 106, 122, 152, 180, 213, 251, 287, 331, 362, 412, 450, 504, 560, 624, 666, 711, 779, 857, 911, 997, 1059, 1125, 1190, 1264, 1370, 1452, 1538, 1628, 1722, 1809, 1911, 1989, 2099, 2213, 2331],
  Q: [11, 20, 32, 46, 60, 74, 86, 108, 130, 151, 177, 203, 241, 258, 292, 322, 364, 394, 442, 482, 509, 565, 611, 661, 715, 751, 805, 868, 908, 982, 1030, 1112, 1168, 1228, 1283, 1351, 1423, 1499, 1579, 1663],
  H: [7, 14, 24, 34, 44, 58, 64, 84, 98, 119, 137, 155, 177, 194, 220, 250, 280, 310, 338, 382, 403, 439, 461, 511, 535, 593, 625, 658, 698, 742, 790, 842, 898, 958, 983, 1051, 1093, 1139, 1219, 1273],
};

interface BlockLayout { total: number; ecPerBlock: number; g1Count: number; g1Data: number; g2Count: number; g2Data: number; }
// 每行: [g1Count, g1Data, g2Count, g2Data, ecPerBlock]
const BLOCKS: Record<string, number[][]> = {};
BLOCKS.M = [[1,16,0,0,10],[1,28,0,0,16],[1,44,0,0,26],[1,64,0,0,18],[1,86,0,0,24],[1,108,0,0,16],[2,68,0,0,18],[2,78,0,0,22],[2,97,0,0,22],[2,116,0,0,26],[2,68,2,69,30],[2,81,2,82,22],[2,92,2,93,24],[2,107,2,108,26],[2,115,3,116,26],[3,91,2,92,28],[3,101,2,102,28],[3,113,2,114,26],[3,125,2,126,28],[4,93,2,94,30],[4,107,2,108,30],[4,115,3,116,30],[5,115,1,116,30],[5,121,1,122,30],[5,127,1,128,30],[5,133,1,134,30],[4,121,5,122,30],[5,135,1,136,30],[6,135,1,136,30],[6,125,2,126,30],[6,151,1,152,30],[7,151,1,152,30],[6,147,4,148,30],[7,151,2,152,30],[7,139,4,140,30],[7,151,2,152,30],[8,143,4,144,30],[9,139,3,140,30],[10,141,2,142,30],[9,143,2,144,30]];
BLOCKS.L = [[1,19,0,0,7],[1,34,0,0,10],[1,55,0,0,15],[1,80,0,0,20],[1,108,0,0,26],[2,68,0,0,18],[2,78,0,0,20],[2,97,0,0,24],[2,116,0,0,30],[2,68,2,69,18],[2,68,2,69,20],[4,81,0,0,24],[2,92,2,93,26],[4,107,0,0,30],[3,115,1,116,22],[5,87,1,88,24],[5,98,1,99,24],[1,107,5,108,28],[5,120,1,121,30],[3,113,4,114,28],[4,125,4,126,28],[2,98,7,99,28],[4,108,5,109,30],[4,75,13,76,30],[6,88,7,89,30],[8,99,4,100,30],[8,96,5,97,30],[9,97,5,98,30],[5,107,13,108,30],[7,115,7,116,30],[13,115,3,116,30],[17,115,3,116,30],[17,115,4,116,30],[4,109,19,110,30],[20,105,4,106,30],[17,109,8,110,30],[20,109,7,110,30],[19,111,8,112,30],[21,111,8,112,30],[21,107,13,108,30]];
BLOCKS.Q = [[1,13,0,0,13],[1,22,0,0,22],[1,34,0,0,18],[2,40,0,0,26],[2,32,2,33,24],[4,43,0,0,18],[2,44,2,45,22],[4,39,2,40,20],[4,43,2,44,22],[4,45,2,46,26],[5,37,3,38,22],[5,41,3,42,24],[8,33,2,34,22],[9,39,2,40,24],[3,40,7,41,22],[3,46,8,47,22],[4,45,10,46,24],[6,45,7,46,24],[9,41,5,42,24],[8,45,6,46,24],[11,44,4,45,24],[8,42,9,43,22],[10,46,6,47,26],[10,46,6,47,26],[14,44,6,45,26],[14,43,9,44,26],[10,44,13,45,26],[11,44,12,45,26],[13,42,14,43,26],[15,46,10,47,28],[17,43,13,44,28],[19,44,9,45,28],[19,44,11,45,28],[20,43,12,44,28],[20,44,13,45,28],[22,41,15,42,28],[22,41,15,42,28],[20,39,19,40,28],[20,39,19,40,28],[23,39,16,40,28]];
BLOCKS.H = [[1,9,0,0,17],[1,16,0,0,28],[2,13,0,0,22],[4,9,0,0,16],[2,17,2,18,28],[4,24,0,0,22],[2,17,2,18,26],[4,22,2,23,26],[4,20,2,21,28],[5,30,2,31,24],[5,23,2,24,28],[5,20,4,21,22],[8,20,1,21,24],[9,19,4,20,22],[4,21,7,22,24],[6,18,8,19,22],[8,18,8,19,24],[3,24,10,25,22],[3,20,15,21,24],[5,19,15,20,24],[7,19,15,20,24],[8,17,18,18,22],[11,19,8,20,24],[11,17,12,18,24],[11,18,12,19,24],[11,17,13,18,24],[12,17,13,18,26],[13,17,13,18,26],[14,17,14,18,26],[15,18,14,19,28],[17,16,16,17,28],[17,17,16,18,28],[15,17,18,19,28],[15,17,18,19,28],[16,17,18,18,28],[17,17,19,18,28],[19,15,20,16,28],[21,15,20,16,28],[22,15,20,16,28],[22,14,21,15,28]];
function getBlockLayout(version: number, level: string): BlockLayout {
  const rows = (BLOCKS as Record<string, number[][]>)[level] ?? [];
  const row = atRow(rows, version - 1);
  return { g1Count: atN(row, 0), g1Data: atN(row, 1), g2Count: atN(row, 2), g2Data: atN(row, 3), ecPerBlock: atN(row, 4), total: atN(row, 0) * atN(row, 1) + atN(row, 2) * atN(row, 3) };
}

const ALIGN_POS: number[][] = [[],[6,18],[6,22],[6,26],[6,30],[6,34],[6,22,38],[6,24,42],[6,26,46],[6,28,50],[6,30,54],[6,32,58],[6,34,62],[6,26,46,66],[6,26,48,70],[6,26,50,74],[6,30,54,78],[6,30,56,82],[6,30,58,86],[6,30,60,90],[6,34,62,94],[6,28,50,72,98],[6,26,50,74,98],[6,30,54,78,102],[6,28,54,80,106],[6,32,58,84,110],[6,30,58,86,114],[6,34,62,90,118],[6,26,50,74,98,122],[6,30,54,78,102,126],[6,26,52,78,104,130],[6,30,56,82,108,134],[6,34,60,86,112,138],[6,30,58,86,114,142],[6,34,62,90,118,146],[6,30,54,78,102,126,150],[6,24,50,76,102,128,154],[6,28,54,80,106,132,158],[6,32,58,84,110,136,162],[6,26,54,82,110,138,166],[6,30,58,86,114,142,170]];

export interface QRMatrix { size: number; get(x: number, y: number): boolean; }

export function encodeQR(text: string, level: 'L' | 'M' | 'Q' | 'H' = 'M'): QRMatrix {
  const bytes = Array.from(new TextEncoder().encode(text));
  if (bytes.length === 0) throw new Error('内容为空');

  const cap = (CAPACITY as Record<string, number[]>)[level] ?? [];
  let version = 1;
  for (; version <= 40; version++) {
    const ccBits = version < 10 ? 8 : 16;
    const dataBits = 4 + ccBits + bytes.length * 8;
    if (dataBits <= atN(cap, version - 1) * 8) break;
  }
  if (version > 40) throw new Error('内容过长, 超过 QR 码最大容量');

  const size = 17 + version * 4;
  const layout = getBlockLayout(version, level);
  const ccBits = version < 10 ? 8 : 16;

  // 位流
  let bits = '0100';
  bits += bytes.length.toString(2).padStart(ccBits, '0');
  for (const b of bytes) bits += b.toString(2).padStart(8, '0');
  // 终止符(最多4位) + 对齐到字节
  const maxTerm = Math.max(0, Math.min(4, layout.total * 8 - bits.length));
  bits += '0'.repeat(maxTerm);
  while (bits.length % 8 !== 0) bits += '0';
  // 填充字节
  const pad = [0xEC, 0x11];
  let pi = 0;
  while (bits.length / 8 < layout.total) { bits += atN(pad, pi % 2).toString(2).padStart(8, '0'); pi++; }

  const dataCodewords: number[] = [];
  for (let i = 0; i < bits.length; i += 8) dataCodewords.push(parseInt(bits.slice(i, i + 8), 2));

  // 分块 + RS
  const blocks: { data: number[]; ec: number[] }[] = [];
  let offset = 0;
  for (let i = 0; i < layout.g1Count; i++) { const d = dataCodewords.slice(offset, offset + layout.g1Data); offset += layout.g1Data; blocks.push({ data: d, ec: rsEncode(d, layout.ecPerBlock) }); }
  for (let i = 0; i < layout.g2Count; i++) { const d = dataCodewords.slice(offset, offset + layout.g2Data); offset += layout.g2Data; blocks.push({ data: d, ec: rsEncode(d, layout.ecPerBlock) }); }

  // 交错
  const interleaved: number[] = [];
  const maxData = Math.max(layout.g1Data, layout.g2Data);
  for (let i = 0; i < maxData; i++) for (const b of blocks) if (i < b.data.length) interleaved.push(atN(b.data, i));
  for (let i = 0; i < layout.ecPerBlock; i++) for (const b of blocks) interleaved.push(atN(b.ec, i));

  let finalBits = interleaved.map((c) => c.toString(2).padStart(8, '0')).join('');
  const totalCodewords = layout.total + (layout.g1Count + layout.g2Count) * layout.ecPerBlock;
  while (finalBits.length < totalCodewords * 8) finalBits += '0';
  finalBits = finalBits.slice(0, totalCodewords * 8);

  // ---- 矩阵 ----
  const m = new Uint8Array(size * size);
  const reserved = new Uint8Array(size * size);
  const set = (x: number, y: number, v: number) => { if (x >= 0 && x < size && y >= 0 && y < size) { m[y * size + x] = v; reserved[y * size + x] = 1; } };

  // 定位图案
  const finder = (r: number, c: number) => {
    for (let dy = -1; dy <= 7; dy++) for (let dx = -1; dx <= 7; dx++) {
      const y = r + dy, x = c + dx; if (y < 0 || y >= size || x < 0 || x >= size) continue;
      const border = dx === 0 || dx === 6 || dy === 0 || dy === 6;
      const inner = dx >= 2 && dx <= 4 && dy >= 2 && dy <= 4;
      set(x, y, (border || inner) ? 1 : 0);
    }
  };
  finder(0, 0); finder(0, size - 7); finder(size - 7, 0);
  // 时序
  for (let i = 8; i < size - 8; i++) { set(i, 6, i % 2 === 0 ? 1 : 0); set(6, i, i % 2 === 0 ? 1 : 0); }
  // 对齐
  const aligns = atRow(ALIGN_POS, version);
  for (const r of aligns) for (const c of aligns) {
    if (reserved[r * size + c]) continue;
    for (let dy = -2; dy <= 2; dy++) for (let dx = -2; dx <= 2; dx++) {
      const border = Math.abs(dx) === 2 || Math.abs(dy) === 2;
      const center = dx === 0 && dy === 0;
      set(c + dx, r + dy, (border || center) ? 1 : 0);
    }
  }
  // 暗模块
  set(8, size - 8, 1);

  // ---- 数据填充 (标准 Z 字形扫描, 掩码 0: (i+j)%2==0 取反) ----
  let bitIdx = 0;
  const nextBit = (): boolean => bitIdx < finalBits.length ? finalBits[bitIdx++] === '1' : false;
  let upward = true;
  for (let col = size - 1; col > 0; col -= 2) {
    if (col === 6) col = 5; // 跳过时序列
    for (let i = 0; i < size; i++) {
      const y = upward ? size - 1 - i : i;
      for (let dx = 0; dx < 2; dx++) {
        const x = col - dx;
        if (x < 0) continue;
        if (reserved[y * size + x]) continue;
        const bit = nextBit();
        const invert = (x + y) % 2 === 0; // mask 0
        m[y * size + x] = (bit !== invert) ? 1 : 0;
      }
    }
    upward = !upward;
  }

  // ---- 格式信息 (掩码 0) ----
  const ecBits = atN(EC_BITS, level);
  let fmt = (ecBits << 3 | 0) << 10;
  const g = 0b10100110111;
  for (let i = 14; i >= 10; i--) if ((fmt >> i) & 1) fmt ^= g << (i - 10);
  fmt = ((ecBits << 3 | 0) << 10) | (fmt & 0x3ff);
  fmt ^= 0b101010000010010;
  // 位置1
  for (let i = 0; i <= 5; i++) set(i, 8, (fmt >> i) & 1);
  set(7, 8, (fmt >> 6) & 1);
  set(8, 8, (fmt >> 7) & 1);
  set(8, size - 8 + 0, (fmt >> 8) & 1);
  for (let i = 9; i < 15; i++) set(8, size - 15 + i, (fmt >> i) & 1);
  // 位置2
  for (let i = 0; i < 8; i++) set(8, size - 1 - i, (fmt >> i) & 1);
  set(size - 8, 8, (fmt >> 14) & 1);
  for (let i = 9; i < 15; i++) set(size - 15 + i, 8, (fmt >> i) & 1);

  return { size, get: (x, y) => m[y * size + x] === 1 };
}
