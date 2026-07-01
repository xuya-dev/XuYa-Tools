<template>
  <ToolShell
    title="HTTP 状态码速查"
    :icon="Globe"
    description="全量 HTTP 状态码、含义与使用场景，可搜索过滤，点击复制。"
  >
    <div class="filter-row">
      <div class="search-box">
        <Search :size="15" />
        <input
          v-model="keyword"
          type="text"
          placeholder="搜索状态码或关键词，如 404 / NotFound / 重定向"
        />
      </div>
      <div class="cat-filters">
        <button
          v-for="c in CATS"
          :key="c.id"
          class="cat-chip"
          :class="{ active: activeCat === c.id }"
          @click="toggleCat(c.id)"
        >
          <span class="cat-dot" :style="{ background: c.color }"></span>
          {{ c.label }}
          <span class="cat-count">{{ countOf(c.id) }}</span>
        </button>
      </div>
    </div>

    <div class="result-count">{{ filtered.length }} 个状态码</div>

    <div class="table-head">
      <span class="th-code">状态码</span>
      <span class="th-name">名称</span>
      <span class="th-desc">含义 / 使用场景</span>
    </div>
    <div class="table-body">
      <div
        v-for="s in filtered"
        :key="s.code"
        class="table-row"
        :title="`点击复制 ${s.code}`"
        @click="copyCode(s)"
      >
        <span class="td-code" :class="s.class">
          <code>{{ s.code }}</code>
        </span>
        <span class="td-name">{{ s.name }}</span>
        <span class="td-desc">{{ s.desc }}</span>
      </div>
      <div v-if="!filtered.length" class="empty">
        <SearchX :size="28" />
        <p>没有匹配 "{{ keyword }}" 的状态码</p>
      </div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Globe, Search, SearchX } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const CATS = [
  { id: '', label: '全部', color: 'var(--xuya-text-secondary)' },
  { id: '1', label: '1xx 信息', color: 'var(--xuya-info)' },
  { id: '2', label: '2xx 成功', color: 'var(--xuya-success)' },
  { id: '3', label: '3xx 重定向', color: 'var(--xuya-accent)' },
  { id: '4', label: '4xx 客户端错误', color: 'var(--xuya-warn)' },
  { id: '5', label: '5xx 服务端错误', color: 'var(--xuya-danger)' },
];

const keyword = useToolState('httpstatus', 'keyword', '');
const activeCat = useToolState('httpstatus', 'category', '');

function toggleCat(id: string) {
  activeCat.value = activeCat.value === id ? '' : id;
}

interface Status {
  code: number;
  name: string;
  desc: string;
  class: string;
}

const STATUSES: Status[] = [
  // 1xx
  {
    code: 100,
    name: 'Continue',
    desc: '客户端应继续请求或忽略（已发送 Expect: 100-continue）。',
    class: 'c1',
  },
  {
    code: 101,
    name: 'Switching Protocols',
    desc: '服务器同意切换协议（如升级到 WebSocket）。',
    class: 'c1',
  },
  {
    code: 102,
    name: 'Processing',
    desc: '服务器已收到请求，正在处理但尚未完成（WebDAV）。',
    class: 'c1',
  },
  { code: 103, name: 'Early Hints', desc: '在最终响应前预加载资源，提升页面性能。', class: 'c1' },
  // 2xx
  { code: 200, name: 'OK', desc: '请求成功。GET 返回资源，POST 返回处理结果。', class: 'c2' },
  {
    code: 201,
    name: 'Created',
    desc: '请求成功并创建了新资源（常用于 POST / PUT）。',
    class: 'c2',
  },
  { code: 202, name: 'Accepted', desc: '请求已接收但尚未处理完成（异步任务）。', class: 'c2' },
  {
    code: 203,
    name: 'Non-Authoritative Information',
    desc: '返回的信息来自第三方而非原始服务器。',
    class: 'c2',
  },
  { code: 204, name: 'No Content', desc: '成功但无内容返回（如 DELETE 操作）。', class: 'c2' },
  { code: 205, name: 'Reset Content', desc: '成功，要求客户端重置表单/视图。', class: 'c2' },
  {
    code: 206,
    name: 'Partial Content',
    desc: '范围请求成功（Range 下载 / 分片传输）。',
    class: 'c2',
  },
  { code: 207, name: 'Multi-Status', desc: '多个操作的复合结果（WebDAV）。', class: 'c2' },
  {
    code: 208,
    name: 'Already Reported',
    desc: 'DAV 绑定已在前面列举过，避免重复（WebDAV）。',
    class: 'c2',
  },
  {
    code: 226,
    name: 'IM Used',
    desc: '服务器对资源执行了实例操作后返回（Delta 编码）。',
    class: 'c2',
  },
  // 3xx
  {
    code: 300,
    name: 'Multiple Choices',
    desc: '资源有多个表示，返回选项列表供选择。',
    class: 'c3',
  },
  { code: 301, name: 'Moved Permanently', desc: '永久重定向到新 URL，SEO 权重转移。', class: 'c3' },
  { code: 302, name: 'Found', desc: '临时重定向（URL 暂时变化，不转移权重）。', class: 'c3' },
  {
    code: 303,
    name: 'See Other',
    desc: '用 GET 请求另一个 URL 获取结果（POST 后重定向）。',
    class: 'c3',
  },
  {
    code: 304,
    name: 'Not Modified',
    desc: '资源未修改，使用缓存（配合 ETag / Last-Modified）。',
    class: 'c3',
  },
  { code: 307, name: 'Temporary Redirect', desc: '临时重定向且保持原请求方法。', class: 'c3' },
  { code: 308, name: 'Permanent Redirect', desc: '永久重定向且保持原请求方法。', class: 'c3' },
  // 4xx
  { code: 400, name: 'Bad Request', desc: '请求语法错误或参数无效，服务器无法理解。', class: 'c4' },
  {
    code: 401,
    name: 'Unauthorized',
    desc: '未认证，需提供有效凭据（如 Token / 登录）。',
    class: 'c4',
  },
  {
    code: 402,
    name: 'Payment Required',
    desc: '需要付费（保留，用于未来支付场景）。',
    class: 'c4',
  },
  { code: 403, name: 'Forbidden', desc: '已认证但无权限访问该资源。', class: 'c4' },
  { code: 404, name: 'Not Found', desc: '请求的资源不存在（URL 错误或已删除）。', class: 'c4' },
  {
    code: 405,
    name: 'Method Not Allowed',
    desc: '请求方法不被允许（如对只读资源用 POST）。',
    class: 'c4',
  },
  { code: 406, name: 'Not Acceptable', desc: '资源不符合 Accept 头要求的格式。', class: 'c4' },
  { code: 407, name: 'Proxy Authentication Required', desc: '需要代理服务器认证。', class: 'c4' },
  { code: 408, name: 'Request Timeout', desc: '请求超时，服务器等待客户端时间过长。', class: 'c4' },
  { code: 409, name: 'Conflict', desc: '请求与服务器当前状态冲突（如重复创建）。', class: 'c4' },
  { code: 410, name: 'Gone', desc: '资源已永久消失（比 404 更明确）。', class: 'c4' },
  { code: 411, name: 'Length Required', desc: '要求 Content-Length 头。', class: 'c4' },
  {
    code: 412,
    name: 'Precondition Failed',
    desc: '前置条件不满足（如 If-Match 失败）。',
    class: 'c4',
  },
  {
    code: 413,
    name: 'Payload Too Large',
    desc: '请求体超过服务器允许大小（上传限制）。',
    class: 'c4',
  },
  { code: 414, name: 'URI Too Long', desc: 'URI 过长（通常 GET 参数太多）。', class: 'c4' },
  {
    code: 415,
    name: 'Unsupported Media Type',
    desc: '请求的 Content-Type 不被支持。',
    class: 'c4',
  },
  { code: 416, name: 'Range Not Satisfiable', desc: 'Range 请求范围超出资源大小。', class: 'c4' },
  { code: 417, name: 'Expectation Failed', desc: 'Expect 头无法满足。', class: 'c4' },
  {
    code: 418,
    name: "I'm a Teapot",
    desc: '我是茶壶（RFC 2324 愚人节彩蛋，不能煮咖啡）。',
    class: 'c4',
  },
  { code: 421, name: 'Misdirected Request', desc: '请求发送到了无法响应的服务器。', class: 'c4' },
  {
    code: 422,
    name: 'Unprocessable Entity',
    desc: '格式正确但语义错误（校验失败）。',
    class: 'c4',
  },
  { code: 423, name: 'Locked', desc: '资源被锁定（WebDAV）。', class: 'c4' },
  { code: 425, name: 'Too Early', desc: '服务器不愿处理可能被重放的请求。', class: 'c4' },
  { code: 426, name: 'Upgrade Required', desc: '客户端需升级协议（如 TLS）。', class: 'c4' },
  {
    code: 428,
    name: 'Precondition Required',
    desc: '要求发送前置条件（如 If-Match）。',
    class: 'c4',
  },
  {
    code: 429,
    name: 'Too Many Requests',
    desc: '请求过于频繁，触发限流（配合 Retry-After）。',
    class: 'c4',
  },
  {
    code: 431,
    name: 'Request Header Fields Too Large',
    desc: '请求头过大，服务器拒绝处理。',
    class: 'c4',
  },
  {
    code: 451,
    name: 'Unavailable For Legal Reasons',
    desc: '因法律原因拒绝访问（如审查 / 版权）。',
    class: 'c4',
  },
  // 5xx
  {
    code: 500,
    name: 'Internal Server Error',
    desc: '服务器内部错误（代码异常 / 配置错误）。',
    class: 'c5',
  },
  { code: 501, name: 'Not Implemented', desc: '服务器不支持该请求方法 / 功能。', class: 'c5' },
  {
    code: 502,
    name: 'Bad Gateway',
    desc: '网关收到上游无效响应（反代 / 上游故障）。',
    class: 'c5',
  },
  {
    code: 503,
    name: 'Service Unavailable',
    desc: '服务暂时不可用（过载 / 维护，配合 Retry-After）。',
    class: 'c5',
  },
  { code: 504, name: 'Gateway Timeout', desc: '网关等待上游响应超时。', class: 'c5' },
  { code: 505, name: 'HTTP Version Not Supported', desc: '不支持的 HTTP 版本。', class: 'c5' },
  { code: 506, name: 'Variant Also Negotiates', desc: '内容协商存在循环引用。', class: 'c5' },
  { code: 507, name: 'Insufficient Storage', desc: '存储空间不足（WebDAV）。', class: 'c5' },
  { code: 508, name: 'Loop Detected', desc: '检测到无限循环（WebDAV）。', class: 'c5' },
  { code: 510, name: 'Not Extended', desc: '要求进一步扩展 HTTP 才能处理。', class: 'c5' },
  {
    code: 511,
    name: 'Network Authentication Required',
    desc: '需网络认证（如公共 WiFi 登录页）。',
    class: 'c5',
  },
];

function countOf(catId: string): number {
  if (!catId) return STATUSES.length;
  return STATUSES.filter((s) => String(s.code)[0] === catId).length;
}

const filtered = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  return STATUSES.filter((s) => {
    if (activeCat.value && String(s.code)[0] !== activeCat.value) return false;
    if (!kw) return true;
    return [s.code, s.name, s.desc].join(' ').toLowerCase().includes(kw);
  });
});

async function copyCode(s: Status) {
  await copyToClipboard(String(s.code), `已复制 ${s.code} ${s.name}`);
}
</script>

<style scoped>
.filter-row {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
  align-items: center;
  flex-wrap: wrap;
}
.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 200px;
  padding: 0 14px;
  height: 38px;
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  color: var(--xuya-text-tertiary);
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.search-box:focus-within {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.search-box input {
  flex: 1;
  border: none;
  background: none;
  color: var(--xuya-text);
  font-size: 13px;
  outline: none;
}
.search-box input::placeholder {
  color: var(--xuya-text-tertiary);
}
.cat-filters {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.cat-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: 99px;
  cursor: pointer;
  transition: all var(--xuya-duration-fast) var(--xuya-ease);
  white-space: nowrap;
}
.cat-chip:hover {
  color: var(--xuya-text);
}
.cat-chip.active {
  color: var(--xuya-text);
  border-color: var(--xuya-accent);
  background: var(--xuya-accent-soft);
}
.cat-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.cat-count {
  font-size: 10px;
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
}

.result-count {
  font-size: 11.5px;
  color: var(--xuya-text-tertiary);
  margin-bottom: 8px;
}

.table-head {
  display: grid;
  grid-template-columns: 80px 160px minmax(0, 1fr);
  gap: 12px;
  padding: 8px 14px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  color: var(--xuya-text-tertiary);
  border-bottom: 1px solid var(--xuya-border);
}
.table-body {
  overflow: auto;
  flex: 1;
}
.table-row {
  display: grid;
  grid-template-columns: 80px 160px minmax(0, 1fr);
  gap: 12px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--xuya-border-light);
  align-items: center;
  cursor: pointer;
  transition: background var(--xuya-duration-fast);
}
.table-row:hover {
  background: var(--xuya-input-bg);
}
.td-code code {
  font-family: var(--xuya-font-mono);
  font-size: 13px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: var(--xuya-radius-sm);
}
.c1 code {
  background: var(--xuya-info-soft);
  color: var(--xuya-info);
}
.c2 code {
  background: var(--xuya-success-soft);
  color: var(--xuya-success);
}
.c3 code {
  background: var(--xuya-accent-soft);
  color: var(--xuya-accent);
}
.c4 code {
  background: var(--xuya-warn-soft);
  color: var(--xuya-warn);
}
.c5 code {
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
}
.td-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--xuya-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.td-desc {
  font-size: 12.5px;
  color: var(--xuya-text-secondary);
  line-height: 1.5;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 50px;
  color: var(--xuya-text-tertiary);
}
.empty p {
  font-size: 13px;
}

@media (max-width: 640px) {
  .table-head {
    display: none;
  }
  .table-row {
    grid-template-columns: auto 1fr;
    grid-template-areas: 'code name' 'desc desc';
    gap: 4px 10px;
    padding: 10px;
  }
  .td-code {
    grid-area: code;
  }
  .td-name {
    grid-area: name;
    white-space: normal;
  }
  .td-desc {
    grid-area: desc;
  }
}
</style>
