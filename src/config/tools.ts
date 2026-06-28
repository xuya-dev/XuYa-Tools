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
} from '@lucide/vue';

export type ToolCategory = 'converter' | 'encoder' | 'crypto' | 'datetime' | 'devops';

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
  encoder: '编码解码',
  crypto: '加密 / 生成',
  datetime: '日期时间',
  devops: '开发配置',
};

export const tools: ToolMeta[] = [
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
    id: 'encoding',
    name: '编码转换',
    description: 'Base64 / URL / HTML 实体 / Hex / Binary 双向转换。',
    category: 'encoder',
    icon: Binary,
    route: '/tools/encoding',
    keywords: ['base64', 'url', 'html', 'hex', 'binary', '编码', '解码', '转义'],
  },
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
    id: 'timestamp',
    name: '时间戳转换',
    description: 'Unix 时间戳与可读时间互转,支持秒 / 毫秒,实时当前时间。',
    category: 'datetime',
    icon: Clock,
    route: '/tools/timestamp',
    keywords: ['timestamp', 'unix', 'epoch', '时间戳', '日期', 'date'],
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
  {
    id: 'cli',
    name: 'CLI 配置 / 反代',
    description: '管理 Claude Code / Codex CLI 供应商配置,本地反代与请求统计。',
    category: 'devops',
    icon: Terminal,
    route: '/tools/cli',
    keywords: ['claude', 'codex', 'cli', 'proxy', '反代', '代理', '统计', '配置', 'provider'],
  },
];
