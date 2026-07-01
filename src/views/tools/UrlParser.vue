<template>
  <ToolShell
    title="URL 解析"
    :icon="Link"
    description="解析 URL 各组成部分，编辑查询参数，实时重建。"
  >
    <div class="url-input-row">
      <input
        v-model="rawUrl"
        class="url-input"
        type="text"
        placeholder="https://example.com/path?key=value"
        spellcheck="false"
      />
      <BaseButton variant="ghost" :disabled="!rawUrl" @click="copyUrl">
        <Copy :size="13" />
      </BaseButton>
    </div>

      <div v-if="parseError" class="error-box">{{ parseError }}</div>

    <template v-if="parsed && !parseError">
      <!-- URL 组成部分 -->
      <div class="parts-grid">
        <div
          v-for="f in URL_FIELDS"
          :key="f.key"
          class="part-card"
          title="点击复制"
          @click="copyText(String(parsed[f.key] || ''))"
        >
          <span class="part-label">
            {{ f.label }}
            <Copy :size="10" class="copy-hint" />
          </span>
          <code class="part-value" :class="{ empty: !parsed[f.key] }">
            {{ parsed[f.key] || '—' }}
          </code>
        </div>
      </div>

      <!-- 路径分段 -->
      <div v-if="pathSegments.length" class="path-segments">
        <span class="section-label">路径分段 ({{ pathSegments.length }})</span>
        <div class="chip-list">
          <code
            v-for="(seg, i) in pathSegments"
            :key="i"
            class="path-chip"
            @click="copyText(decodeURIComponent(seg))"
          >
            {{ decodeURIComponent(seg) }}
          </code>
        </div>
      </div>

      <!-- 查询参数编辑 -->
      <div v-if="parsed.search" class="query-section">
        <div class="section-head">
          <span class="section-label">查询参数 ({{ queryParams.length }})</span>
          <div class="head-actions">
            <label class="toggle-label">
              <input
                v-model="encodeParams"
                type="checkbox"
                class="toggle-check"
                @change="rebuildUrl"
              />
              URL 编码值
            </label>
            <button class="mini-btn" @click="addParam">
              <Plus :size="13" />
              添加
            </button>
          </div>
        </div>
        <div class="param-list">
          <div v-for="(p, i) in queryParams" :key="i" class="param-row">
            <input
              v-model="p.key"
              class="param-input"
              placeholder="参数名"
              spellcheck="false"
              @input="rebuildUrl"
            />
            <span class="param-eq">=</span>
            <input
              v-model="p.value"
              class="param-input"
              placeholder="值"
              spellcheck="false"
              @input="rebuildUrl"
            />
            <button
              class="param-del"
              @click="
                removeParam(i);
                rebuildUrl();
              "
            >
              <X :size="14" />
            </button>
          </div>
        </div>
      </div>

      <!-- 重建 URL -->
      <div class="rebuild-section">
        <div class="section-head">
          <span class="section-label">重建 URL</span>
          <button class="mini-btn" @click="copyRebuilt">
            <Copy :size="13" />
            复制
          </button>
        </div>
        <code class="rebuilt-url" @click="copyRebuilt">{{ rebuiltUrl || rawUrl }}</code>
      </div>

      <!-- 快捷操作 -->
      <div class="actions-row">
        <BaseButton variant="ghost" @click="encodeUrl">
          <ArrowUpRight :size="13" />
          编码全文
        </BaseButton>
        <BaseButton variant="ghost" @click="decodeUrl">
          <ArrowDownRight :size="13" />
          解码全文
        </BaseButton>
        <BaseButton variant="ghost" @click="copyText(rawUrl)">复制原始</BaseButton>
      </div>
    </template>

    <div v-else-if="!parseError && !rawUrl" class="placeholder">
      <Link :size="40" />
      <p>输入 URL 后自动解析各组成部分</p>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Link, Copy, Plus, X, ArrowUpRight, ArrowDownRight } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const rawUrl = useToolState('urlparser', 'input', '');
const parseError = ref('');

const URL_FIELDS = [
  { key: 'protocol' as const, label: '协议' },
  { key: 'hostname' as const, label: '主机' },
  { key: 'port' as const, label: '端口' },
  { key: 'pathname' as const, label: '路径' },
  { key: 'hash' as const, label: '锚点' },
  { key: 'username' as const, label: '用户名' },
  { key: 'password' as const, label: '密码' },
];

interface ParsedUrl {
  protocol: string;
  hostname: string;
  port: string;
  pathname: string;
  hash: string;
  username: string;
  password: string;
  search: string;
}

const parsed = ref<ParsedUrl | null>(null);

interface QueryParam {
  key: string;
  value: string;
}
const queryParams = ref<QueryParam[]>([]);
const rebuiltUrl = ref('');
const encodeParams = ref(true);

const pathSegments = computed(() => {
  if (!parsed.value?.pathname) return [];
  return parsed.value.pathname.split('/').filter(Boolean);
});

function parseUrl() {
  parseError.value = '';
  parsed.value = null;
  const url = rawUrl.value.trim();
  if (!url) return;
  try {
    const u = new URL(url);
    parsed.value = {
      protocol: u.protocol,
      hostname: u.hostname,
      port: u.port,
      pathname: u.pathname,
      hash: u.hash,
      username: u.username,
      password: u.password,
      search: u.search,
    };
    queryParams.value = [...u.searchParams.entries()].map(([k, v]) => ({ key: k, value: v }));
    rebuildUrl();
  } catch {
    parseError.value = '无效的 URL，请确保包含协议 (https://...)';
  }
}

function rebuildUrl() {
  if (!parsed.value) return;
  try {
    const base = `${parsed.value.protocol}//${parsed.value.hostname}${parsed.value.port ? ':' + parsed.value.port : ''}${parsed.value.pathname || '/'}`;
    const validParams = queryParams.value.filter((p) => p.key.trim());
    let qs: string;
    if (encodeParams.value) {
      const params = new URLSearchParams();
      for (const p of validParams) params.append(p.key.trim(), p.value);
      qs = params.toString();
    } else {
      qs = validParams.map((p) => `${p.key.trim()}=${p.value}`).join('&');
    }
    rebuiltUrl.value = base + (qs ? '?' + qs : '') + (parsed.value.hash || '');
  } catch {
    rebuiltUrl.value = rawUrl.value;
  }
}

function addParam() {
  queryParams.value.push({ key: '', value: '' });
}

function removeParam(index: number) {
  queryParams.value.splice(index, 1);
}

function encodeUrl() {
  try {
    rawUrl.value = encodeURIComponent(rawUrl.value);
  } catch {
    /* ignore */
  }
}

function decodeUrl() {
  try {
    rawUrl.value = decodeURIComponent(rawUrl.value);
  } catch {
    /* ignore */
  }
}

watch(rawUrl, parseUrl, { immediate: true });

async function copyText(text: string) {
  if (text) await copyToClipboard(text, '已复制');
}
async function copyUrl() {
  await copyText(rawUrl.value);
}
async function copyRebuilt() {
  if (rebuiltUrl.value) await copyToClipboard(rebuiltUrl.value, '已复制 URL');
}
</script>

<style scoped>
.url-input-row {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}
.url-input {
  flex: 1;
  padding: 10px 14px;
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.url-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

.error-box {
  padding: 12px 16px;
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
  border-radius: var(--xuya-radius);
  font-size: 13px;
  margin-bottom: 16px;
}

.parts-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 10px;
  margin-bottom: 18px;
}
.part-card {
  padding: 12px 14px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  cursor: pointer;
  transition:
    border-color var(--xuya-duration-fast),
    transform var(--xuya-duration-fast);
}
.part-card:hover {
  border-color: var(--xuya-accent);
  transform: translateY(-1px);
}
.part-label {
  display: block;
  font-size: 10.5px;
  color: var(--xuya-text-tertiary);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  margin-bottom: 4px;
}
.part-value {
  display: block;
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
  word-break: break-all;
}
.part-value.empty {
  color: var(--xuya-text-tertiary);
  font-weight: 400;
}
.copy-hint {
  opacity: 0;
  transition: opacity var(--xuya-duration-fast);
}
.part-card:hover .copy-hint {
  opacity: 0.6;
}

.path-segments {
  margin-bottom: 18px;
}
.chip-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
  align-items: center;
}
.path-chip {
  padding: 4px 10px;
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
  border: 1px solid var(--xuya-accent-soft-strong);
  border-radius: 99px;
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.path-chip:hover {
  background: var(--xuya-accent);
  color: var(--xuya-on-accent);
}

.head-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}
.toggle-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--xuya-text-secondary);
  cursor: pointer;
  user-select: none;
}
.toggle-check {
  accent-color: var(--xuya-accent);
  cursor: pointer;
}

.query-section {
  margin-bottom: 18px;
}
.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}
.section-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.param-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.param-row {
  display: flex;
  align-items: center;
  gap: 6px;
}
.param-input {
  flex: 1;
  min-width: 0;
  padding: 6px 10px;
  font-size: 12.5px;
  font-family: var(--xuya-font-mono);
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  transition: border-color var(--xuya-duration-fast);
}
.param-input:focus {
  outline: none;
  border-color: var(--xuya-accent);
}
.param-eq {
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
}
.param-del {
  width: 28px;
  height: 28px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: var(--xuya-radius-sm);
  color: var(--xuya-text-tertiary);
  background: transparent;
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.param-del:hover {
  color: var(--xuya-danger);
  background: var(--xuya-danger-soft);
}

.rebuild-section {
  margin-bottom: 16px;
}
.rebuilt-url {
  display: block;
  padding: 12px 14px;
  background: var(--xuya-accent-soft);
  border: 1px solid var(--xuya-accent-soft-strong);
  border-radius: var(--xuya-radius);
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-accent);
  word-break: break-all;
  cursor: pointer;
}

.actions-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
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
.mini-btn:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}

.placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 0;
  color: var(--xuya-text-tertiary);
}
.placeholder p {
  font-size: 14px;
}
</style>
