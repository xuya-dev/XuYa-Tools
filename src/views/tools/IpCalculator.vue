<template>
  <ToolShell
    title="IP / 子网计算器"
    :icon="Network"
    description="查询公网 IP 与本机 IP，IPv4 子网划分、CIDR、掩码、主机数计算。"
  >
    <!-- 我的 IP -->
    <div class="myip-section">
      <div class="myip-head">
        <span class="section-label">我的 IP</span>
        <button class="mini-btn" :disabled="ipLoading" @click="loadMyIp">
          <RefreshCw :size="13" :class="{ spin: ipLoading }" />
          刷新
        </button>
      </div>

      <div class="myip-grid">
        <!-- 公网 IP -->
        <div class="myip-card public">
          <div class="myip-card-head">
            <span class="myip-label">公网 IP</span>
          </div>

          <!-- 直连 IP (Rust 后端 → 国内 API) -->
          <div v-if="directIp" class="ip-block">
            <div class="ip-line">
              <span class="ip-tag direct">直连</span>
              <code class="myip-value">{{ directIp.ip }}</code>
              <button class="copy-mini" @click="copyText(directIp.ip)"><Copy :size="12" /></button>
            </div>
            <div class="myip-detail">
              <span v-if="directIp.city">{{ directIp.city }}, {{ directIp.country }}</span>
              <span v-if="directIp.org">{{ directIp.org }}</span>
            </div>
          </div>

          <!-- 代理出口 IP (浏览器 → 国际 API, 仅当与直连不同时显示) -->
          <div v-if="showProxy && proxyIp" class="ip-block proxy-block">
            <div class="ip-line">
              <span class="ip-tag proxy">代理出口</span>
              <code class="myip-value proxy">{{ proxyIp.ip }}</code>
              <button class="copy-mini" @click="copyText(proxyIp.ip)"><Copy :size="12" /></button>
            </div>
            <div class="myip-detail">
              <span v-if="proxyIp.city">{{ proxyIp.city }}, {{ proxyIp.country }}</span>
              <span v-if="proxyIp.org">{{ proxyIp.org }}</span>
            </div>
          </div>

          <span v-if="!directIp && !proxyIp && !ipLoading" class="myip-ph">点击刷新获取</span>
          <span v-else-if="ipLoading && !directIp" class="myip-ph">查询中…</span>
        </div>

        <!-- 本机 IP -->
        <div class="myip-card local">
          <div class="myip-card-head">
            <span class="myip-label">本机 IP</span>
          </div>
          <div v-if="localIps.length" class="local-list">
            <div
              v-for="(ip, i) in localIps"
              :key="i"
              class="local-item"
              @click="copyText(ip.address)"
            >
              <span class="local-name">{{ ip.name }}</span>
              <code class="local-ip">{{ ip.address }}</code>
              <span class="local-mask">{{ ip.netmask }}</span>
              <span class="local-type" :class="ip.type">{{ ip.type }}</span>
            </div>
          </div>
          <span v-else-if="ipLoading" class="myip-ph">检测中…</span>
          <span v-else class="myip-ph">点击刷新获取</span>
        </div>
      </div>
    </div>

    <!-- 子网计算器 -->
    <div class="calc-section">
      <div class="section-label">子网计算器</div>
      <div class="ip-input-row">
        <div class="ip-field">
          <input
            v-model="input"
            class="ip-input"
            type="text"
            placeholder="192.168.1.100/24"
            spellcheck="false"
            @input="compute"
          />
        </div>
        <div class="ip-result-badge" :class="{ valid: result, invalid: error }">
          <CheckCircle2 v-if="result" :size="16" />
          <AlertCircle v-else-if="error" :size="16" />
          {{ error ? '无效' : result ? '有效' : '输入 IP/CIDR' }}
        </div>
      </div>

      <div v-if="error" class="ip-error">{{ error }}</div>

      <div v-if="result" class="ip-results">
        <div
          v-for="f in FIELDS"
          :key="f.key"
          class="ip-card"
          @click="copyText(result![f.key] || '')"
        >
          <span class="ip-card-label">{{ f.label }}</span>
          <code class="ip-card-value">{{ result[f.key] }}</code>
          <span class="ip-card-copy"><Copy :size="12" /></span>
        </div>
      </div>
    </div>

    <!-- CIDR 对照表 -->
    <details class="cidr-table">
      <summary>CIDR 子网掩码对照表</summary>
      <div class="cidr-grid">
        <div class="cidr-row cidr-head">
          <span>CIDR</span>
          <span>掩码</span>
          <span>可用主机</span>
          <span>总地址</span>
        </div>
        <div v-for="r in CIDR_TABLE" :key="r.cidr" class="cidr-row" @click="useCidr(r.cidr)">
          <span class="mono">/{{ r.cidr }}</span>
          <span class="mono">{{ r.mask }}</span>
          <span class="mono">{{ r.hosts.toLocaleString() }}</span>
          <span class="mono small">{{ r.total.toLocaleString() }}</span>
        </div>
      </div>
    </details>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Network, Copy, CheckCircle2, AlertCircle, RefreshCw } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';
import { httpRequest } from '@/composables/useHttpTool';

// ============ 我的 IP ============
interface IpInfo {
  ip: string;
  city?: string;
  country?: string;
  org?: string;
  timezone?: string;
}
interface LocalIp {
  address: string;
  type: string;
  name: string;
  netmask: string;
}

const ipLoading = ref(false);
const directIp = ref<IpInfo | null>(null);
const proxyIp = ref<IpInfo | null>(null);
const localIps = ref<LocalIp[]>([]);
const showProxy = computed(() => proxyIp.value && proxyIp.value.ip !== directIp.value?.ip);

function classifyIp(ip: string): string {
  const parts = ip.split('.').map(Number);
  const [a, b] = parts;
  if (a === 127) return 'Loopback';
  if (a === 10) return 'Private';
  if (a === 172 && b! >= 16 && b! <= 31) return 'Private';
  if (a === 192 && b === 168) return 'Private';
  if (a === 169 && b === 254) return 'Link-Local';
  return 'Public';
}

async function fetchDirectIp(): Promise<IpInfo | null> {
  try {
    const res = await httpRequest({
      method: 'GET',
      url: 'https://myip.ipip.net',
      headers: [{ key: 'User-Agent', value: 'curl/7.88.0', enabled: true }],
      query: [],
      body: null,
      bodyType: null,
      timeoutMs: 10000,
    });
    if (!res.error && res.body) {
      const text = res.body;
      const ipMatch = text.match(/IP[：:]\s*([\d.]+)/);
      const locMatch = text.match(/来自于[：:]\s*(.+)/);
      if (ipMatch) {
        const loc = locMatch?.[1]?.trim() ?? '';
        const parts = loc.split(/\s+/).filter(Boolean);
        return {
          ip: ipMatch[1]!,
          country: parts[0] ?? '',
          city: parts[2] ?? parts[1] ?? '',
          org: parts.slice(3).join(' ') || (parts[parts.length - 1] ?? ''),
        };
      }
    }
  } catch {
    /* fallback below */
  }
  try {
    const res2 = await httpRequest({
      method: 'GET',
      url: 'http://ip-api.com/json/?lang=zh-CN&fields=status,query,country,regionName,city,isp,timezone',
      headers: [],
      query: [],
      body: null,
      bodyType: null,
      timeoutMs: 10000,
    });
    if (!res2.error && res2.body) {
      const d = JSON.parse(res2.body);
      if (d.status === 'success') {
        return {
          ip: d.query,
          city: d.city,
          country: d.country,
          org: d.isp,
          timezone: d.timezone,
        };
      }
    }
  } catch {
    /* give up */
  }
  return null;
}

async function fetchProxyIp(): Promise<IpInfo | null> {
  try {
    const res = await httpRequest({
      method: 'GET',
      url: 'https://ipinfo.io/json',
      headers: [],
      query: [],
      body: null,
      bodyType: null,
      timeoutMs: 10000,
    });
    if (!res.error && res.body) {
      const d = JSON.parse(res.body);
      return { ip: d.ip, city: d.city, country: d.country, org: d.org, timezone: d.timezone };
    }
  } catch {
    /* fallback */
  }
  try {
    const resp = await fetch('https://ipinfo.io/json');
    if (resp.ok) {
      const d = await resp.json();
      return { ip: d.ip, city: d.city, country: d.country, org: d.org, timezone: d.timezone };
    }
  } catch {
    /* ignore */
  }
  return null;
}

async function fetchLocalIps(): Promise<LocalIp[]> {
  try {
    const interfaces = await invoke<
      {
        name: string;
        ip: string;
        netmask: string;
        broadcast: string | null;
        is_loopback: boolean;
      }[]
    >('get_local_interfaces');
    return interfaces.map((iface) => ({
      address: iface.ip,
      type: iface.is_loopback ? 'Loopback' : classifyIp(iface.ip),
      name: iface.name,
      netmask: iface.netmask,
    }));
  } catch {
    return [];
  }
}

async function loadMyIp() {
  ipLoading.value = true;
  const [direct, proxy, local] = await Promise.all([
    fetchDirectIp(),
    fetchProxyIp(),
    fetchLocalIps(),
  ]);
  directIp.value = direct;
  proxyIp.value = proxy;
  localIps.value = local;
  ipLoading.value = false;
}

onMounted(loadMyIp);

// ============ 子网计算 ============
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
  return [(n >>> 24) & 255, (n >>> 16) & 255, (n >>> 8) & 255, n & 255]
    .map((b) => b.toString(2).padStart(8, '0'))
    .join('.');
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
  if (a === 127) return '是 (环回)';
  return '否 (公网)';
}

function compute() {
  error.value = '';
  result.value = null;
  const trimmed = input.value.trim();
  if (!trimmed) return;
  const [ipPart, cidrPart] = trimmed.split('/');
  const ipInt = ipToInt(ipPart?.trim() || '');
  if (ipInt === null) {
    error.value = 'IP 地址格式无效';
    return;
  }
  let cidr: number;
  if (cidrPart) {
    cidr = parseInt(cidrPart.trim(), 10);
    if (isNaN(cidr) || cidr < 0 || cidr > 32) {
      error.value = 'CIDR 必须是 0-32';
      return;
    }
  } else {
    const a = (ipInt >>> 24) & 255;
    cidr = a < 128 ? 8 : a < 192 ? 16 : 24;
  }
  const mask = maskFromCidr(cidr);
  const wildcard = ~mask >>> 0;
  const network = (ipInt & mask) >>> 0;
  const broadcast = (network | wildcard) >>> 0;
  const total = cidr >= 32 ? 1 : Math.pow(2, 32 - cidr);
  const hostCount = cidr >= 31 ? (cidr === 32 ? 0 : 2) : total - 2;
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

const CIDR_TABLE: { cidr: number; mask: string; hosts: number; total: number }[] = [];
const CIDR_VALUES = [8, 12, 16, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
for (const c of CIDR_VALUES) {
  const total = c >= 32 ? 1 : Math.pow(2, 32 - c);
  CIDR_TABLE.push({
    cidr: c,
    mask: intToIp(maskFromCidr(c)),
    hosts: c >= 31 ? (c === 32 ? 0 : 2) : total - 2,
    total,
  });
}

function useCidr(cidr: number) {
  const ipPart = input.value.split('/')[0]?.trim() || '192.168.1.1';
  input.value = `${ipPart}/${cidr}`;
  compute();
}

async function copyText(text: string) {
  if (text) await copyToClipboard(text, '已复制 ' + text);
}

compute();
</script>

<style scoped>
.section-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--xuya-text-tertiary);
}

/* 我的 IP */
.myip-section {
  margin-bottom: 24px;
}
.myip-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}
.mini-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  font-size: 11px;
  color: var(--xuya-text-secondary);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.mini-btn:hover:not(:disabled) {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}
.mini-btn:disabled {
  opacity: 0.5;
}
.copy-mini {
  width: 22px;
  height: 22px;
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

.myip-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}
.myip-card {
  padding: 14px 16px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.myip-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}
.myip-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.myip-value {
  font-family: var(--xuya-font-mono);
  font-size: 18px;
  font-weight: 700;
  color: var(--xuya-accent);
}
.ip-block {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.ip-block.proxy-block {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px dashed var(--xuya-border);
}
.ip-line {
  display: flex;
  align-items: center;
  gap: 8px;
}
.ip-tag {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 99px;
  flex-shrink: 0;
}
.ip-tag.direct {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.ip-tag.proxy {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}
.myip-value.proxy {
  font-size: 16px;
  color: var(--xuya-warn);
}
.myip-ph {
  font-size: 13px;
  color: var(--xuya-text-tertiary);
}
.myip-detail {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-top: 8px;
  font-size: 11.5px;
  color: var(--xuya-text-secondary);
}
.local-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.local-item {
  display: grid;
  grid-template-columns: 80px 1fr auto auto;
  gap: 8px;
  align-items: center;
  cursor: pointer;
  padding: 5px 0;
  transition: color var(--xuya-duration-fast);
}
.local-item:hover {
  color: var(--xuya-accent);
}
.local-name {
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.local-ip {
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
}
.local-mask {
  font-family: var(--xuya-font-mono);
  font-size: 11px;
  color: var(--xuya-text-tertiary);
}
.local-type {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 99px;
}
.local-type.Private {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.local-type.Loopback {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}
.local-type.Public {
  background: var(--xuya-info-soft);
  color: var(--xuya-info);
}
.local-type.Link-Local {
  background: var(--xuya-border);
  color: var(--xuya-text-secondary);
}

/* 子网计算 */
.calc-section {
  margin-bottom: 20px;
}
.calc-section .section-label {
  display: block;
  margin-bottom: 10px;
}
.ip-input-row {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  margin-bottom: 16px;
}
.ip-field {
  flex: 1;
}
.ip-input {
  width: 100%;
  padding: 10px 14px;
  font-family: var(--xuya-font-mono);
  font-size: 14px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.ip-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.ip-result-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  font-size: 12px;
  font-weight: 600;
  border-radius: var(--xuya-radius);
  background: var(--xuya-input-bg);
  color: var(--xuya-text-tertiary);
  border: 1px solid var(--xuya-border);
  white-space: nowrap;
}
.ip-result-badge.valid {
  color: var(--xuya-success);
  background: var(--xuya-success-soft);
  border-color: transparent;
}
.ip-result-badge.invalid {
  color: var(--xuya-danger);
  background: var(--xuya-danger-soft);
  border-color: transparent;
}

.ip-error {
  padding: 12px 16px;
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
  border-radius: var(--xuya-radius);
  font-size: 13px;
  margin-bottom: 16px;
}

.ip-results {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 10px;
}
.ip-card {
  position: relative;
  padding: 12px 14px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  cursor: pointer;
  transition:
    border-color var(--xuya-duration-fast),
    transform var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.ip-card:hover {
  border-color: var(--xuya-accent);
  transform: translateY(-1px);
  box-shadow: var(--xuya-shadow-hover);
}
.ip-card-label {
  display: block;
  font-size: 11px;
  color: var(--xuya-text-tertiary);
  margin-bottom: 4px;
}
.ip-card-value {
  display: block;
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
  word-break: break-all;
}
.ip-card-copy {
  position: absolute;
  top: 10px;
  right: 10px;
  color: var(--xuya-text-tertiary);
  opacity: 0;
  transition: opacity var(--xuya-duration-fast);
}
.ip-card:hover .ip-card-copy {
  opacity: 1;
}

.cidr-table {
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  padding: 12px 16px;
}
.cidr-table summary {
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.cidr-grid {
  margin-top: 12px;
}
.cidr-table {
  overflow-x: auto;
}
.cidr-grid {
  min-width: 340px;
}
.cidr-row {
  display: grid;
  grid-template-columns: 60px 130px 100px minmax(0, 1fr);
  gap: 8px;
  padding: 6px 10px;
  font-size: 12px;
  align-items: center;
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: background var(--xuya-duration-fast);
}
.cidr-row:hover {
  background: var(--xuya-input-bg);
}
.cidr-row.cidr-head {
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  font-size: 11px;
  cursor: default;
}
.cidr-row.cidr-head:hover {
  background: none;
}
.mono {
  font-family: var(--xuya-font-mono);
}
.small {
  color: var(--xuya-text-tertiary);
}

.spin {
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 600px) {
  .myip-grid {
    grid-template-columns: 1fr;
  }
  .ip-input-row {
    flex-wrap: wrap;
  }
  .local-item {
    grid-template-columns: 60px minmax(0, 1fr) auto;
  }
  .local-mask {
    display: none;
  }
}
</style>
