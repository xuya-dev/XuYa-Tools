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
  Regex,
  Palette,
  FileText,
  QrCode,
  GitCompare,
  ScanText,
  ListTree,
  Globe,
  Network,
  BarChart3,
} from '@lucide/vue';

export type ToolCategory =
  | 'converter'
  | 'encoder'
  | 'crypto'
  | 'datetime'
  | 'devops'
  | 'text'
  | 'network';

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
  encoder: '编码 / 生成',
  crypto: '加密 / 解析',
  datetime: '日期时间',
  devops: '开发配置',
  text: '文本处理',
  network: '网络 / 参考',
};

export const tools: ToolMeta[] = [
  // ---- 格式转换 ----
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
    name: '正则表达式测试',
    description: '实时匹配高亮、捕获组提取、标志位切换、常用速查。',
    category: 'converter',
    icon: Regex,
    route: '/tools/regex',
    keywords: ['regex', 'regexp', 'regular expression', '正则', '匹配', 'pattern'],
  },
  {
    id: 'diff',
    name: '文本差异对比',
    description: '双栏输入,行级 diff 高亮,统计增删改。',
    category: 'converter',
    icon: GitCompare,
    route: '/tools/diff',
    keywords: ['diff', 'compare', '差异', '对比', '比较'],
  },
  {
    id: 'markdown',
    name: 'Markdown 预览',
    description: '实时分栏:左侧编辑,右侧预览,支持代码块与表格。',
    category: 'converter',
    icon: FileText,
    route: '/tools/markdown',
    keywords: ['markdown', 'md', 'preview', '预览', '文档', '渲染'],
  },
  // ---- 编码 / 生成 ----
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
    name: '二维码生成',
    description: '文本 / URL 转 QR 码,可调尺寸与纠错级别,下载 PNG。',
    category: 'encoder',
    icon: QrCode,
    route: '/tools/qrcode',
    keywords: ['qrcode', 'qr', '二维码', '条码', 'barcode'],
  },
  {
    id: 'lorem',
    name: '占位文本生成',
    description: '生成 Lorem Ipsum / 中文占位文本,段落 / 句子 / 单词。',
    category: 'encoder',
    icon: ScanText,
    route: '/tools/lorem',
    keywords: ['lorem', 'ipsum', 'placeholder', '占位', '假文', '填充文本'],
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
    id: 'hashgen',
    name: 'UUID / 密码 / 哈希',
    description: '生成 UUID、随机密码,计算 MD5 / SHA-1 / SHA-256 等哈希。',
    category: 'crypto',
    icon: Fingerprint,
    route: '/tools/hashgen',
    keywords: ['uuid', 'password', 'md5', 'sha', 'hash', '密码', '哈希', '随机'],
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
    name: 'Cron 表达式解析',
    description: '解析 Cron 五段表达式,人类可读描述,未来执行时间预览。',
    category: 'datetime',
    icon: ListTree,
    route: '/tools/cron',
    keywords: ['cron', 'crontab', 'schedule', '定时', '计划任务', '表达式'],
  },
  // ---- 网络参考 ----
  {
    id: 'httpstatus',
    name: 'HTTP 状态码速查',
    description: '全量 HTTP 状态码、含义与使用场景,可搜索过滤。',
    category: 'network',
    icon: Globe,
    route: '/tools/httpstatus',
    keywords: ['http', 'status', 'code', '状态码', '响应码', 'rest'],
  },
  // ---- 开发配置 ----
  {
    id: 'cli',
    name: 'CLI 配置管理',
    description: '管理 Claude Code / Codex CLI 供应商配置,一键切换、编辑、查看 live 配置。',
    category: 'devops',
    icon: Terminal,
    route: '/tools/cli',
    keywords: ['claude', 'codex', 'cli', 'provider', '配置', '切换', 'config'],
  },
  {
    id: 'proxy',
    name: '本地反代',
    description: '启动本地代理,接管 Claude Code / Codex CLI,将请求导向配置的上游供应商。',
    category: 'devops',
    icon: Network,
    route: '/tools/proxy',
    keywords: ['proxy', '反代', '代理', 'takeover', '接管', '上游', 'upstream'],
  },
  {
    id: 'usage',
    name: '请求统计',
    description: '查看反代记录的请求数据:成功率、延迟、Token 用量、费用与明细日志。',
    category: 'devops',
    icon: BarChart3,
    route: '/tools/usage',
    keywords: ['usage', 'stats', '统计', '日志', 'log', 'token', '费用', '延迟'],
  },
];
