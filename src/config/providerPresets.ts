/**
 * Provider 预设数据 (对齐 cc-switch 的预设体系)
 *
 * 用户添加配置时从预设列表点选, 自动填充 base_url/category 等字段,
 * 只需手动填 API Key 即可。这是 cc-switch 表单体验的核心。
 */

export type PresetCategory =
    | 'official'
    | 'cn_official'
    | 'cloud_provider'
    | 'aggregator'
    | 'third_party'
    | 'custom';

export type PresetScope = 'claude' | 'codex' | 'both';
export type PresetAuthField = 'ANTHROPIC_AUTH_TOKEN' | 'ANTHROPIC_API_KEY';
export type PresetApiFormat = 'anthropic' | 'openai_chat' | 'openai_responses' | 'gemini_native';

export interface ProviderPreset {
    /** 预设唯一 id */
    id: string;
    /** 显示名称 */
    name: string;
    /** 适用范围 */
    scope: PresetScope;
    /** 分类 */
    category: PresetCategory;
    /** 上游 base URL (官方为空) */
    baseUrl: string;
    /** 官网链接 */
    websiteUrl: string;
    /** 获取 API Key 的链接 (可选) */
    apiKeyUrl?: string;
    /** Claude 认证字段 (scope 含 claude 时) */
    authField?: PresetAuthField;
    /** API 格式 */
    apiFormat?: PresetApiFormat;
    /** 默认模型 (可选) */
    model?: string;
    /** 图标标识 (用于前端图标渲染) */
    icon?: string;
    /** 是否官方 */
    isOfficial?: boolean;
    /** 模型列表端点覆写 (可选, 解决部分供应商 /models 不在默认路径) */
    modelsUrl?: string;
}

// ==================== 分类标签 ====================
export const CATEGORY_LABELS: Record<PresetCategory, string> = {
    official: '官方',
    cn_official: '国内官方',
    cloud_provider: '云厂商',
    aggregator: '聚合中转',
    third_party: '第三方',
    custom: '自定义',
};

// ==================== 品牌色 / 图标首字母映射 ====================
// 用于 provider 卡片左侧的彩色圆标。对齐 cc-switch 的 icon + iconColor 体系。
export const BRAND_COLORS: Record<string, string> = {
    claude: '#d97757',
    openai: '#10a37f',
    kimi: '#1e1e1e',
    deepseek: '#4d6bfe',
    glm: '#3859ff',
    bailian: '#7b3df5',
    volcano: '#ff5c00',
    minimax: '#ff3d00',
    stepfun: '#0066ff',
    longcat: '#ffc300',
    custom: '#6b7280',
};

/** 获取品牌色, 未知时回退灰色 */
export function brandColor(icon?: string): string {
    if (icon) {
        const c = BRAND_COLORS[icon];
        if (c) return c;
    }
    return BRAND_COLORS.custom ?? '#6b7280';
}

/** 取名称首字作为图标文字 (在 SVG 图标缺失时作为回退) */
export function iconText(name: string): string {
    const trimmed = name.trim();
    if (!trimmed) return '?';
    return trimmed.charAt(0).toUpperCase();
}

// ==================== LobeHub 图标库映射 ====================
// 通过 npm 安装 @lobehub/icons-static-svg, 显式 import 需要的 SVG。
// LobeHub 的 SVG 均为 fill="currentColor" 单色图标, 用 ?raw 取原始标记,
// 再通过 v-html 内联渲染 —— 这样 currentColor 可继承容器 CSS 的 color,
// 在品牌色圆底上显示为白色图标。完全离线, 不依赖任何 CDN。
// 详见 https://www.npmjs.com/package/@lobehub/icons-static-svg

// 静态 import 需要的图标原始 SVG 标记 (Vite ?raw: 返回文件内容字符串)
import claudeIconRaw from '@lobehub/icons-static-svg/icons/claude.svg?raw';
import openaiIconRaw from '@lobehub/icons-static-svg/icons/openai.svg?raw';
import kimiIconRaw from '@lobehub/icons-static-svg/icons/kimi.svg?raw';
import deepseekIconRaw from '@lobehub/icons-static-svg/icons/deepseek.svg?raw';
import zhipuIconRaw from '@lobehub/icons-static-svg/icons/zhipu.svg?raw';
import bailianIconRaw from '@lobehub/icons-static-svg/icons/bailian.svg?raw';
import doubaoIconRaw from '@lobehub/icons-static-svg/icons/doubao.svg?raw';
import minimaxIconRaw from '@lobehub/icons-static-svg/icons/minimax.svg?raw';
import stepfunIconRaw from '@lobehub/icons-static-svg/icons/stepfun.svg?raw';
import longcatIconRaw from '@lobehub/icons-static-svg/icons/longcat.svg?raw';

/** 项目内 icon slug -> 本地 SVG 标记 */
const LOBE_ICONS_RAW: Record<string, string> = {
    claude: claudeIconRaw,
    openai: openaiIconRaw,
    kimi: kimiIconRaw,
    deepseek: deepseekIconRaw,
    // glm 在 LobeHub 中没有同名 slug, 对应品牌是「智谱」
    glm: zhipuIconRaw,
    bailian: bailianIconRaw,
    // Volcengine 的模型是「豆包」, LobeHub 中是 doubao
    volcano: doubaoIconRaw,
    minimax: minimaxIconRaw,
    stepfun: stepfunIconRaw,
    longcat: longcatIconRaw,
};

/**
 * 取 LobeHub 图标的内联 SVG 标记 (用于 v-html 渲染)。
 * 图标为 currentColor 单色, 颜色由容器 CSS color 决定。
 * 返回 null 表示无对应图标 (调用方应回退到文字图标)。
 */
export function iconSvg(icon?: string): string | null {
    if (!icon) return null;
    return LOBE_ICONS_RAW[icon] ?? null;
}

/** 是否存在可用的 SVG 图标 */
export function hasIconImage(icon?: string): boolean {
    return iconSvg(icon) !== null;
}

// ==================== Claude Code 预设 ====================
// 仅保留「官方」与「国内官方」，其余一律移除内置。自定义入口保留。
export const CLAUDE_PRESETS: ProviderPreset[] = [
    {
        id: 'claude-official',
        name: 'Claude 官方',
        scope: 'claude',
        category: 'official',
        baseUrl: '',
        websiteUrl: 'https://claude.ai',
        authField: 'ANTHROPIC_AUTH_TOKEN',
        apiFormat: 'anthropic',
        isOfficial: true,
        icon: 'claude',
    },
    {
        id: 'claude-kimi',
        name: 'Kimi (Moonshot)',
        scope: 'claude',
        category: 'cn_official',
        baseUrl: 'https://api.moonshot.cn/anthropic',
        websiteUrl: 'https://platform.moonshot.cn',
        apiKeyUrl: 'https://platform.moonshot.cn/console',
        authField: 'ANTHROPIC_AUTH_TOKEN',
        apiFormat: 'anthropic',
        icon: 'kimi',
    },
    {
        id: 'claude-kimi-coding',
        name: 'Kimi For Coding',
        scope: 'claude',
        category: 'cn_official',
        baseUrl: 'https://api.kimi.com/coding/',
        websiteUrl: 'https://platform.moonshot.cn',
        apiKeyUrl: 'https://platform.moonshot.cn/console',
        authField: 'ANTHROPIC_AUTH_TOKEN',
        apiFormat: 'anthropic',
        icon: 'kimi',
    },
    {
        id: 'claude-deepseek',
        name: 'DeepSeek',
        scope: 'claude',
        category: 'cn_official',
        baseUrl: 'https://api.deepseek.com/anthropic',
        websiteUrl: 'https://platform.deepseek.com',
        apiKeyUrl: 'https://platform.deepseek.com/api_keys',
        authField: 'ANTHROPIC_AUTH_TOKEN',
        apiFormat: 'anthropic',
        icon: 'deepseek',
    },
    {
        id: 'claude-zhipu',
        name: '智谱 GLM',
        scope: 'claude',
        category: 'cn_official',
        baseUrl: 'https://open.bigmodel.cn/api/anthropic',
        websiteUrl: 'https://open.bigmodel.cn',
        apiKeyUrl: 'https://open.bigmodel.cn/usercenter/apikeys',
        authField: 'ANTHROPIC_AUTH_TOKEN',
        apiFormat: 'anthropic',
        icon: 'glm',
    },
    {
        id: 'claude-bailian',
        name: '阿里百炼',
        scope: 'claude',
        category: 'cn_official',
        baseUrl: 'https://dashscope.aliyuncs.com/apps/anthropic',
        websiteUrl: 'https://bailian.console.aliyun.com',
        apiKeyUrl: 'https://bailian.console.aliyun.com/?apiKey=1',
        authField: 'ANTHROPIC_AUTH_TOKEN',
        apiFormat: 'anthropic',
        icon: 'bailian',
    },
    {
        id: 'claude-volcano',
        name: '火山 Agentplan',
        scope: 'claude',
        category: 'cn_official',
        baseUrl: 'https://ark.cn-beijing.volces.com/api/coding',
        websiteUrl: 'https://www.volcengine.com',
        apiKeyUrl: 'https://console.volcengine.com/ark/region:ark+cn-beijing/apiKey',
        authField: 'ANTHROPIC_AUTH_TOKEN',
        apiFormat: 'anthropic',
        icon: 'volcano',
    },
    {
        id: 'claude-minimax',
        name: 'MiniMax',
        scope: 'claude',
        category: 'cn_official',
        baseUrl: 'https://api.minimaxi.com/anthropic',
        websiteUrl: 'https://platform.minimaxi.com',
        apiKeyUrl: 'https://platform.minimaxi.com/user-center/basic-information/interface-key',
        authField: 'ANTHROPIC_AUTH_TOKEN',
        apiFormat: 'anthropic',
        icon: 'minimax',
    },
    {
        id: 'claude-stepfun',
        name: '阶跃星辰',
        scope: 'claude',
        category: 'cn_official',
        baseUrl: 'https://api.stepfun.com/step_plan',
        websiteUrl: 'https://platform.stepfun.com',
        apiKeyUrl: 'https://platform.stepfun.com/interface-key',
        authField: 'ANTHROPIC_AUTH_TOKEN',
        apiFormat: 'anthropic',
        icon: 'stepfun',
    },
    {
        id: 'claude-longcat',
        name: '美团 Longcat',
        scope: 'claude',
        category: 'cn_official',
        baseUrl: 'https://api.longcat.chat/anthropic',
        websiteUrl: 'https://platform.longcat.chat',
        apiKeyUrl: 'https://platform.longcat.chat/operation/apikey',
        authField: 'ANTHROPIC_AUTH_TOKEN',
        apiFormat: 'anthropic',
        icon: 'longcat',
    },
    {
        id: 'claude-custom',
        name: '自定义',
        scope: 'claude',
        category: 'custom',
        baseUrl: '',
        websiteUrl: '',
        authField: 'ANTHROPIC_AUTH_TOKEN',
        apiFormat: 'anthropic',
        icon: 'custom',
    },
];

// ==================== Codex CLI 预设 ====================
// 仅保留「官方」与「国内官方」，其余一律移除内置。自定义入口保留。
export const CODEX_PRESETS: ProviderPreset[] = [
    {
        id: 'codex-openai-official',
        name: 'OpenAI 官方',
        scope: 'codex',
        category: 'official',
        baseUrl: '',
        websiteUrl: 'https://platform.openai.com',
        apiFormat: 'openai_responses',
        isOfficial: true,
        icon: 'openai',
    },
    {
        id: 'codex-kimi',
        name: 'Kimi (Moonshot)',
        scope: 'codex',
        category: 'cn_official',
        baseUrl: 'https://api.moonshot.cn/v1',
        websiteUrl: 'https://platform.moonshot.cn',
        apiKeyUrl: 'https://platform.moonshot.cn/console',
        apiFormat: 'openai_chat',
        icon: 'kimi',
    },
    {
        id: 'codex-kimi-coding',
        name: 'Kimi For Coding',
        scope: 'codex',
        category: 'cn_official',
        baseUrl: 'https://api.kimi.com/coding/v1',
        websiteUrl: 'https://platform.moonshot.cn',
        apiKeyUrl: 'https://platform.moonshot.cn/console',
        apiFormat: 'openai_chat',
        icon: 'kimi',
    },
    {
        id: 'codex-deepseek',
        name: 'DeepSeek',
        scope: 'codex',
        category: 'cn_official',
        baseUrl: 'https://api.deepseek.com',
        websiteUrl: 'https://platform.deepseek.com',
        apiKeyUrl: 'https://platform.deepseek.com/api_keys',
        apiFormat: 'openai_chat',
        icon: 'deepseek',
    },
    {
        id: 'codex-zhipu',
        name: '智谱 GLM',
        scope: 'codex',
        category: 'cn_official',
        baseUrl: 'https://open.bigmodel.cn/api/coding/paas/v4',
        websiteUrl: 'https://open.bigmodel.cn',
        apiKeyUrl: 'https://open.bigmodel.cn/usercenter/apikeys',
        apiFormat: 'openai_chat',
        icon: 'glm',
    },
    {
        id: 'codex-bailian',
        name: '阿里百炼',
        scope: 'codex',
        category: 'cn_official',
        baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
        websiteUrl: 'https://bailian.console.aliyun.com',
        apiKeyUrl: 'https://bailian.console.aliyun.com/?apiKey=1',
        apiFormat: 'openai_responses',
        icon: 'bailian',
    },
    {
        id: 'codex-volcano',
        name: '火山 Agentplan',
        scope: 'codex',
        category: 'cn_official',
        baseUrl: 'https://ark.cn-beijing.volces.com/api/coding/v3',
        websiteUrl: 'https://www.volcengine.com',
        apiKeyUrl: 'https://console.volcengine.com/ark/region:ark+cn-beijing/apiKey',
        apiFormat: 'openai_chat',
        icon: 'volcano',
    },
    {
        id: 'codex-minimax',
        name: 'MiniMax',
        scope: 'codex',
        category: 'cn_official',
        baseUrl: 'https://api.minimaxi.com/v1',
        websiteUrl: 'https://platform.minimaxi.com',
        apiKeyUrl: 'https://platform.minimaxi.com/user-center/basic-information/interface-key',
        apiFormat: 'openai_responses',
        icon: 'minimax',
    },
    {
        id: 'codex-stepfun',
        name: '阶跃星辰',
        scope: 'codex',
        category: 'cn_official',
        baseUrl: 'https://api.stepfun.com/step_plan/v1',
        websiteUrl: 'https://platform.stepfun.com',
        apiKeyUrl: 'https://platform.stepfun.com/interface-key',
        apiFormat: 'openai_chat',
        icon: 'stepfun',
    },
    {
        id: 'codex-longcat',
        name: '美团 Longcat',
        scope: 'codex',
        category: 'cn_official',
        baseUrl: 'https://api.longcat.chat/openai/v1',
        websiteUrl: 'https://platform.longcat.chat',
        apiKeyUrl: 'https://platform.longcat.chat/operation/apikey',
        apiFormat: 'openai_responses',
        icon: 'longcat',
    },
    {
        id: 'codex-custom',
        name: '自定义',
        scope: 'codex',
        category: 'custom',
        baseUrl: '',
        websiteUrl: '',
        apiFormat: 'openai_responses',
        icon: 'custom',
    },
];

/** 按 scope 获取预设列表 */
export function getPresetsByScope(scope: PresetScope): ProviderPreset[] {
    if (scope === 'both') return [...CLAUDE_PRESETS, ...CODEX_PRESETS];
    if (scope === 'claude') return CLAUDE_PRESETS;
    return CODEX_PRESETS;
}

/** 按 id 查找预设 */
export function findPreset(id: string): ProviderPreset | undefined {
    return [...CLAUDE_PRESETS, ...CODEX_PRESETS].find((p) => p.id === id);
}
