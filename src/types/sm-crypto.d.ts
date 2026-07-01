declare module 'sm-crypto' {
  export interface Sm2KeyPair {
    privateKey: string;
    publicKey: string;
  }

  export const sm2: {
    generateKeyPairHex(): Sm2KeyPair;
    doEncrypt(msg: string, publicKey: string, cipherMode?: number): string;
    doDecrypt(encryptData: string, privateKey: string, cipherMode?: number): string;
    doSignature(msg: string, privateKey: string, options?: Record<string, unknown>): string;
    doVerifySignature(
      msg: string,
      signHex: string,
      publicKey: string,
      options?: Record<string, unknown>,
    ): boolean;
    getPublicKeyFromPrivateKey(privateKey: string): string;
  };

  export const sm3: {
    (msg: string | number[]): string;
  };

  export const sm4: {
    encrypt(
      msg: string | number[],
      key: string,
      options?: { mode?: 'ecb' | 'cbc'; iv?: string },
    ): string;
    decrypt(
      encryptData: string,
      key: string,
      options?: { mode?: 'ecb' | 'cbc'; iv?: string },
    ): string;
  };
}
