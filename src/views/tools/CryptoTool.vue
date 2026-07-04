<template>
  <ToolShell
    title="加密解密"
    :icon="Lock"
    description="哈希 (MD5/SHA/SM3)、对称加密 (AES/SM4)、RSA 加解密与签名验证、SM2 非对称。"
  >
    <div class="tabs">
      <button
        v-for="t in tabs"
        :key="t.id"
        class="tab"
        :class="{ active: tab === t.id }"
        @click="tab = t.id"
      >
        {{ t.label }}
      </button>
    </div>

    <!-- 哈希摘要 -->
    <div v-if="tab === 'hash'" class="panel">
      <textarea
        v-model="hashInput"
        class="text-area"
        placeholder="输入要计算哈希的文本…"
        spellcheck="false"
      ></textarea>
      <div class="hash-list">
        <div v-for="a in hashAlgos" :key="a.id" class="hash-item">
          <div class="hash-head">
            <span class="hash-algo">{{ a.label }}</span>
            <button
              class="mini-btn"
              :disabled="!hashOutputs[a.id]"
              @click="copy(hashOutputs[a.id])"
            >
              <Copy :size="12" />
              复制
            </button>
          </div>
          <code class="hash-value">{{ hashOutputs[a.id] || '—' }}</code>
        </div>
      </div>
    </div>

    <!-- 对称加密 AES / SM4 -->
    <div v-else-if="tab === 'symmetric'" class="panel">
      <div class="sym-algo-row">
        <span class="ctrl-label">算法</span>
        <div class="seg">
          <button
            v-for="a in symAlgos"
            :key="a.id"
            :class="{ active: symAlgo === a.id }"
            @click="symAlgo = a.id"
          >
            {{ a.label }}
          </button>
        </div>
        <span class="ctrl-label">模式</span>
        <div class="seg">
          <button
            v-for="m in symModes"
            :key="m.id"
            :class="{ active: symMode === m.id }"
            @click="symMode = m.id"
          >
            {{ m.label }}
          </button>
        </div>
      </div>

      <div class="key-row">
        <div class="key-field">
          <label class="ctrl-label">
            <span>密钥</span>
            <span class="key-meta">
              <span
                class="fmt-toggle"
                :class="{ active: symKeyFormat === 'text' }"
                role="button"
                tabindex="0"
                :title="
                  symKeyFormat === 'hex'
                    ? '当前:Hex 字符串(每 2 hex = 1 字节)'
                    : '当前:UTF-8 文本(原始字节,MySQL 语义,右侧补 0)'
                "
                @click="symKeyFormat = symKeyFormat === 'hex' ? 'text' : 'hex'"
                @keydown.enter="symKeyFormat = symKeyFormat === 'hex' ? 'text' : 'hex'"
                @keydown.space.prevent="symKeyFormat = symKeyFormat === 'hex' ? 'text' : 'hex'"
              >
                {{ symKeyFormat === 'hex' ? 'Hex' : '文本' }}
              </span>
              <button
                v-if="symKeyFormat === 'hex'"
                class="gen-btn"
                @click="symKey = genHex(symKeyBytes)"
              >
                随机
              </button>
              <span
                v-else
                class="text-keysize"
                :title="'按字节长度自动判定:≤16→AES-128,17~24→192,25~32→256'"
              >
                {{ textKeySizeHint }}
              </span>
            </span>
          </label>
          <input
            v-model="symKey"
            class="mono-input"
            :placeholder="
              symKeyFormat === 'hex'
                ? `${symKeyBytes} 字节 = ${symKeyBytes * 2} hex`
                : '文本密钥(MySQL 兼容,取 UTF-8 原始字节)'
            "
            spellcheck="false"
          />
        </div>
        <div v-if="symMode !== 'ecb'" class="key-field">
          <label class="ctrl-label">
            IV (Hex)
            <button class="gen-btn" @click="symIv = genHex(16)">随机</button>
          </label>
          <input
            v-model="symIv"
            class="mono-input"
            placeholder="16 字节 = 32 hex"
            spellcheck="false"
          />
        </div>
      </div>

      <div class="io-row">
        <div class="io-col">
          <label class="ctrl-label">{{ symDirection === 'encrypt' ? '明文' : '密文 (Hex)' }}</label>
          <textarea
            v-model="symInput"
            class="text-area-sm"
            :placeholder="symDirection === 'encrypt' ? '输入明文…' : '输入 Hex 密文…'"
            spellcheck="false"
          ></textarea>
        </div>
        <div class="io-col">
          <label class="ctrl-label">结果</label>
          <div class="result-box">
            <span v-if="symError" class="err-text">{{ symError }}</span>
            <code v-else-if="symOutput">{{ symOutput }}</code>
            <span v-else class="placeholder-text">—</span>
          </div>
        </div>
      </div>

      <div class="sym-actions">
        <div class="seg">
          <button :class="{ active: symDirection === 'encrypt' }" @click="symDirection = 'encrypt'">
            加密 ▸
          </button>
          <button :class="{ active: symDirection === 'decrypt' }" @click="symDirection = 'decrypt'">
            ◂ 解密
          </button>
        </div>
        <BaseButton variant="primary" @click="runSym">执行</BaseButton>
        <BaseButton variant="ghost" :disabled="!symOutput" @click="copy(symOutput)">
          <Copy :size="13" />
          复制结果
        </BaseButton>
      </div>
    </div>

    <!-- RSA / 签名 -->
    <div v-else-if="tab === 'rsa'" class="panel">
      <div class="sym-algo-row">
        <span class="ctrl-label">操作</span>
        <div class="seg">
          <button :class="{ active: rsaMode === 'crypt' }" @click="rsaMode = 'crypt'">
            加密 / 解密
          </button>
          <button :class="{ active: rsaMode === 'sign' }" @click="rsaMode = 'sign'">
            签名 / 验签
          </button>
        </div>
      </div>

      <div class="rsa-key-section">
        <label class="ctrl-label">{{ rsaKeyLabel }}</label>
        <textarea
          v-model="rsaKey"
          class="text-area-sm rsa-key-input"
          placeholder="粘贴 Base64 或 PEM 格式密钥"
          spellcheck="false"
        ></textarea>
      </div>

      <div class="io-row">
        <div class="io-col">
          <label class="ctrl-label">{{ rsaInputLabel }}</label>
          <textarea
            v-model="rsaInput"
            class="text-area-sm"
            :placeholder="rsaInputPh"
            spellcheck="false"
          ></textarea>
        </div>
        <div v-if="rsaMode === 'sign' && rsaDirection === 'verify'" class="io-col">
          <label class="ctrl-label">签名 (Hex)</label>
          <textarea
            v-model="rsaSig"
            class="text-area-sm"
            placeholder="粘贴 Hex 签名…"
            spellcheck="false"
          ></textarea>
        </div>
        <div class="io-col">
          <label class="ctrl-label">结果</label>
          <div class="result-box">
            <span v-if="rsaError" class="err-text">{{ rsaError }}</span>
            <span
              v-else-if="rsaVerifyDone"
              class="verify-text"
              :class="rsaVerifyOk ? 'ok' : 'fail'"
            >
              {{ rsaVerifyOk ? '✓ 签名验证通过' : '✗ 签名验证失败' }}
            </span>
            <code v-else-if="rsaOutput">{{ rsaOutput }}</code>
            <span v-else class="placeholder-text">—</span>
          </div>
        </div>
      </div>

      <div class="sym-actions">
        <div class="seg">
          <button
            v-if="rsaMode === 'crypt'"
            :class="{ active: rsaDirection === 'encrypt' }"
            @click="rsaDirection = 'encrypt'"
          >
            公钥加密 ▸
          </button>
          <button
            v-if="rsaMode === 'crypt'"
            :class="{ active: rsaDirection === 'decrypt' }"
            @click="rsaDirection = 'decrypt'"
          >
            ◂ 私钥解密
          </button>
          <button
            v-if="rsaMode === 'sign'"
            :class="{ active: rsaDirection === 'sign' }"
            @click="rsaDirection = 'sign'"
          >
            私钥签名 ▸
          </button>
          <button
            v-if="rsaMode === 'sign'"
            :class="{ active: rsaDirection === 'verify' }"
            @click="rsaDirection = 'verify'"
          >
            ◂ 公钥验签
          </button>
        </div>
        <BaseButton variant="primary" @click="runRsa">执行</BaseButton>
        <BaseButton
          v-if="rsaMode === 'sign' && rsaDirection === 'sign'"
          variant="ghost"
          :disabled="!rsaOutput"
          @click="copy(rsaOutput)"
        >
          <Copy :size="13" />
          复制签名
        </BaseButton>
      </div>
    </div>

    <!-- SM2 非对称 -->
    <div v-else class="panel">
      <div class="sm2-keypair">
        <div class="key-display">
          <label class="ctrl-label">公钥</label>
          <div class="key-box-row">
            <input v-model="sm2PublicKey" class="mono-input" placeholder="04…" spellcheck="false" />
            <button class="mini-btn" :disabled="!sm2PublicKey" @click="copy(sm2PublicKey)">
              <Copy :size="12" />
            </button>
          </div>
        </div>
        <div class="key-display">
          <label class="ctrl-label">私钥</label>
          <div class="key-box-row">
            <input v-model="sm2PrivateKey" class="mono-input" placeholder="…" spellcheck="false" />
            <button class="mini-btn" :disabled="!sm2PrivateKey" @click="copy(sm2PrivateKey)">
              <Copy :size="12" />
            </button>
          </div>
        </div>
        <BaseButton variant="ghost" @click="genSm2Keypair">
          <RefreshCw :size="13" />
          生成密钥对
        </BaseButton>
      </div>

      <div class="io-row">
        <div class="io-col">
          <label class="ctrl-label">
            {{ sm2Direction === 'encrypt' ? '明文' : '密文 (Hex/C1C3C2)' }}
          </label>
          <textarea
            v-model="sm2Input"
            class="text-area-sm"
            :placeholder="sm2Direction === 'encrypt' ? '输入明文…' : '输入密文…'"
            spellcheck="false"
          ></textarea>
        </div>
        <div class="io-col">
          <label class="ctrl-label">结果</label>
          <div class="result-box">
            <span v-if="sm2Error" class="err-text">{{ sm2Error }}</span>
            <code v-else-if="sm2Output">{{ sm2Output }}</code>
            <span v-else class="placeholder-text">—</span>
          </div>
        </div>
      </div>

      <div class="sym-actions">
        <div class="seg">
          <button :class="{ active: sm2Direction === 'encrypt' }" @click="sm2Direction = 'encrypt'">
            公钥加密 ▸
          </button>
          <button :class="{ active: sm2Direction === 'decrypt' }" @click="sm2Direction = 'decrypt'">
            ◂ 私钥解密
          </button>
        </div>
        <BaseButton variant="primary" @click="runSm2">执行</BaseButton>
        <BaseButton variant="ghost" :disabled="!sm2Output" @click="copy(sm2Output)">
          <Copy :size="13" />
          复制
        </BaseButton>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Lock, Copy, RefreshCw } from '@lucide/vue';
import { md5 } from 'js-md5';
import CryptoJS from 'crypto-js';
import { sm2, sm3, sm4 } from 'sm-crypto';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const tabs = [
  { id: 'hash' as const, label: '哈希摘要' },
  { id: 'symmetric' as const, label: '对称加密' },
  { id: 'rsa' as const, label: 'RSA / 签名' },
  { id: 'sm2' as const, label: 'SM2 非对称' },
];
const tab = ref<'hash' | 'symmetric' | 'rsa' | 'sm2'>('hash');

async function copy(text: string | undefined) {
  if (text) await copyToClipboard(text, '已复制');
}

function bufToHex(buf: ArrayBuffer): string {
  return Array.from(new Uint8Array(buf))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

function hexToBytes(hex: string) {
  const clean = hex.replace(/\s/g, '');
  const len = Math.floor(clean.length / 2);
  const buf = new ArrayBuffer(len);
  const out = new Uint8Array(buf);
  for (let i = 0; i < len; i++) out[i] = parseInt(clean.substr(i * 2, 2), 16);
  return out;
}

function genHex(bytes: number): string {
  const buf = new Uint8Array(bytes);
  crypto.getRandomValues(buf);
  return Array.from(buf)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

function pemToBuffer(pem: string): ArrayBuffer {
  const base64 = pem.replace(/-----[^-]+-----/g, '').replace(/\s/g, '');
  const binary = atob(base64);
  const buf = new ArrayBuffer(binary.length);
  const view = new Uint8Array(buf);
  for (let i = 0; i < binary.length; i++) view[i] = binary.charCodeAt(i);
  return buf;
}

// ============ 哈希 ============
const hashInput = ref('');
const hashAlgos = [
  { id: 'md5' as const, label: 'MD5' },
  { id: 'sha1' as const, label: 'SHA-1' },
  { id: 'sha256' as const, label: 'SHA-256' },
  { id: 'sha384' as const, label: 'SHA-384' },
  { id: 'sha512' as const, label: 'SHA-512' },
  { id: 'sm3' as const, label: 'SM3 (国密)' },
];
const hashOutputs = ref<Record<string, string>>({});

async function computeHash() {
  const text = hashInput.value;
  if (!text) {
    hashOutputs.value = {};
    return;
  }
  const enc = new TextEncoder().encode(text);
  const out: Record<string, string> = {};
  out.md5 = md5(text);
  for (const algo of ['SHA-1', 'SHA-256', 'SHA-384', 'SHA-512']) {
    const digest = await crypto.subtle.digest(algo, enc);
    out[algo.replace('-', '').toLowerCase()] = bufToHex(digest);
  }
  out.sm3 = sm3(text);
  hashOutputs.value = out;
}
watch(hashInput, computeHash);

// ============ 对称加密 ============
const symAlgo = useToolState<'aes' | 'sm4'>('crypto', 'symAlgo', 'aes');
const symMode = useToolState<'ecb' | 'cbc' | 'gcm'>('crypto', 'symMode', 'cbc');
const symDirection = useToolState<'encrypt' | 'decrypt'>('crypto', 'symDir', 'encrypt');
/** 密钥格式:hex = 按 hex 解析(严格 16/24/32 字节);text = 按 UTF-8 文本字节(MySQL 语义,右侧补 0)。 */
const symKeyFormat = useToolState<'hex' | 'text'>('crypto', 'symKeyFormat', 'hex');
const symKey = ref('');
const symIv = ref('');
const symInput = ref('');
const symOutput = ref('');
const symError = ref('');

const symAlgos = [
  { id: 'aes' as const, label: 'AES' },
  { id: 'sm4' as const, label: 'SM4 (国密)' },
];
const symModes = computed(() => {
  if (symAlgo.value === 'aes') {
    return [
      { id: 'ecb' as const, label: 'ECB/PKCS7' },
      { id: 'cbc' as const, label: 'CBC' },
      { id: 'gcm' as const, label: 'GCM' },
    ];
  }
  return [
    { id: 'ecb' as const, label: 'ECB' },
    { id: 'cbc' as const, label: 'CBC' },
  ];
});
const symKeyBytes = computed(() => (symAlgo.value === 'aes' ? 16 : 16));

/** 文本密钥模式下,根据当前密钥的字节长度提示 MySQL 自动判定的 AES key size。 */
const textKeySizeHint = computed(() => {
  const len = symKey.value.trim() ? new TextEncoder().encode(symKey.value.trim()).length : 0;
  if (len === 0) return 'AES-?';
  if (len <= 16) return `AES-128 (${len}B)`;
  if (len <= 24) return `AES-192 (${len}B)`;
  return `AES-256 (${len}B)`;
});

watch(symAlgo, () => {
  const modes = symModes.value;
  if (!modes.find((m) => m.id === symMode.value)) symMode.value = modes[0]!.id;
});

async function runSym() {
  symError.value = '';
  symOutput.value = '';
  if (!symInput.value.trim() || !symKey.value.trim()) {
    symError.value = '请输入文本和密钥';
    return;
  }
  try {
    symOutput.value = symAlgo.value === 'aes' ? await runAes() : runSm4();
  } catch (e) {
    symError.value = e instanceof Error ? e.message : String(e);
  }
}

function runAesEcb(): string {
  const key = parseSymKey();
  if (symDirection.value === 'encrypt') {
    const enc = CryptoJS.AES.encrypt(symInput.value, key, {
      mode: CryptoJS.mode.ECB,
      padding: CryptoJS.pad.Pkcs7,
    });
    return enc.ciphertext.toString(CryptoJS.enc.Hex);
  }
  const cipherParams = CryptoJS.lib.CipherParams.create({
    ciphertext: CryptoJS.enc.Hex.parse(symInput.value.trim()),
  });
  const dec = CryptoJS.AES.decrypt(cipherParams, key, {
    mode: CryptoJS.mode.ECB,
    padding: CryptoJS.pad.Pkcs7,
  });
  return safeUtf8(dec);
}

/**
 * 解析对称密钥。
 * - hex:严格按 hex 解析(必须 16/24/32 字节)。
 * - text:取 UTF-8 原始字节(MySQL AES_ENCRYPT 语义),右侧补 \0 到 16/24/32,
 *   字节长度决定 key size(≤16→128,17~24→192,25~32→256)。
 *   这样可直接解 MySQL 库中 HEX(AES_ENCRYPT(...)) 的密文。
 */
function parseSymKey(): CryptoJS.lib.WordArray {
  const raw = symKey.value.trim();
  if (!raw) throw new Error('请输入密钥');
  if (symKeyFormat.value === 'hex') {
    return CryptoJS.enc.Hex.parse(raw.replace(/\s/g, ''));
  }
  const bytes = new TextEncoder().encode(raw);
  const size = bytes.length <= 16 ? 16 : bytes.length <= 24 ? 24 : 32;
  if (bytes.length > size) throw new Error(`密钥过长(${bytes.length} 字节),文本模式最长 32 字节`);
  const padded = new Uint8Array(size);
  padded.set(bytes);
  return bytesToWordArray(padded);
}

function bytesToWordArray(bytes: Uint8Array): CryptoJS.lib.WordArray {
  const hex = Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
  return CryptoJS.enc.Hex.parse(hex);
}

/** 安全转 UTF-8:解密失败(密钥/模式不对)时 dec 往往含非法字节,直接 toString(Utf8) 会抛错吞掉结果。 */
function safeUtf8(dec: CryptoJS.lib.WordArray): string {
  const { words, sigBytes } = dec;
  const bytes = new Uint8Array(sigBytes);
  for (let i = 0; i < sigBytes; i++) {
    const word = words[i >>> 2] ?? 0;
    bytes[i] = (word >>> (24 - (i % 4) * 8)) & 0xff;
  }
  return new TextDecoder('utf-8', { fatal: false }).decode(bytes);
}

async function runAes(): Promise<string> {
  if (symMode.value === 'ecb') return runAesEcb();
  const keyBytes = hexToBytes(symKey.value);
  if (keyBytes.length !== 16 && keyBytes.length !== 24 && keyBytes.length !== 32) {
    throw new Error('AES 密钥必须为 16/24/32 字节 (128/192/256 位)');
  }
  const mode = symMode.value === 'gcm' ? 'AES-GCM' : 'AES-CBC';
  const cryptoKey = await crypto.subtle.importKey('raw', keyBytes, { name: mode }, false, [
    symDirection.value,
  ]);
  if (symDirection.value === 'encrypt') {
    const iv = hexToBytes(symIv.value || genHex(16));
    const data = new TextEncoder().encode(symInput.value);
    const encrypted = await crypto.subtle.encrypt({ name: mode, iv }, cryptoKey, data);
    return bufToHex(encrypted);
  }
  const iv = hexToBytes(symIv.value);
  const data = hexToBytes(symInput.value.trim());
  const decrypted = await crypto.subtle.decrypt({ name: mode, iv }, cryptoKey, data);
  return new TextDecoder().decode(decrypted);
}

function runSm4(): string {
  const key = symKey.value.trim();
  if (key.length !== 32) throw new Error('SM4 密钥必须为 32 hex (16 字节 / 128 位)');
  const opts =
    symMode.value === 'cbc'
      ? { mode: 'cbc' as const, iv: symIv.value.trim() }
      : { mode: 'ecb' as const };
  return symDirection.value === 'encrypt'
    ? sm4.encrypt(symInput.value, key, opts)
    : sm4.decrypt(symInput.value.trim(), key, opts);
}

// ============ RSA ============
const rsaMode = useToolState<'crypt' | 'sign'>('crypto', 'rsaMode', 'crypt');
const rsaDirection = useToolState<'encrypt' | 'decrypt' | 'sign' | 'verify'>(
  'crypto',
  'rsaDir',
  'encrypt',
);
const rsaKey = ref('');
const rsaInput = ref('');
const rsaSig = ref('');
const rsaOutput = ref('');
const rsaError = ref('');
const rsaVerifyDone = ref(false);
const rsaVerifyOk = ref(false);

watch(rsaMode, (m) => {
  rsaDirection.value = m === 'crypt' ? 'encrypt' : 'sign';
  rsaOutput.value = '';
  rsaError.value = '';
  rsaVerifyDone.value = false;
});
watch(rsaDirection, () => {
  rsaOutput.value = '';
  rsaError.value = '';
  rsaVerifyDone.value = false;
});

const rsaKeyLabel = computed(() => {
  if (rsaMode.value === 'crypt') {
    return rsaDirection.value === 'encrypt' ? '公钥' : '私钥';
  }
  return rsaDirection.value === 'sign' ? '私钥' : '公钥';
});
const rsaInputLabel = computed(() => {
  if (rsaMode.value === 'crypt') return rsaDirection.value === 'encrypt' ? '明文' : '密文 (Hex)';
  return '签名原文';
});
const rsaInputPh = computed(() => {
  if (rsaMode.value === 'crypt')
    return rsaDirection.value === 'encrypt' ? '输入明文…' : '输入 Hex 密文…';
  return '输入要签名 / 验签的文本…';
});

async function runRsa() {
  rsaError.value = '';
  rsaOutput.value = '';
  rsaVerifyDone.value = false;
  if (!rsaKey.value.trim()) {
    rsaError.value = '请输入密钥';
    return;
  }
  if (!rsaInput.value.trim()) {
    rsaError.value = '请输入文本';
    return;
  }
  try {
    const keyBuffer = pemToBuffer(rsaKey.value);
    if (rsaMode.value === 'crypt') {
      rsaOutput.value = await rsaCrypt(keyBuffer);
    } else if (rsaDirection.value === 'sign') {
      rsaOutput.value = await rsaSign(keyBuffer);
    } else {
      await rsaVerify(keyBuffer);
    }
  } catch (e) {
    rsaError.value = e instanceof Error ? e.message : String(e);
  }
}

async function rsaCrypt(keyBuffer: ArrayBuffer): Promise<string> {
  if (rsaDirection.value === 'encrypt') {
    const key = await crypto.subtle.importKey(
      'spki',
      keyBuffer,
      { name: 'RSA-OAEP', hash: 'SHA-256' },
      false,
      ['encrypt'],
    );
    const data = new TextEncoder().encode(rsaInput.value);
    const encrypted = await crypto.subtle.encrypt('RSA-OAEP', key, data);
    return bufToHex(encrypted);
  }
  const key = await crypto.subtle.importKey(
    'pkcs8',
    keyBuffer,
    { name: 'RSA-OAEP', hash: 'SHA-256' },
    false,
    ['decrypt'],
  );
  const data = hexToBytes(rsaInput.value.trim());
  const decrypted = await crypto.subtle.decrypt('RSA-OAEP', key, data);
  return new TextDecoder().decode(decrypted);
}

async function rsaSign(keyBuffer: ArrayBuffer): Promise<string> {
  const key = await crypto.subtle.importKey(
    'pkcs8',
    keyBuffer,
    { name: 'RSASSA-PKCS1-v1_5', hash: 'SHA-256' },
    false,
    ['sign'],
  );
  const data = new TextEncoder().encode(rsaInput.value);
  const signature = await crypto.subtle.sign('RSASSA-PKCS1-v1_5', key, data);
  return bufToHex(signature);
}

async function rsaVerify(keyBuffer: ArrayBuffer): Promise<void> {
  if (!rsaSig.value.trim()) {
    rsaError.value = '请输入签名 (Hex)';
    return;
  }
  const key = await crypto.subtle.importKey(
    'spki',
    keyBuffer,
    { name: 'RSASSA-PKCS1-v1_5', hash: 'SHA-256' },
    false,
    ['verify'],
  );
  const data = new TextEncoder().encode(rsaInput.value);
  const sig = hexToBytes(rsaSig.value.trim());
  rsaVerifyOk.value = await crypto.subtle.verify('RSASSA-PKCS1-v1_5', key, sig, data);
  rsaVerifyDone.value = true;
}

// ============ SM2 ============
const sm2PublicKey = ref('');
const sm2PrivateKey = ref('');
const sm2Input = ref('');
const sm2Output = ref('');
const sm2Error = ref('');
const sm2Direction = useToolState<'encrypt' | 'decrypt'>('crypto', 'sm2Dir', 'encrypt');
const SM2_CIPHER_MODE = 1;

function genSm2Keypair() {
  const kp = sm2.generateKeyPairHex();
  sm2PublicKey.value = kp.publicKey;
  sm2PrivateKey.value = kp.privateKey;
}

function runSm2() {
  sm2Error.value = '';
  sm2Output.value = '';
  if (!sm2Input.value.trim()) {
    sm2Error.value = '请输入文本';
    return;
  }
  try {
    if (sm2Direction.value === 'encrypt') {
      if (!sm2PublicKey.value.trim()) throw new Error('请输入或生成公钥');
      sm2Output.value = sm2.doEncrypt(sm2Input.value, sm2PublicKey.value.trim(), SM2_CIPHER_MODE);
    } else {
      if (!sm2PrivateKey.value.trim()) throw new Error('请输入或生成私钥');
      sm2Output.value = sm2.doDecrypt(
        sm2Input.value.trim(),
        sm2PrivateKey.value.trim(),
        SM2_CIPHER_MODE,
      );
    }
    if (!sm2Output.value) throw new Error('解密失败，请检查密钥和密文');
  } catch (e) {
    sm2Error.value = e instanceof Error ? e.message : String(e);
  }
}
</script>

<style scoped>
.tabs {
  display: flex;
  gap: 2px;
  margin-bottom: 20px;
  border-bottom: 1px solid var(--xuya-border);
}
.tab {
  padding: 8px 16px;
  font-size: 13px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition:
    color var(--xuya-duration-fast),
    border-color var(--xuya-duration-fast);
  cursor: pointer;
}
.tab:hover {
  color: var(--xuya-text);
}
.tab.active {
  color: var(--xuya-accent);
  border-bottom-color: var(--xuya-accent);
  font-weight: 600;
}
.panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.text-area {
  width: 100%;
  min-height: 100px;
  max-height: 200px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  resize: vertical;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.text-area:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

.hash-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.hash-item {
  padding: 10px 12px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
}
.hash-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}
.hash-algo {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-accent);
}
.hash-value {
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  word-break: break-all;
  color: var(--xuya-text);
}

.ctrl-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.seg {
  display: inline-flex;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  padding: 2px;
  gap: 2px;
}
.seg button {
  padding: 5px 14px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-radius: var(--xuya-radius-sm);
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  cursor: pointer;
  white-space: nowrap;
}
.seg button:hover {
  color: var(--xuya-text);
}
.seg button.active {
  background: var(--xuya-bg-elevated);
  color: var(--xuya-accent);
  font-weight: 600;
  box-shadow: var(--xuya-shadow-sm);
}

.sym-algo-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.key-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
.key-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.mono-input {
  width: 100%;
  padding: 8px 12px;
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  color: var(--xuya-text);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  outline: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.mono-input:focus {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.gen-btn {
  font-size: 10px;
  color: var(--xuya-accent);
  background: none;
  border: none;
  cursor: pointer;
}
.gen-btn:hover {
  text-decoration: underline;
}
.key-meta {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.fmt-toggle {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 7px;
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text-secondary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  cursor: pointer;
  user-select: none;
  transition: all var(--xuya-duration-fast);
}
.fmt-toggle:hover {
  color: var(--xuya-text);
  border-color: var(--xuya-accent);
}
.fmt-toggle.active {
  color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
  border-color: var(--xuya-accent);
}
.text-keysize {
  font-size: 10px;
  font-weight: 600;
  color: var(--xuya-accent);
  opacity: 0.85;
}

.io-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
.io-col {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.text-area-sm {
  width: 100%;
  min-height: 80px;
  max-height: 140px;
  padding: 10px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  line-height: 1.5;
  resize: vertical;
}
.text-area-sm:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.result-box {
  min-height: 80px;
  max-height: 140px;
  overflow: auto;
  padding: 10px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  word-break: break-all;
  color: var(--xuya-text);
}
.err-text {
  color: var(--xuya-danger);
}
.placeholder-text {
  color: var(--xuya-text-tertiary);
}
.verify-text {
  font-weight: 600;
  font-size: 13px;
}
.verify-text.ok {
  color: var(--xuya-success);
}
.verify-text.fail {
  color: var(--xuya-danger);
}

.sym-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.mini-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  font-size: 11px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-bg-elevated);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.mini-btn:hover:not(:disabled) {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}
.mini-btn:disabled {
  opacity: 0.4;
}

.rsa-key-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.rsa-key-input {
  min-height: 100px;
}

.sm2-keypair {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.key-display {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.key-box-row {
  display: flex;
  gap: 6px;
  align-items: stretch;
}
.key-box-row .mono-input {
  flex: 1;
}
</style>
