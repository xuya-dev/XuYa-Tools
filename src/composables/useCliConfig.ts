import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// ==================== 类型定义 ====================
// 与 Rust 端 cli_config::types 对齐

export type AppType = 'claude' | 'codex';
export type ProviderScope = 'claude' | 'codex' | 'both';
export type ProviderKind = 'official' | 'relay' | 'custom';
export type ProviderCategory = 'official' | 'cn_official' | 'cloud_provider' | 'aggregator' | 'third_party' | 'custom';
export type AuthField = 'ANTHROPIC_AUTH_TOKEN' | 'ANTHROPIC_API_KEY';
export type ApiFormat = 'anthropic' | 'openai_chat' | 'openai_responses' | 'gemini_native';

export interface CliProvider {
    id: string;
    name: string;
    scope: ProviderScope;
    kind: ProviderKind;
    category: ProviderCategory;
    base_url: string;
    api_key: string;
    model: string;
    model_sonnet: string;
    model_haiku: string;
    model_opus: string;
    sonnet_name: string;
    opus_name: string;
    haiku_name: string;
    sonnet_1m: boolean;
    opus_1m: boolean;
    haiku_1m: boolean;
    note: string;
    website_url: string;
    auth_field: AuthField;
    api_format: ApiFormat;
    custom_user_agent: string;
    models_url: string;
    preset_id: string;
    icon: string;
    icon_color: string;
    codex_auth_json: string;
    codex_config_toml: string;
    claude_settings_json: string;
    updated_at: number;
}

export interface CliLiveStatus {
    installed: boolean;
    config_path: string;
    base_url: string;
    model: string;
    model_sonnet: string;
    model_opus: string;
    model_haiku: string;
    taken_over: boolean;
    matched_provider_id: string | null;
    matched_provider_name: string | null;
}

export interface CliStatus {
    claude: CliLiveStatus;
    codex: CliLiveStatus;
}

export interface SwitchResult {
    success: boolean;
    message: string;
    backup_path: string;
}

/** 从供应商 /v1/models 拉取的模型条目 */
export interface FetchedModel {
    id: string;
    ownedBy: string | null;
}

export interface CliWidgetSnapshot {
    claude_provider: string | null;
    claude_base_url: string;
    claude_installed: boolean;
    codex_provider: string | null;
    codex_base_url: string;
    codex_installed: boolean;
    proxy_running: boolean;
    today_requests: number;
    today_errors: number;
}

/** live 配置文件内容 (用于预览) */
export interface LiveConfigContent {
    path: string;
    content: string;
}

// ==================== 代理相关类型 ====================
export interface ProxyStatus {
    running: boolean;
    address: string;
    port: number;
    started_at: number;
    claude_provider_id: string | null;
    claude_provider_name: string | null;
    codex_provider_id: string | null;
    codex_provider_name: string | null;
    claude_taken_over: boolean;
    codex_taken_over: boolean;
}

export interface ProxyServerInfo {
    address: string;
    port: number;
    started_at: number;
}

export interface TakeoverResult {
    success: boolean;
    message: string;
}

// ==================== 请求统计类型 ====================
export interface UsageSummary {
    totalRequests: number;
    successCount: number;
    errorCount: number;
    successRate: number;
    avgLatencyMs: number;
    avgFirstTokenMs: number;
    streamingCount: number;
    totalInputTokens: number;
    totalOutputTokens: number;
    totalCacheReadTokens: number;
    totalCacheCreationTokens: number;
    totalCostUsd: number;
}

export interface LogFilters {
    appType?: string;
    providerId?: string;
    statusCode?: number;
}

export interface RequestLogDetail {
    id: number;
    requestId: string;
    appType: string;
    providerId: string | null;
    providerName: string | null;
    model: string | null;
    inputTokens: number;
    outputTokens: number;
    cacheReadTokens: number;
    cacheCreationTokens: number;
    totalCostUsd: number;
    latencyMs: number;
    firstTokenMs: number | null;
    statusCode: number;
    errorMessage: string | null;
    isStreaming: boolean;
    createdAt: number;
}

export interface DailyStat {
    date: string;
    timestamp: number;
    requestCount: number;
    successCount: number;
    errorCount: number;
    inputTokens: number;
    outputTokens: number;
    cacheReadTokens: number;
    cacheCreationTokens: number;
    costUsd: number;
    avgLatencyMs: number;
}

export type TimeRange = 'today' | '7d' | '30d';

// ==================== 余额查询类型 ====================
export interface BalanceItem {
    label: string;
    remaining: number | null;
    total: number | null;
    used: number | null;
    unit: string;
    isValid: boolean;
    invalidMessage: string | null;
    resetsAt: string | null;
}

export interface BalanceResult {
    success: boolean;
    items: BalanceItem[];
    error: string | null;
    isPlan: boolean;
}

export interface PaginatedLogs {
    data: RequestLogDetail[];
    total: number;
    page: number;
    pageSize: number;
}

// ==================== 单例状态 ====================
// provider 列表在多组件间共享，用模块级 ref 做轻量 store

const providers = ref<CliProvider[]>([]);
const status = ref<CliStatus | null>(null);
const loading = ref(false);

/** 拉取 provider 列表 */
async function refreshProviders() {
    try {
        providers.value = await invoke<CliProvider[]>('list_cli_providers');
    } catch (e) {
        console.error('拉取 provider 列表失败', e);
    }
}

/** 拉取当前 CLI live 状态 */
async function refreshStatus() {
    try {
        status.value = await invoke<CliStatus>('get_cli_status');
    } catch (e) {
        console.error('拉取 CLI 状态失败', e);
    }
}

/** 一次性刷新全部 */
async function refreshAll() {
    loading.value = true;
    await Promise.all([refreshProviders(), refreshStatus()]);
    loading.value = false;
}

/** 保存/更新 provider */
async function saveProvider(p: CliProvider): Promise<CliProvider> {
    const saved = await invoke<CliProvider>('save_cli_provider', { provider: p });
    await refreshProviders();
    return saved;
}

/** 删除 provider */
async function deleteProvider(id: string): Promise<boolean> {
    const ok = await invoke<boolean>('delete_cli_provider', { id });
    if (ok) await refreshProviders();
    return ok;
}

/** 切换某 app 到指定 provider */
async function switchProvider(appType: AppType, providerId: string): Promise<SwitchResult> {
    const result = await invoke<SwitchResult>('switch_cli_provider', { appType, providerId });
    // 正常模式: status 会变 (live config 被重写)
    // 接管模式: proxyStatus 会变 (代理上游变了, live config 没动)
    await Promise.all([refreshStatus(), refreshProxyStatus()]);
    return result;
}

/** 空壳 provider 模板 */
async function newProviderTemplate(): Promise<CliProvider> {
    return await invoke<CliProvider>('new_cli_provider_template');
}

/** 从供应商端点获取可用模型列表 (OpenAI 兼容 GET /v1/models) */
async function fetchModels(
    baseUrl: string,
    apiKey: string,
    customUserAgent?: string,
    modelsUrl?: string,
): Promise<FetchedModel[]> {
    return await invoke<FetchedModel[]>('fetch_cli_models', {
        baseUrl,
        apiKey,
        customUserAgent: customUserAgent || null,
        modelsUrl: modelsUrl || null,
    });
}

/** 读取某 app 的 live 配置文件内容 (用于预览) */
async function getLiveConfig(appType: AppType): Promise<LiveConfigContent> {
    return await invoke<LiveConfigContent>('get_cli_live_config', { appType });
}

/** 在系统默认编辑器中打开配置文件 */
async function openConfigFile(appType: AppType): Promise<void> {
    await invoke('open_cli_config_file', { appType });
}

// ==================== 代理状态 ====================
const proxyStatus = ref<ProxyStatus | null>(null);
const proxyBusy = ref(false);

async function refreshProxyStatus() {
    try {
        proxyStatus.value = await invoke<ProxyStatus>('get_cli_proxy_status');
    } catch (e) {
        console.error('拉取代理状态失败', e);
    }
}

async function startProxy(): Promise<ProxyServerInfo> {
    proxyBusy.value = true;
    try {
        const info = await invoke<ProxyServerInfo>('start_cli_proxy');
        await refreshProxyStatus();
        return info;
    } finally {
        proxyBusy.value = false;
    }
}

async function stopProxy(): Promise<void> {
    proxyBusy.value = true;
    try {
        await invoke('stop_cli_proxy');
        await refreshProxyStatus();
    } finally {
        proxyBusy.value = false;
    }
}

async function setTakeover(appType: AppType, enabled: boolean): Promise<TakeoverResult> {
    const result = await invoke<TakeoverResult>('set_cli_takeover', { appType, enabled });
    await refreshProxyStatus();
    return result;
}

// ==================== 请求统计状态 ====================
const usageSummary = ref<UsageSummary | null>(null);
const dailyStats = ref<DailyStat[]>([]);
const requestLogs = ref<RequestLogDetail[]>([]);
const logsTotal = ref(0);
const logsPage = ref(1);
const logsPageSize = ref(20);
const statsTimeRange = ref<TimeRange>('today');

function rangeToTimestamps(range: TimeRange): { start: number | null; end: number | null } {
    const now = Math.floor(Date.now() / 1000);
    const daySec = 86400;
    const todayStart = now - (now % daySec);
    switch (range) {
        case 'today': return { start: todayStart, end: null };
        case '7d': return { start: todayStart - 6 * daySec, end: null };
        case '30d': return { start: todayStart - 29 * daySec, end: null };
    }
}

async function refreshUsageSummary() {
    try {
        const { start, end } = rangeToTimestamps(statsTimeRange.value);
        usageSummary.value = await invoke<UsageSummary>('get_usage_summary', {
            startDate: start,
            endDate: end,
        });
    } catch (e) {
        console.error('拉取统计失败', e);
    }
}

async function refreshDailyStats() {
    try {
        const { start, end } = rangeToTimestamps(statsTimeRange.value);
        dailyStats.value = await invoke<DailyStat[]>('get_daily_stats', {
            startDate: start,
            endDate: end,
        });
    } catch (e) {
        console.error('拉取每日统计失败', e);
    }
}

async function setStatsTimeRange(range: TimeRange) {
    statsTimeRange.value = range;
    await Promise.all([refreshUsageSummary(), refreshDailyStats()]);
}

async function refreshRequestLogs(page = logsPage.value) {
    try {
        const res = await invoke<PaginatedLogs>('get_request_logs', {
            filters: { appType: null, providerId: null, statusCode: null },
            page,
            pageSize: logsPageSize.value,
        });
        requestLogs.value = res.data;
        logsTotal.value = res.total;
        logsPage.value = res.page;
    } catch (e) {
        console.error('拉取日志失败', e);
    }
}

async function clearRequestLogs(): Promise<void> {
    await invoke('clear_request_logs');
    await Promise.all([refreshUsageSummary(), refreshRequestLogs(1)]);
}

// ==================== 余额查询 ====================
const balanceCache = ref<Record<string, { result: BalanceResult; ts: number }>>({});

async function fetchBalance(
    p: CliProvider,
): Promise<BalanceResult> {
    if (!p.base_url || !p.api_key) {
        return { success: false, items: [], error: '缺少 base_url 或 API Key', isPlan: false };
    }
    try {
        const result = await invoke<BalanceResult>('fetch_balance', {
            baseUrl: p.base_url,
            apiKey: p.api_key,
        });
        balanceCache.value[p.id] = { result, ts: Date.now() };
        return result;
    } catch (e) {
        const result: BalanceResult = {
            success: false,
            items: [],
            error: String(e),
            isPlan: false,
        };
        balanceCache.value[p.id] = { result, ts: Date.now() };
        return result;
    }
}

export function useCliConfig() {
    return {
        providers,
        status,
        loading,
        refreshProviders,
        refreshStatus,
        refreshAll,
        saveProvider,
        deleteProvider,
        switchProvider,
        newProviderTemplate,
        fetchModels,
        getLiveConfig,
        openConfigFile,
        // 代理相关
        proxyStatus,
        proxyBusy,
        refreshProxyStatus,
        startProxy,
        stopProxy,
        setTakeover,
        // 统计相关
        usageSummary,
        dailyStats,
        statsTimeRange,
        requestLogs,
        logsTotal,
        logsPage,
        logsPageSize,
        refreshUsageSummary,
        refreshDailyStats,
        setStatsTimeRange,
        refreshRequestLogs,
        clearRequestLogs,
        // 余额查询
        balanceCache,
        fetchBalance,
    };
}
