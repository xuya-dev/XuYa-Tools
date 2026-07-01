/**
 * 工具注册表 —— 工具箱的单一数据源。
 * 新增工具只需:1) 在此添加一条记录 2) 在 router/index.ts 加一条路由 3) 实现 view 组件。
 */
import type { Component } from 'vue';
import {
  Braces,
  Binary,
  KeyRound,
  Clock,
  Fingerprint,
  Terminal,
  Lock,
  Regex,
  Palette,
  FileText,
  QrCode,
  ScanQrCode,
  GitCompare,
  ListTree,
  Globe,
  Network,
  Send,
  Webhook,
  Database,
  ArrowLeftRight,
  Link,
  Hash,
  Image,
  Type,
} from '@lucide/vue';

export type ToolCategory =
  'converter' | 'encoder' | 'crypto' | 'datetime' | 'devops' | 'text' | 'network';

export interface ToolMeta {
  id: string;
  name: string;
  description: string;
  category: ToolCategory;
  icon: Component;
  route: string;
  /** 搜索关键词 (含中英文别名) */
  keywords: string[];
}

export const CATEGORY_LABELS: Record<ToolCategory, string> = {
  converter: '格式转换',
  encoder: '编码生成',
  crypto: '加密解析',
  datetime: '日期时间',
  devops: '开发配置',
  text: '文本处理',
  network: '网络参考',
};

export const CATEGORY_COLORS: Record<ToolCategory, string> = {
  converter: '#2dd4bf',
  encoder: '#fbbf24',
  crypto: '#fb7185',
  datetime: '#38bdf8',
  devops: '#c4b5fd',
  text: '#a78bfa',
  network: '#34d399',
};

export const tools: ToolMeta[] = [
  // ---- 格式转换 ----
  {
    id: 'converter',
    name: '格式互转',
    description: 'JSON ↔ YAML / CSV / XML / TOML / Properties 双向转换。',
    category: 'converter',
    icon: ArrowLeftRight,
    route: '/tools/converter',
    keywords: ['json', 'yaml', 'csv', 'xml', 'toml', 'properties', '转换', '互转', '格式'],
  },
  {
    id: 'json',
    name: 'JSON 格式化',
    description: '美化、压缩、转义、反转义 JSON,带错误定位与字符统计。',
    category: 'converter',
    icon: Braces,
    route: '/tools/json',
    keywords: ['json', 'format', 'beautify', 'minify', '美化', '压缩', '格式化'],
  },
  {
    id: 'regex',
    name: '正则测试',
    description: '实时匹配高亮、捕获组提取、标志位切换、常用速查。',
    category: 'converter',
    icon: Regex,
    route: '/tools/regex',
    keywords: ['regex', 'regexp', 'regular expression', '正则', '匹配', 'pattern'],
  },
  {
    id: 'diff',
    name: '差异对比',
    description: '双栏输入,行级 diff 高亮,统计增删改。',
    category: 'converter',
    icon: GitCompare,
    route: '/tools/diff',
    keywords: ['diff', 'compare', '差异', '对比', '比较'],
  },
  {
    id: 'markdown',
    name: 'MD 预览',
    description: '实时分栏:左侧编辑,右侧预览,支持代码块与表格。',
    category: 'converter',
    icon: FileText,
    route: '/tools/markdown',
    keywords: ['markdown', 'md', 'preview', '预览', '文档', '渲染'],
  },
  {
    id: 'sql',
    name: 'SQL 格式化',
    description: '美化 / 压缩 SQL,关键字大写,支持多方言,智能缩进。',
    category: 'converter',
    icon: Database,
    route: '/tools/sql',
    keywords: ['sql', 'format', 'beautify', 'mysql', 'postgres', '格式化', '美化', '查询'],
  },
  // ---- 编码生成 ----
  {
    id: 'urlparser',
    name: 'URL 解析',
    description: '解析 URL 各组成部分，编辑查询参数，实时重建。',
    category: 'encoder',
    icon: Link,
    route: '/tools/urlparser',
    keywords: ['url', 'uri', 'parse', 'query', 'param', '解析', '链接', '参数'],
  },
  {
    id: 'baseconv',
    name: '进制转换',
    description: '二进制 / 八进制 / 十进制 / 十六进制 / Base36 / Base64 / ASCII 互转。',
    category: 'encoder',
    icon: Hash,
    route: '/tools/baseconv',
    keywords: ['binary', 'hex', 'decimal', 'octal', 'base36', 'base64', 'ascii', '进制', '转换'],
  },
  {
    id: 'encoding',
    name: '编码转换',
    description: 'Base64 / URL / HTML 实体 / Hex / Binary 双向转换。',
    category: 'encoder',
    icon: Binary,
    route: '/tools/encoding',
    keywords: ['base64', 'url', 'html', 'hex', 'binary', '编码', '解码', '转义'],
  },
  {
    id: 'imgtool',
    name: '图片工具',
    description: '图片压缩、缩放、格式转换 (PNG/JPG/WebP)、Base64 互转。',
    category: 'encoder',
    icon: Image,
    route: '/tools/imgtool',
    keywords: ['image', 'compress', 'resize', 'convert', 'webp', 'base64', '图片', '压缩', '缩放'],
  },
  {
    id: 'color',
    name: '颜色工具',
    description: 'HEX / RGB / HSL 互转、色板生成、WCAG 对比度检测。',
    category: 'encoder',
    icon: Palette,
    route: '/tools/color',
    keywords: ['color', 'hex', 'rgb', 'hsl', '颜色', '色板', '对比度', 'palette'],
  },
  {
    id: 'qrcode',
    name: 'QR 码生成',
    description: '文本 / URL 转 QR 码,可调尺寸与纠错级别,下载 PNG。',
    category: 'encoder',
    icon: QrCode,
    route: '/tools/qrcode',
    keywords: ['qrcode', 'qr', '二维码', '条码', 'barcode'],
  },
  {
    id: 'qrdecode',
    name: 'QR 码解码',
    description: '上传二维码图片,自动识别并提取内容,支持拖拽粘贴。',
    category: 'encoder',
    icon: ScanQrCode,
    route: '/tools/qrdecode',
    keywords: ['qrcode', 'qr', 'decode', 'scan', '二维码', '解码', '识别', '扫码'],
  },
  // ---- 加密 / 解析 ----
  {
    id: 'jwt',
    name: 'JWT 解析',
    description: '解码 JWT 的 Header / Payload / Signature,查看算法与过期时间。',
    category: 'crypto',
    icon: KeyRound,
    route: '/tools/jwt',
    keywords: ['jwt', 'token', 'json web token', '解析', '解码'],
  },
  {
    id: 'idgen',
    name: 'ID 生成器',
    description: '生成 UUID、雪花 ID、随机密码，支持批量与自定义选项。',
    category: 'crypto',
    icon: Fingerprint,
    route: '/tools/idgen',
    keywords: ['uuid', 'snowflake', 'password', '雪花', '密码', '随机', 'id'],
  },
  {
    id: 'crypto',
    name: '加密解密',
    description: 'MD5 / SHA / SM3 哈希摘要，AES / SM4 对称加密，SM2 非对称加密。',
    category: 'crypto',
    icon: Lock,
    route: '/tools/crypto',
    keywords: ['md5', 'sha', 'aes', 'sm2', 'sm3', 'sm4', '国密', '加密', '解密', 'hash'],
  },
  {
    id: 'keygen',
    name: '密钥生成',
    description: '生成 RSA / ECDSA / Ed25519 密钥对，导出 PEM / JWK 格式。',
    category: 'crypto',
    icon: KeyRound,
    route: '/tools/keygen',
    keywords: ['rsa', 'ecdsa', 'ed25519', 'keypair', '公钥', '私钥', '密钥对', 'ssl', 'pem'],
  },
  // ---- 文本处理 ----
  {
    id: 'textproc',
    name: '文本处理',
    description: '大小写转换、去重、排序、提取、统计，一键处理文本。',
    category: 'text',
    icon: Type,
    route: '/tools/textproc',
    keywords: ['text', 'case', 'upper', 'lower', 'camel', 'snake', 'kebab', 'sort', 'dedup', 'extract', '文本', '大小写', '排序', '去重', '统计'],
  },
  // ---- 日期时间 ----
  {
    id: 'timestamp',
    name: '时间戳转换',
    description: 'Unix 时间戳与可读时间互转,支持秒 / 毫秒,实时当前时间。',
    category: 'datetime',
    icon: Clock,
    route: '/tools/timestamp',
    keywords: ['timestamp', 'unix', 'epoch', '时间戳', '日期', 'date'],
  },
  {
    id: 'cron',
    name: 'Cron 解析',
    description: '解析 Cron 五段表达式,人类可读描述,未来执行时间预览。',
    category: 'datetime',
    icon: ListTree,
    route: '/tools/cron',
    keywords: ['cron', 'crontab', 'schedule', '定时', '计划任务', '表达式'],
  },
  // ---- 网络参考 ----
  {
    id: 'http',
    name: 'HTTP 请求',
    description: '发送 HTTP 请求测试接口,支持 Headers / Query / Body,完整展示响应。',
    category: 'network',
    icon: Send,
    route: '/tools/http',
    keywords: ['http', 'request', 'api', 'postman', 'get', 'post', 'rest', '请求', '接口测试'],
  },
  {
    id: 'websocket',
    name: 'WS 调试',
    description: '连接 WebSocket 服务,双向收发消息,实时查看消息流。',
    category: 'network',
    icon: Webhook,
    route: '/tools/websocket',
    keywords: ['websocket', 'ws', 'socket', 'wss', '实时', '推送', '调试', 'echo'],
  },
  {
    id: 'httpstatus',
    name: '状态码表',
    description: '全量 HTTP 状态码、含义与使用场景,可搜索过滤。',
    category: 'network',
    icon: Globe,
    route: '/tools/httpstatus',
    keywords: ['http', 'status', 'code', '状态码', '响应码', 'rest'],
  },
  {
    id: 'ipcalc',
    name: 'IP 子网',
    description: 'IPv4 子网划分:CIDR、掩码、网段范围、可用主机数一键计算。',
    category: 'network',
    icon: Network,
    route: '/tools/ipcalc',
    keywords: ['ip', 'subnet', 'cidr', 'mask', '网络', '子网', '掩码', 'ipv4'],
  },
  // ---- 开发配置 ----
  {
    id: 'cli',
    name: '配置管理',
    description: '管理 Claude Code / Codex CLI 供应商配置,一键切换、编辑、查看 live 配置。',
    category: 'devops',
    icon: Terminal,
    route: '/tools/cli',
    keywords: ['claude', 'codex', 'cli', 'provider', '配置', '切换', 'config'],
  },
  {
    id: 'proxy',
    name: '反代统计',
    description: '启动本地代理接管 CLI,实时查看请求数据、Token 用量与费用日志。',
    category: 'devops',
    icon: Network,
    route: '/tools/proxy',
    keywords: [
      'proxy',
      '反代',
      '代理',
      'takeover',
      '接管',
      '上游',
      'usage',
      'stats',
      '统计',
      '日志',
      'token',
      '费用',
    ],
  },
];
