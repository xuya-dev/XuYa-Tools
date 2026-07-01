<template>
  <ToolShell
    title="公私钥生成"
    :icon="KeyRound"
    description="生成 RSA / ECDSA / Ed25519 密钥对，导出 Base64 / JWK 格式。"
  >
    <div class="kp-toolbar">
      <div class="opt-group">
        <span class="opt-label">算法</span>
        <select v-model="algo" class="opt-select" :disabled="generating">
          <option v-for="a in algorithms" :key="a.id" :value="a.id">{{ a.label }}</option>
        </select>
      </div>

      <span class="sep"></span>

      <div class="opt-group">
        <span class="opt-label">格式</span>
        <div class="seg">
          <button :class="{ active: format === 'pem' }" @click="format = 'pem'">Base64</button>
          <button :class="{ active: format === 'jwk' }" @click="format = 'jwk'">JWK</button>
        </div>
      </div>

      <span style="flex: 1"></span>

      <BaseButton variant="primary" :disabled="generating" @click="generate">
        <RefreshCw :size="14" :class="{ spin: generating }" />
        {{ generating ? '生成中…' : '生成密钥对' }}
      </BaseButton>
    </div>

    <div v-if="error" class="kp-error">{{ error }}</div>

    <template v-if="publicKey">
      <div v-if="keyInfo" class="kp-info">
        <span class="info-item">
          <span class="info-label">算法</span>
          <span class="info-val">{{ keyInfo.algoName }}</span>
        </span>
        <span class="info-item">
          <span class="info-label">{{ keyInfo.keySizeLabel }}</span>
          <span class="info-val">{{ keyInfo.keySize }}</span>
        </span>
        <span class="info-item">
          <span class="info-label">用途</span>
          <span class="info-val">{{ keyInfo.usage }}</span>
        </span>
        <span class="info-item">
          <span class="info-label">格式</span>
          <span class="info-val">{{ format.toUpperCase() }}</span>
        </span>
      </div>

      <div class="kp-grid">
        <div class="kp-col">
          <div class="kp-head">
            <span class="dot pub"></span>
            公钥 (Public Key)
            <button class="mini-btn" title="下载公钥" @click="downloadKey(publicKey, 'public')">
              <Download :size="12" />
            </button>
            <button class="mini-btn" title="复制公钥" @click="copyKey(publicKey)">
              <Copy :size="12" />
            </button>
          </div>
          <pre class="kp-code"><code>{{ publicKey }}</code></pre>
        </div>
        <div class="kp-col">
          <div class="kp-head">
            <span class="dot pri"></span>
            私钥 (Private Key)
            <span class="warn-badge">勿泄露</span>
            <button class="mini-btn" title="下载私钥" @click="downloadKey(privateKey, 'private')">
              <Download :size="12" />
            </button>
            <button class="mini-btn" title="复制私钥" @click="copyKey(privateKey)">
              <Copy :size="12" />
            </button>
          </div>
          <pre class="kp-code"><code>{{ privateKey }}</code></pre>
        </div>
      </div>
    </template>

    <div v-else-if="!error" class="kp-placeholder">
      <KeyRound :size="40" />
      <p>选择算法后点击「生成密钥对」</p>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { KeyRound, Copy, Download, RefreshCw } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToast } from '@/composables/useToast';
import { useToolState } from '@/composables/useToolState';

const toast = useToast();

type AlgoId = 'rsa-2048' | 'rsa-4096' | 'ecdsa-p256' | 'ecdsa-p384' | 'ecdsa-p521' | 'ed25519';
type KeyFormat = 'pem' | 'jwk';

const algorithms: { id: AlgoId; label: string }[] = [
  { id: 'rsa-2048', label: 'RSA 2048' },
  { id: 'rsa-4096', label: 'RSA 4096' },
  { id: 'ecdsa-p256', label: 'ECDSA P-256 (secp256r1)' },
  { id: 'ecdsa-p384', label: 'ECDSA P-384 (secp384r1)' },
  { id: 'ecdsa-p521', label: 'ECDSA P-521 (secp521r1)' },
  { id: 'ed25519', label: 'Ed25519' },
];

const algo = useToolState<AlgoId>('keygen', 'algo', 'rsa-2048');
const format = useToolState<KeyFormat>('keygen', 'format', 'pem');
const publicKey = ref('');
const privateKey = ref('');
const generating = ref(false);
const error = ref('');

const keyInfo = computed(() => {
  switch (algo.value) {
    case 'rsa-2048':
      return {
        algoName: 'RSASSA-PKCS1-v1_5',
        keySizeLabel: '密钥长度',
        keySize: '2048 bit',
        usage: '签名 / 验证',
      };
    case 'rsa-4096':
      return {
        algoName: 'RSASSA-PKCS1-v1_5',
        keySizeLabel: '密钥长度',
        keySize: '4096 bit',
        usage: '签名 / 验证',
      };
    case 'ecdsa-p256':
      return { algoName: 'ECDSA', keySizeLabel: '曲线', keySize: 'P-256', usage: '签名 / 验证' };
    case 'ecdsa-p384':
      return { algoName: 'ECDSA', keySizeLabel: '曲线', keySize: 'P-384', usage: '签名 / 验证' };
    case 'ecdsa-p521':
      return { algoName: 'ECDSA', keySizeLabel: '曲线', keySize: 'P-521', usage: '签名 / 验证' };
    case 'ed25519':
      return {
        algoName: 'Ed25519',
        keySizeLabel: '密钥长度',
        keySize: '256 bit',
        usage: '签名 / 验证',
      };
  }
  return null;
});

function getAlgoParams(id: AlgoId) {
  switch (id) {
    case 'rsa-2048':
      return {
        name: 'RSASSA-PKCS1-v1_5',
        modulusLength: 2048,
        publicExponent: new Uint8Array([1, 0, 1]),
        hash: 'SHA-256',
      };
    case 'rsa-4096':
      return {
        name: 'RSASSA-PKCS1-v1_5',
        modulusLength: 4096,
        publicExponent: new Uint8Array([1, 0, 1]),
        hash: 'SHA-256',
      };
    case 'ecdsa-p256':
      return { name: 'ECDSA', namedCurve: 'P-256' };
    case 'ecdsa-p384':
      return { name: 'ECDSA', namedCurve: 'P-384' };
    case 'ecdsa-p521':
      return { name: 'ECDSA', namedCurve: 'P-521' };
    case 'ed25519':
      return { name: 'Ed25519' };
  }
}

function abToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary);
}

function abToBase64Lines(buffer: ArrayBuffer): string {
  const base64 = abToBase64(buffer);
  const lines = base64.match(/.{1,64}/g) ?? [];
  return lines.join('\n');
}

async function generate() {
  generating.value = true;
  error.value = '';
  publicKey.value = '';
  privateKey.value = '';
  try {
    const params = getAlgoParams(algo.value);
    const result = await crypto.subtle.generateKey(params, true, ['sign', 'verify']);
    const keyPair = 'publicKey' in result ? result : { publicKey: result, privateKey: result };

    if (format.value === 'jwk') {
      const pubJwk = await crypto.subtle.exportKey('jwk', keyPair.publicKey);
      const priJwk = await crypto.subtle.exportKey('jwk', keyPair.privateKey);
      const { d, ...pubWithoutPrivate } = pubJwk;
      void d;
      publicKey.value = JSON.stringify(pubWithoutPrivate, null, 2);
      privateKey.value = JSON.stringify(priJwk, null, 2);
    } else {
      const pubBuf = await crypto.subtle.exportKey('spki', keyPair.publicKey);
      const priBuf = await crypto.subtle.exportKey('pkcs8', keyPair.privateKey);
      publicKey.value = abToBase64Lines(pubBuf);
      privateKey.value = abToBase64Lines(priBuf);
    }
    toast.success('密钥对生成成功');
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    generating.value = false;
  }
}

async function copyKey(text: string) {
  await copyToClipboard(text, '已复制');
}

function downloadKey(content: string, type: 'public' | 'private') {
  const ext = format.value === 'jwk' ? 'json' : 'pem';
  const name = `${type}_key.${ext}`;
  const blob = new Blob([content], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = name;
  a.click();
  URL.revokeObjectURL(url);
  toast.success(`已下载 ${name}`);
}
</script>

<style scoped>
.kp-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}
.sep {
  width: 1px;
  height: 20px;
  background: var(--xuya-border);
  margin: 0 2px;
}
.opt-group {
  display: flex;
  align-items: center;
  gap: 6px;
}
.opt-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.opt-select {
  padding: 5px 10px;
  font-size: 12px;
  color: var(--xuya-text);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  outline: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.opt-select:hover {
  border-color: var(--xuya-border-strong);
}
.opt-select:focus {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.seg {
  display: inline-flex;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  padding: 2px;
  gap: 2px;
}
.seg button {
  padding: 4px 12px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: none;
  border-radius: var(--xuya-radius-sm);
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  cursor: pointer;
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

.kp-error {
  padding: 12px 16px;
  margin-bottom: 16px;
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
  border-radius: var(--xuya-radius);
  font-size: 13px;
}

.kp-info {
  display: flex;
  flex-wrap: wrap;
  gap: 16px 28px;
  padding: 14px 18px;
  margin-bottom: 16px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.info-item {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.info-label {
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
  font-weight: 600;
}
.info-val {
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
}

.kp-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
.kp-col {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.kp-head {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
}
.kp-head .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot.pub {
  background: var(--xuya-accent-2);
}
.dot.pri {
  background: var(--xuya-danger);
}
.warn-badge {
  font-size: 10px;
  font-weight: 600;
  color: var(--xuya-warn);
  background: var(--xuya-warn-soft);
  padding: 1px 6px;
  border-radius: 99px;
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
  margin-left: auto;
}
.mini-btn:first-of-type {
  margin-left: auto;
}
.mini-btn:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}
.kp-code {
  margin: 0;
  padding: 12px;
  max-height: 320px;
  overflow: auto;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  font-family: var(--xuya-font-mono);
  font-size: 11.5px;
  line-height: 1.6;
  word-break: break-all;
  color: var(--xuya-text);
  white-space: pre-wrap;
}

.kp-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 0;
  color: var(--xuya-text-tertiary);
}
.kp-placeholder p {
  font-size: 14px;
}

.spin {
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
