<template>
  <ToolShell
    title="进制转换"
    :icon="Binary"
    description="二进制 / 八进制 / 十进制 / 十六进制 / Base36 / Base64 / ASCII 互转。"
  >
    <div class="base-grid">
      <div v-for="b in BASES" :key="b.id" class="base-card">
        <div class="base-head">
          <span class="base-label">{{ b.label }}</span>
          <span class="base-prefix">{{ b.prefix }}</span>
        </div>
        <div class="base-input-wrap">
          <input
            v-model="values[b.id]"
            class="base-input"
            :placeholder="b.placeholder"
            spellcheck="false"
            @input="onInput(b.id)"
            @focus="($event.target as HTMLInputElement).select()"
          />
          <button v-if="values[b.id]" class="copy-mini" @click="copyText(values[b.id]!)">
            <Copy :size="12" />
          </button>
        </div>
      </div>
    </div>

    <!-- 位运算面板 -->
    <div class="bitwise-section">
      <div class="section-head">
        <span class="section-label">位运算</span>
      </div>
      <div class="bitwise-grid">
        <div class="bit-input-group">
          <label class="bit-label">操作数 A (DEC)</label>
          <input v-model="bitA" class="bit-input" placeholder="0" spellcheck="false" />
        </div>
        <div class="bit-input-group">
          <label class="bit-label">操作数 B (DEC)</label>
          <input v-model="bitB" class="bit-input" placeholder="0" spellcheck="false" />
        </div>
        <div class="bit-input-group">
          <label class="bit-label">位移 N</label>
          <input
            v-model.number="shiftN"
            type="number"
            min="0"
            class="bit-input"
            placeholder="0"
          />
        </div>
      </div>
      <div class="bit-btns">
        <button class="bit-btn" @click="bitOp('and')">A &amp; B</button>
        <button class="bit-btn" @click="bitOp('or')">A | B</button>
        <button class="bit-btn" @click="bitOp('xor')">A ^ B</button>
        <button class="bit-btn" @click="bitOp('not_a')">~A</button>
        <button class="bit-btn" @click="bitOp('not_b')">~B</button>
        <button class="bit-btn" @click="bitOp('shl')">A &lt;&lt; N</button>
        <button class="bit-btn" @click="bitOp('shr')">A &gt;&gt; N</button>
      </div>
      <div v-if="bitResult !== null" class="bit-result">
        <div class="bit-result-row">
          <span class="bit-result-label">DEC</span>
          <code class="bit-result-val">{{ bitResultDec }}</code>
          <button v-if="bitResultDec" class="copy-mini" @click="copyText(bitResultDec)">
            <Copy :size="12" />
          </button>
        </div>
        <div class="bit-result-row">
          <span class="bit-result-label">HEX</span>
          <code class="bit-result-val">{{ bitResultHex }}</code>
          <button v-if="bitResultHex" class="copy-mini" @click="copyText(bitResultHex)">
            <Copy :size="12" />
          </button>
        </div>
        <div class="bit-result-row">
          <span class="bit-result-label">BIN</span>
          <code class="bit-result-val">{{ bitResultBin }}</code>
          <button v-if="bitResultBin" class="copy-mini" @click="copyText(bitResultBin)">
            <Copy :size="12" />
          </button>
        </div>
      </div>
    </div>

    <!-- ASCII 字符表 -->
    <div class="ascii-section">
      <div class="section-head">
        <span class="section-label">ASCII 字符参考</span>
        <input
          v-model="asciiSearch"
          class="ascii-search"
          type="text"
          placeholder="搜索字符或编码…"
        />
      </div>
      <div class="ascii-grid">
        <div
          v-for="c in filteredAscii"
          :key="c.code"
          class="ascii-item"
          :title="`点击复制 ${c.char}`"
          @click="copyText(c.char)"
        >
          <span class="ascii-char">{{ c.char === ' ' ? '␣' : c.char }}</span>
          <span class="ascii-dec">{{ c.code }}</span>
          <span class="ascii-hex">{{ c.code.toString(16).toUpperCase().padStart(2, '0') }}</span>
        </div>
      </div>
    </div>

    <div v-if="convError" class="error-box">{{ convError }}</div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { Binary, Copy } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';

type BaseId = 'bin' | 'oct' | 'dec' | 'hex' | 'b36' | 'b64' | 'ascii';

const BASES: { id: BaseId; label: string; prefix: string; placeholder: string }[] = [
  { id: 'dec', label: '十进制', prefix: 'DEC', placeholder: '255' },
  { id: 'hex', label: '十六进制', prefix: 'HEX', placeholder: 'FF' },
  { id: 'bin', label: '二进制', prefix: 'BIN', placeholder: '11111111' },
  { id: 'oct', label: '八进制', prefix: 'OCT', placeholder: '377' },
  { id: 'b36', label: 'Base36', prefix: 'B36', placeholder: '73' },
  { id: 'b64', label: 'Base64', prefix: 'B64', placeholder: 'MjU1' },
  { id: 'ascii', label: 'ASCII', prefix: 'CHR', placeholder: 'ÿ' },
];

const values = ref<Record<BaseId, string>>({
  dec: '',
  hex: '',
  bin: '',
  oct: '',
  b36: '',
  b64: '',
  ascii: '',
});

const convError = ref('');

function parseValue(id: BaseId, val: string): bigint | null {
  const v = val.trim();
  if (!v) return null;
  try {
    switch (id) {
      case 'dec':
        return BigInt(v);
      case 'hex':
        return BigInt('0x' + v.replace(/^0x/i, ''));
      case 'bin':
        return BigInt('0b' + v.replace(/^0b/i, ''));
      case 'oct':
        return BigInt('0o' + v.replace(/^0o/i, ''));
      case 'b36': {
        const n = BigInt(parseInt(v, 36));
        if (n < 0n) return null;
        return n;
      }
      case 'b64': {
        const decoded = atob(v);
        if (decoded.length > 8) return null;
        let n = 0n;
        for (const ch of decoded) n = (n << 8n) | BigInt(ch.charCodeAt(0));
        return n;
      }
      case 'ascii': {
        const codes = [...v].map((c) => c.charCodeAt(0));
        if (codes.length === 0) return null;
        let n = 0n;
        for (const code of codes) {
          if (code > 255) return null;
          n = (n << 8n) | BigInt(code);
        }
        return n;
      }
    }
  } catch {
    return null;
  }
}

function formatValue(id: BaseId, n: bigint): string {
  switch (id) {
    case 'dec':
      return n.toString();
    case 'hex':
      return n.toString(16).toUpperCase();
    case 'bin':
      return n.toString(2);
    case 'oct':
      return n.toString(8);
    case 'b36':
      return Number(n).toString(36).toUpperCase();
    case 'b64': {
      const bytes: number[] = [];
      let tmp = n;
      if (tmp === 0n) bytes.push(0);
      while (tmp > 0n) {
        bytes.unshift(Number(tmp & 0xffn));
        tmp >>= 8n;
      }
      return btoa(String.fromCharCode(...bytes));
    }
    case 'ascii': {
      const chars: string[] = [];
      let tmp = n;
      if (tmp === 0n) return '\0';
      while (tmp > 0n) {
        chars.unshift(String.fromCharCode(Number(tmp & 0xffn)));
        tmp >>= 8n;
      }
      return chars.join('');
    }
  }
}

function onInput(sourceId: BaseId) {
  convError.value = '';
  const val = values.value[sourceId];
  if (!val || !val.trim()) {
    for (const b of BASES) if (b.id !== sourceId) values.value[b.id] = '';
    return;
  }
  const num = parseValue(sourceId, val);
  if (num === null) {
    convError.value = `"${val}" 不是有效的 ${BASES.find((b) => b.id === sourceId)?.label}`;
    return;
  }
  for (const b of BASES) {
    if (b.id !== sourceId) {
      try {
        values.value[b.id] = formatValue(b.id, num);
      } catch {
        values.value[b.id] = '';
      }
    }
  }
}

// ASCII 表
const ASCII_TABLE = Array.from({ length: 128 }, (_, i) => {
  const char = String.fromCharCode(i);
  const display = i < 32 ? '·' : char;
  return { code: i, char: display, raw: char };
});

const asciiSearch = ref('');
const filteredAscii = computed(() => {
  const q = asciiSearch.value.trim().toLowerCase();
  if (!q) return ASCII_TABLE;
  return ASCII_TABLE.filter((c) => {
    return (
      String(c.code) === q ||
      c.code.toString(16).toLowerCase() === q ||
      c.char.toLowerCase().includes(q) ||
      c.raw.toLowerCase() === q
    );
  });
});

async function copyText(text: string) {
  if (text) await copyToClipboard(text, '已复制');
}

const bitA = ref('');
const bitB = ref('');
const shiftN = ref(0);
const bitResult = ref<bigint | null>(null);

const bitResultDec = computed(() =>
  bitResult.value === null ? '' : formatValue('dec', bitResult.value),
);
const bitResultHex = computed(() =>
  bitResult.value === null ? '' : formatValue('hex', bitResult.value),
);
const bitResultBin = computed(() =>
  bitResult.value === null ? '' : formatValue('bin', bitResult.value),
);

function parseBigIntSafe(s: string): bigint {
  const v = s.trim();
  if (!v) return 0n;
  try {
    return BigInt(v);
  } catch {
    return 0n;
  }
}

function bitOp(op: 'and' | 'or' | 'xor' | 'not_a' | 'not_b' | 'shl' | 'shr') {
  const a = parseBigIntSafe(bitA.value);
  const b = parseBigIntSafe(bitB.value);
  const n = BigInt(Math.max(0, shiftN.value));
  switch (op) {
    case 'and':
      bitResult.value = a & b;
      break;
    case 'or':
      bitResult.value = a | b;
      break;
    case 'xor':
      bitResult.value = a ^ b;
      break;
    case 'not_a':
      bitResult.value = ~a;
      break;
    case 'not_b':
      bitResult.value = ~b;
      break;
    case 'shl':
      bitResult.value = a << n;
      break;
    case 'shr':
      bitResult.value = a >> n;
      break;
  }
}
</script>

<style scoped>
.base-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 10px;
  margin-bottom: 20px;
}
.base-card {
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  padding: 12px 14px;
}
.base-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}
.base-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text);
}
.base-prefix {
  font-size: 10px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  padding: 1px 6px;
  border-radius: 99px;
  background: var(--xuya-input-bg);
}
.base-input-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
}
.base-input {
  flex: 1;
  min-width: 0;
  padding: 8px 10px;
  font-family: var(--xuya-font-mono);
  font-size: 14px;
  font-weight: 600;
  color: var(--xuya-accent);
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  outline: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.base-input:focus {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
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
  flex-shrink: 0;
}
.copy-mini:hover {
  color: var(--xuya-accent);
}

.ascii-section {
  margin-bottom: 8px;
}
.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}
.section-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.ascii-search {
  width: 160px;
  padding: 5px 10px;
  font-size: 12px;
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  outline: none;
  transition: border-color var(--xuya-duration-fast);
}
.ascii-search:focus {
  border-color: var(--xuya-accent);
}

.ascii-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(64px, 1fr));
  gap: 4px;
  max-height: 300px;
  overflow-y: auto;
}
.ascii-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 6px 4px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border-light);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.ascii-item:hover {
  border-color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
}
.ascii-char {
  font-size: 16px;
  font-weight: 700;
  color: var(--xuya-text);
}
.ascii-dec {
  font-family: var(--xuya-font-mono);
  font-size: 10px;
  color: var(--xuya-accent);
}
.ascii-hex {
  font-family: var(--xuya-font-mono);
  font-size: 9px;
  color: var(--xuya-text-tertiary);
}

.error-box {
  padding: 10px 14px;
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
  border-radius: var(--xuya-radius);
  font-size: 12.5px;
  margin-top: 14px;
}

.bitwise-section {
  margin-bottom: 20px;
  padding: 14px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
}
.bitwise-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 10px;
  margin-bottom: 12px;
}
.bit-input-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.bit-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.bit-input {
  padding: 7px 10px;
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-accent);
  border-radius: var(--xuya-radius-sm);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  outline: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.bit-input:focus {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.bit-btns {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 12px;
}
.bit-btn {
  padding: 6px 12px;
  font-family: var(--xuya-font-mono);
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.bit-btn:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
}
.bit-result {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.bit-result-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 10px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border-light);
  border-radius: var(--xuya-radius-sm);
}
.bit-result-label {
  font-size: 10px;
  font-weight: 700;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
  width: 32px;
  flex-shrink: 0;
}
.bit-result-val {
  flex: 1;
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text);
  word-break: break-all;
}
</style>
