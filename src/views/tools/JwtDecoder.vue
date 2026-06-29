<template>
  <ToolShell title="JWT 解析" :icon="KeyRound" description="解码 JWT 的 Header / Payload / Signature,查看算法与过期时间。">
    <div class="jwt-input-wrap">
      <textarea
        v-model="token"
        class="jwt-input"
        placeholder="粘贴 JWT (eyJhbGciOi... 开头)"
        spellcheck="false"
      ></textarea>
    </div>

    <!-- 错误 -->
    <div v-if="errorMsg" class="jwt-error">
      <AlertCircle :size="16" />
      <span>{{ errorMsg }}</span>
    </div>

    <!-- 解析结果 -->
    <template v-else-if="parsed">
      <!-- 概览 -->
      <div class="jwt-overview">
        <div class="overview-item">
          <span class="label">算法</span>
          <span class="value">{{ parsed.header.alg || '—' }}</span>
        </div>
        <div class="overview-item">
          <span class="label">类型</span>
          <span class="value">{{ parsed.header.typ || '—' }}</span>
        </div>
        <div v-if="issuedAt" class="overview-item">
          <span class="label">签发于</span>
          <span class="value">{{ issuedAt }}</span>
        </div>
        <div v-if="expiresAt" class="overview-item">
          <span class="label">过期时间</span>
          <span class="value" :class="{ expired: isExpired, valid: !isExpired }">
            {{ expiresAt }}
            <small v-if="isExpired">(已过期)</small>
            <small v-else>(有效)</small>
          </span>
        </div>
      </div>

      <!-- 三段着色展示 -->
      <div class="jwt-segments">
        <span class="seg seg-0">{{ parsed.parts[0] }}</span>
        <span class="seg-dot">.</span>
        <span class="seg seg-1">{{ parsed.parts[1] }}</span>
        <span class="seg-dot">.</span>
        <span class="seg seg-2">{{ parsed.parts[2] }}</span>
      </div>

      <!-- Header / Payload -->
      <div class="parts-grid">
        <div class="part">
          <div class="part-head">
            <span class="dot seg-0-bg"></span>
            Header
            <button class="mini-btn" @click="copy(parsed.headerRaw)"><Copy :size="12" /></button>
          </div>
          <pre class="part-code"><code>{{ parsed.headerRaw }}</code></pre>
        </div>
        <div class="part">
          <div class="part-head">
            <span class="dot seg-1-bg"></span>
            Payload
            <button class="mini-btn" @click="copy(parsed.payloadRaw)"><Copy :size="12" /></button>
          </div>
          <pre class="part-code"><code>{{ parsed.payloadRaw }}</code></pre>
        </div>
      </div>
    </template>

    <!-- 占位 -->
    <div v-else class="jwt-placeholder">
      <KeyRound :size="40" />
      <p>粘贴 JWT 后自动解码</p>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { KeyRound, Copy, AlertCircle } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';

const token = ref('');

interface Parsed {
  parts: string[];
  header: Record<string, unknown>;
  payload: Record<string, unknown>;
  headerRaw: string;
  payloadRaw: string;
}

type ParseResult = { ok: true; data: Parsed } | { ok: false; error: string } | null;

const parsedResult = computed<ParseResult>(() => {
  const t = token.value.trim();
  if (!t) return null;
  const parts = t.split('.');
  if (parts.length !== 3) {
    return { ok: false, error: 'JWT 格式错误:应由 3 段 (header.payload.signature) 用点号分隔。' };
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
    return { ok: false, error: `解码失败:${e instanceof Error ? e.message : String(e)}` };
  }
});

const errorMsg = computed(() =>
  parsedResult.value && !parsedResult.value.ok ? parsedResult.value.error : '',
);

const parsed = computed<Parsed | null>(() =>
  parsedResult.value && parsedResult.value.ok ? parsedResult.value.data : null,
);

const issuedAt = computed(() => fmtClaim(parsed.value?.payload.iat));
const expiresAt = computed(() => fmtClaim(parsed.value?.payload.exp));
const isExpired = computed(() => {
  const exp = parsed.value?.payload.exp;
  if (typeof exp !== 'number') return false;
  return exp * 1000 < Date.now();
});

function b64urlDecode(s: string): string {
  // base64url → base64,补齐 padding
  let b64 = s.replace(/-/g, '+').replace(/_/g, '/');
  while (b64.length % 4) b64 += '=';
  const bin = atob(b64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return new TextDecoder().decode(bytes);
}

function fmtClaim(v: unknown): string | null {
  if (typeof v !== 'number') return null;
  const d = new Date(v * 1000);
  return d.toLocaleString('zh-CN', { hour12: false });
}

async function copy(text: string | undefined) {
  if (text) await copyToClipboard(text, '已复制');
}
</script>

<style scoped>
.jwt-input-wrap {
  margin-bottom: 16px;
}
.jwt-input {
  width: 100%;
  min-height: 70px;
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
  transition: border-color 0var(--xuya-duration-fast), box-shadow 0var(--xuya-duration-fast);
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
  display: flex;
  flex-wrap: wrap;
  gap: 16px 28px;
  padding: 16px 20px;
  margin-bottom: 16px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.overview-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.overview-item .label {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.overview-item .value {
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
}
.overview-item .value.expired {
  color: var(--xuya-danger);
}
.overview-item .value.valid {
  color: var(--xuya-success);
}
.overview-item .value small {
  font-weight: 400;
  opacity: 0.8;
}

.jwt-segments {
  padding: 12px;
  margin-bottom: 16px;
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius);
  font-family: var(--xuya-font-mono);
  font-size: 11.5px;
  word-break: break-all;
  line-height: 1.8;
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
  transition: all 0var(--xuya-duration-fast);
}
.mini-btn:hover {
  color: var(--xuya-text);
  border-color: var(--xuya-accent);
}
.part-code {
  margin: 0;
  padding: 12px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.6;
  overflow: auto;
  max-height: 280px;
  color: var(--xuya-text);
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
