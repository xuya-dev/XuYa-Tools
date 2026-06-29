<template>
  <ToolShell title="IP / 子网计算器" :icon="Network" description="IPv4 子网划分:CIDR、掩码、网段范围、可用主机数一键计算。">
    <!-- 输入 -->
    <div class="ip-input-row">
      <div class="ip-field">
        <label class="ctrl-label">IP 地址 / CIDR</label>
        <input v-model="input" class="ip-input mono" type="text" placeholder="192.168.1.100/24" spellcheck="false" @input="compute" />
      </div>
      <div class="ip-result-badge" :class="{ valid: result, invalid: error }">
        <CheckCircle2 v-if="result" :size="16" />
        <AlertCircle v-else-if="error" :size="16" />
        {{ error ? '无效' : (result ? '有效' : '输入 IP/CIDR') }}
      </div>
    </div>

    <div v-if="error" class="ip-error">⚠️ {{ error }}</div>

    <!-- 结果网格 -->
    <div v-if="result" class="ip-results">
      <div v-for="f in FIELDS" :key="f.key" class="ip-card" @click="copy(result![f.key] || '')">
        <span class="ip-card-label">{{ f.label }}</span>
        <code class="ip-card-value mono">{{ result[f.key] }}</code>
        <span class="ip-card-copy"><Copy :size="12" /></span>
      </div>
    </div>

    <!-- 子网掩码快速参考 -->
    <details class="cidr-table">
      <summary>CIDR 子网掩码对照表</summary>
      <div class="cidr-grid">
        <div class="cidr-row cidr-head">
          <span>CIDR</span><span>掩码</span><span>可用主机</span><span>范围</span>
        </div>
        <div v-for="r in CIDR_TABLE" :key="r.cidr" class="cidr-row" @click="useCidr(r.cidr)">
          <span class="mono">/{{ r.cidr }}</span>
          <span class="mono">{{ r.mask }}</span>
          <span class="mono">{{ r.hosts.toLocaleString() }}</span>
          <span class="mono small">{{ r.range }}</span>
        </div>
      </div>
    </details>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Network, Copy, CheckCircle2, AlertCircle } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const input = useToolState('ipcalc', 'input', '192.168.1.100/24');
const result = ref<Record<string, string> | null>(null);
const error = ref('');

const FIELDS: { key: string; label: string }[] = [
  { key: 'ip', label: 'IP 地址' },
  { key: 'cidr', label: 'CIDR' },
  { key: 'mask', label: '子网掩码' },
  { key: 'wildcard', label: '通配符掩码' },
  { key: 'network', label: '网络地址' },
  { key: 'broadcast', label: '广播地址' },
  { key: 'firstHost', label: '首个主机' },
  { key: 'lastHost', label: '末个主机' },
  { key: 'hostCount', label: '可用主机数' },
  { key: 'ipClass', label: '地址类别' },
  { key: 'isPrivate', label: '私有地址' },
  { key: 'binary', label: 'IP 二进制' },
];

function ipToInt(ip: string): number | null {
  const parts = ip.split('.');
  if (parts.length !== 4) return null;
  let n = 0;
  for (const p of parts) {
    const v = parseInt(p, 10);
    if (isNaN(v) || v < 0 || v > 255) return null;
    n = (n << 8) | v;
  }
  return n >>> 0;
}
function intToIp(n: number): string {
  return [(n >>> 24) & 255, (n >>> 16) & 255, (n >>> 8) & 255, n & 255].join('.');
}
function toBinary(n: number): string {
  return [(n >>> 24) & 255, (n >>> 16) & 255, (n >>> 8) & 255, n & 255].map((b) => b.toString(2).padStart(8, '0')).join('.');
}
function maskFromCidr(cidr: number): number {
  return cidr === 0 ? 0 : (0xffffffff << (32 - cidr)) >>> 0;
}
function classify(ipInt: number): string {
  const a = (ipInt >>> 24) & 255;
  if (a < 128) return 'A';
  if (a < 192) return 'B';
  if (a < 224) return 'C';
  if (a < 240) return 'D (组播)';
  return 'E (保留)';
}
function isPrivateIp(ipInt: number): string {
  const a = (ipInt >>> 24) & 255;
  const b = (ipInt >>> 16) & 255;
  if (a === 10) return '是 (10.0.0.0/8)';
  if (a === 172 && b >= 16 && b <= 31) return '是 (172.16.0.0/12)';
  if (a === 192 && b === 168) return '是 (192.168.0.0/16)';
  if (a === 127) return '是 (环回 127.0.0.0/8)';
  return '否 (公网)';
}

function compute() {
  error.value = '';
  result.value = null;
  const trimmed = input.value.trim();
  if (!trimmed) return;

  const [ipPart, cidrPart] = trimmed.split('/');
  const ipInt = ipToInt(ipPart?.trim() || '');
  if (ipInt === null) { error.value = 'IP 地址格式无效'; return; }

  let cidr: number;
  if (cidrPart) {
    cidr = parseInt(cidrPart.trim(), 10);
    if (isNaN(cidr) || cidr < 0 || cidr > 32) { error.value = 'CIDR 必须是 0-32'; return; }
  } else {
    // 无 CIDR 时按类别推断默认掩码
    const a = (ipInt >>> 24) & 255;
    cidr = a < 128 ? 8 : a < 192 ? 16 : 24;
  }

  const mask = maskFromCidr(cidr);
  const wildcard = (~mask) >>> 0;
  const network = (ipInt & mask) >>> 0;
  const broadcast = (network | wildcard) >>> 0;
  const hostCount = cidr >= 31 ? (cidr === 32 ? 0 : 2) : Math.pow(2, 32 - cidr) - 2;

  result.value = {
    ip: intToIp(ipInt),
    cidr: `/${cidr}`,
    mask: intToIp(mask),
    wildcard: intToIp(wildcard),
    network: intToIp(network),
    broadcast: intToIp(broadcast),
    firstHost: cidr === 32 ? intToIp(ipInt) : intToIp(network + 1),
    lastHost: cidr === 32 ? intToIp(ipInt) : intToIp(broadcast - 1),
    hostCount: hostCount.toLocaleString(),
    ipClass: classify(ipInt),
    isPrivate: isPrivateIp(ipInt),
    binary: toBinary(ipInt),
  };
}

// CIDR 对照表
const CIDR_TABLE: { cidr: number; mask: string; hosts: number; range: string }[] = [];
for (let c = 24; c <= 30; c++) {
  const mask = maskFromCidr(c);
  CIDR_TABLE.push({
    cidr: c,
    mask: intToIp(mask),
    hosts: c >= 31 ? 2 : Math.pow(2, 32 - c) - 2,
    range: c === 32 ? '1' : `${Math.pow(2, 32 - c)}`,
  });
}
// 补充常见
[8, 16, 20, 23, 25, 26, 27, 28].forEach((c) => {
  const mask = maskFromCidr(c);
  CIDR_TABLE.push({
    cidr: c,
    mask: intToIp(mask),
    hosts: c >= 31 ? 2 : Math.pow(2, 32 - c) - 2,
    range: c === 32 ? '1' : `${Math.pow(2, 32 - c)}`,
  });
});
CIDR_TABLE.sort((a, b) => b.cidr - a.cidr);

function useCidr(cidr: number) {
  const ipPart = input.value.split('/')[0]?.trim() || '192.168.1.1';
  input.value = `${ipPart}/${cidr}`;
  compute();
}

async function copy(text: string) {
  if (text) await copyToClipboard(text, '已复制 ' + text);
}

// 初始计算
compute();
</script>

<style scoped>
.ip-input-row { display: flex; align-items: flex-end; gap: 12px; margin-bottom: 18px; }
.ip-field { flex: 1; display: flex; flex-direction: column; gap: 6px; }
.ctrl-label { font-size: 12px; font-weight: 600; color: var(--xuya-text-secondary); }
.ip-input { padding: 10px 14px; font-size: 14px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); transition: border-color .12s, box-shadow .12s; }
.ip-input:focus { outline: none; border-color: var(--xuya-accent); box-shadow: 0 0 0 3px var(--xuya-accent-ring); }
.ip-result-badge { display: inline-flex; align-items: center; gap: 6px; padding: 8px 14px; font-size: 12px; font-weight: 600; border-radius: var(--xuya-radius); background: var(--xuya-input-bg); color: var(--xuya-text-tertiary); border: 1px solid var(--xuya-border); }
.ip-result-badge.valid { color: var(--xuya-success); background: var(--xuya-success-soft); border-color: transparent; }
.ip-result-badge.invalid { color: var(--xuya-danger); background: var(--xuya-danger-soft); border-color: transparent; }

.ip-error { padding: 12px 16px; background: var(--xuya-danger-soft); color: var(--xuya-danger); border-radius: var(--xuya-radius); font-size: 13px; margin-bottom: 16px; }

.ip-results { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 10px; margin-bottom: 20px; }
.ip-card { position: relative; padding: 14px 16px; background: var(--xuya-card-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius); cursor: pointer; transition: border-color .12s, transform .12s, box-shadow .12s; }
.ip-card:hover { border-color: var(--xuya-accent); transform: translateY(-1px); box-shadow: var(--xuya-shadow-hover); }
.ip-card-label { display: block; font-size: 11px; color: var(--xuya-text-tertiary); margin-bottom: 4px; }
.ip-card-value { display: block; font-size: 14px; font-weight: 600; color: var(--xuya-text); word-break: break-all; }
.ip-card-copy { position: absolute; top: 12px; right: 12px; color: var(--xuya-text-tertiary); opacity: 0; transition: opacity .12s; }
.ip-card:hover .ip-card-copy { opacity: 1; }

.cidr-table { background: var(--xuya-card-bg); border: 1px solid var(--xuya-border); border-radius: var(--xuya-radius); padding: 12px 16px; }
.cidr-table summary { cursor: pointer; font-size: 13px; font-weight: 600; color: var(--xuya-text-secondary); }
.cidr-grid { margin-top: 12px; }
.cidr-row { display: grid; grid-template-columns: 60px 130px 100px 1fr; gap: 8px; padding: 6px 10px; font-size: 12px; align-items: center; border-radius: var(--xuya-radius-sm); cursor: pointer; transition: background .1s; }
.cidr-row:hover { background: var(--xuya-input-bg); }
.cidr-row.cidr-head { font-weight: 600; color: var(--xuya-text-tertiary); font-size: 11px; cursor: default; }
.cidr-row.cidr-head:hover { background: none; }
.cidr-row .small { color: var(--xuya-text-tertiary); }
</style>
