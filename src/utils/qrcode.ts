import QRCode from 'qrcode';

export interface QRMatrix {
  size: number;
  get(x: number, y: number): boolean;
}

export function encodeQR(text: string, level: 'L' | 'M' | 'Q' | 'H' = 'M'): QRMatrix {
  const qr = QRCode.create(text || ' ', { errorCorrectionLevel: level });
  const modules = qr.modules;
  const size = (modules as unknown as { size: number }).size;
  return {
    size,
    get: (x: number, y: number) => modules.get(y, x) === 1,
  };
}
