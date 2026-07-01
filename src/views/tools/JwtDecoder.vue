<template>
  <ToolShell
    title="JWT 解析"
    :icon="KeyRound"
    description="解码 JWT 的 Header / Payload / Signature，查看算法、标准声明与过期状态。"
  >
    <div class="jwt-input-wrap">
      <textarea
        v-model="token"
        class="jwt-input"
        placeholder="粘贴 JWT (eyJhbGciOi… 开头)"
        spellcheck="false"
        @paste="onPaste"
      ></textarea>
    </div>

    <div v-if="errorMsg" class="jwt-error">
      <AlertCircle :size="16" />
      <span>{{ errorMsg }}</span>
    </div>

    <template v-else-if="parsed">
      <div class="jwt-overview">
        <div class="overview-row">
          <div class="overview-item">
            <span class="ov-label">算法</span>
            <span class="ov-value mono">{{ parsed.header.alg || '—' }}</span>
          </div>
          <div class="overview-item">
            <span class="ov-label">类型</span>
            <span class="ov-value">{{ parsed.header.typ || '—' }}</span>
          </div>
          <div class="overview-item">
            <span class="ov-label">状态</span>
            <span class="ov-value">
              <span class="status-badge" :class="tokenStatus">{{ statusLabel }}</span>
            </span>
          </div>
          <div v-if="expRemaining" class="overview-item">
            <span class="ov-label">剩余有效期</span>
            <span class="ov-value" :class="{ expired: isExpired }">{{ expRemaining }}</span>
          </div>
        </div>

        <div v-if="standardClaims.length" class="claims-row">
          <div v-for="c in standardClaims" :key="c.key" class="claim-item" :title="c.desc">
            <span class="claim-key mono">{{ c.key }}</span>
            <span class="claim-value">{{ c.display }}</span>
            <button class="copy-mini" @click="copyText(String(c.raw))">
              <Copy :size="11" />
            </button>
          </div>
        </div>
      </div>

      <div class="jwt-segments">
        <span class="seg seg-0" :title="'Header (Base64URL)'">{{ parsed.parts[0] }}</span>
        <span class="seg-dot">.</span>
        <span class="seg seg-1" :title="'Payload (Base64URL)'">{{ parsed.parts[1] }}</span>
        <span class="seg-dot">.</span>
        <span class="seg seg-2" :title="'Signature'">{{ parsed.parts[2] }}</span>
      </div>

      <div class="parts-grid">
        <div class="part">
          <div class="part-head">
            <span class="dot seg-0-bg"></span>
            Header
            <button class="mini-btn" @click="copyText(parsed.headerRaw)">
              <Copy :size="12" />
            </button>
          </div>
          <CodeView :model-value="parsed.headerRaw" language="json" placeholder="—" />
        </div>
        <div class="part">
          <div class="part-head">
            <span class="dot seg-1-bg"></span>
            Payload
            <button class="mini-btn" @click="copyText(parsed.payloadRaw)">
              <Copy :size="12" />
            </button>
          </div>
          <CodeView :model-value="parsed.payloadRaw" language="json" placeholder="—" />
        </div>
      </div>
    </template>

    <div v-else class="jwt-placeholder">
      <KeyRound :size="40" />
      <p>粘贴 JWT 后自动解码</p>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { KeyRound, Copy, AlertCircle } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import CodeView from '@/components/ui/CodeView.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const token = useToolState('jwt', 'token', '');

interface Parsed {
  parts: string[];
  header: Record<string, unknown>;
  payload: Record<string, unknown>;
  headerRaw: string;
  payloadRaw: string;
}

type ParseResult = { ok: true; data: Parsed } | { ok: false; error: string } | null;

const CLAIM_META: Record<string, { label: string; desc: string }> = {
  iss: { label: '签发者', desc: 'Issuer — 签发 JWT 的主体' },
  sub: { label: '主题', desc: 'Subject — JWT 所面向的用户' },
  aud: { label: '受众', desc: 'Audience — JWT 的接收方' },
  exp: { label: '过期时间', desc: 'Expiration Time' },
  nbf: { label: '生效时间', desc: 'Not Before — 在此之前不可用' },
  iat: { label: '签发时间', desc: 'Issued At' },
  jti: { label: '唯一 ID', desc: 'JWT ID' },
};

const parsedResult = computed<ParseResult>(() => {
  const t = token.value.trim();
  if (!t) return null;
  const parts = t.split('.');
  if (parts.length !== 3) {
    return { ok: false, error: 'JWT 格式错误：应由 3 段 (header.payload.signature) 用点号分隔。' };
  }
  try {
    const header = JSON.parse(b64urlDecode(parts[0] ?? ''));
    const payload = JSON.parse(b64urlDecode(parts[1] ?? ''));
    return {
      ok: true,
      data: {
        parts,
        header,
        payload,
        headerRaw: JSON.stringify(header, null, 2),
        payloadRaw: JSON.stringify(payload, null, 2),
      },
    };
  } catch (e) {
    return { ok: false, error: `解码失败：${e instanceof Error ? e.message : String(e)}` };
  }
});

const errorMsg = computed(() =>
  parsedResult.value && !parsedResult.value.ok ? parsedResult.value.error : '',
);

const parsed = computed<Parsed | null>(() =>
  parsedResult.value && parsedResult.value.ok ? parsedResult.value.data : null,
);

const tokenStatus = computed<'valid' | 'expired' | 'not-yet' | 'unknown'>(() => {
  if (!parsed.value) return 'unknown';
  const now = Math.floor(Date.now() / 1000);
  const exp = parsed.value.payload.exp;
  const nbf = parsed.value.payload.nbf;
  if (typeof exp === 'number' && now > exp) return 'expired';
  if (typeof nbf === 'number' && now < nbf) return 'not-yet';
  return 'valid';
});

const statusLabel = computed(() => {
  return { valid: '有效', expired: '已过期', 'not-yet': '尚未生效', unknown: '—' }[
    tokenStatus.value
  ];
});

const isExpired = computed(() => tokenStatus.value === 'expired');

const expRemaining = computed(() => {
  if (!parsed.value) return '';
  const exp = parsed.value.payload.exp;
  if (typeof exp !== 'number') return '';
  return timeRemaining(exp);
});

function timeRemaining(exp: number): string {
  const diff = exp * 1000 - Date.now();
  const abs = Math.abs(diff);
  const days = Math.floor(abs / 86400000);
  const hours = Math.floor((abs % 86400000) / 3600000);
  const mins = Math.floor((abs % 3600000) / 60000);
  const parts: string[] = [];
  if (days > 0) parts.push(`${days} 天`);
  if (hours > 0) parts.push(`${hours} 小时`);
  if (days === 0 && hours === 0 && mins > 0) parts.push(`${mins} 分钟`);
  if (!parts.length) return diff < 0 ? '刚刚过期' : '不足 1 分钟';
  return diff < 0 ? `已过期 ${parts.join(' ')}` : `剩余 ${parts.join(' ')}`;
}

interface ClaimDisplay {
  key: string;
  raw: unknown;
  display: string;
  desc: string;
}

const standardClaims = computed<ClaimDisplay[]>(() => {
  if (!parsed.value) return [];
  const payload = parsed.value.payload;
  const result: ClaimDisplay[] = [];
  for (const [key, meta] of Object.entries(CLAIM_META)) {
    if (key in payload) {
      const val = payload[key];
      let display: string;
      if (key === 'exp' || key === 'iat' || key === 'nbf') {
        display = typeof val === 'number' ? fmtTime(val) : String(val);
      } else if (key === 'aud' && Array.isArray(val)) {
        display = val.join(', ');
      } else {
        display = String(val);
      }
      result.push({ key, raw: val, display, desc: meta.desc });
    }
  }
  return result;
});

function b64urlDecode(s: string): string {
  let b64 = s.replace(/-/g, '+').replace(/_/g, '/');
  while (b64.length % 4) b64 += '=';
  const bin = atob(b64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return new TextDecoder().decode(bytes);
}

function fmtTime(ts: number): string {
  return new Date(ts * 1000).toLocaleString('zh-CN', { hour12: false });
}

function onPaste(e: ClipboardEvent) {
  const text = e.clipboardData?.getData('text') ?? '';
  if (text.trim()) {
    setTimeout(() => {
      token.value = token.value.trim();
    }, 0);
  }
}

async function copyText(text: string) {
  await copyToClipboard(text, '已复制');
}
</script>

<style scoped>
.jwt-input-wrap {
  margin-bottom: 16px;
}
.jwt-input {
  width: 100%;
  min-height: 70px;
  max-height: 140px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  line-height: 1.5;
  resize: vertical;
  word-break: break-all;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.jwt-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

.jwt-error {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  margin-bottom: 16px;
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
  border-radius: var(--xuya-radius);
  font-size: 13px;
}

.jwt-overview {
  padding: 14px 18px;
  margin-bottom: 14px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.overview-row {
  display: flex;
  flex-wrap: wrap;
  gap: 16px 28px;
}
.overview-item {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.ov-label {
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
  font-weight: 600;
}
.ov-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
}
.ov-value.mono {
  font-family: var(--xuya-font-mono);
}
.ov-value.expired {
  color: var(--xuya-danger);
}

.status-badge {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 99px;
  font-size: 11px;
  font-weight: 600;
}
.status-badge.valid {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.status-badge.expired {
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
}
.status-badge.not-yet {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}

.claims-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.claim-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px 4px 10px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border-light);
  border-radius: var(--xuya-radius-sm);
}
.claim-key {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-accent);
}
.claim-value {
  font-size: 11.5px;
  color: var(--xuya-text);
  word-break: break-all;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.copy-mini {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  color: var(--xuya-text-tertiary);
  border: none;
  background: transparent;
  cursor: pointer;
  transition: color var(--xuya-duration-fast);
}
.copy-mini:hover {
  color: var(--xuya-accent);
}

.jwt-segments {
  padding: 10px 12px;
  margin-bottom: 14px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  font-family: var(--xuya-font-mono);
  font-size: 11px;
  word-break: break-all;
  line-height: 1.8;
  overflow: auto;
}
.seg {
  padding: 2px 4px;
  border-radius: var(--xuya-radius-sm);
}
.seg-0 {
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent-2);
}
.seg-1 {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.seg-2 {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}
.seg-dot {
  color: var(--xuya-text-tertiary);
  margin: 0 2px;
}

.parts-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
.part {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
}
.part-head {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
}
.part-head .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.seg-0-bg {
  background: var(--xuya-accent-2);
}
.seg-1-bg {
  background: var(--xuya-success);
}
.mini-btn {
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  padding: 3px 6px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.mini-btn:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}

.jwt-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 0;
  color: var(--xuya-text-tertiary);
}
.jwt-placeholder p {
  font-size: 14px;
}
</style>
