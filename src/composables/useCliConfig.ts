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
    active_provider_id: string | null;
    active_provider_name: string | null;
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
    totalInputTokens: number;
    totalOutputTokens: number;
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
    totalCostUsd: number;
    latencyMs: number;
    firstTokenMs: number | null;
    statusCode: number;
    errorMessage: string | null;
    isStreaming: boolean;
    createdAt: number;
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
    await refreshStatus();
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
const requestLogs = ref<RequestLogDetail[]>([]);
const logsTotal = ref(0);
const logsPage = ref(1);
const logsPageSize = ref(20);

async function refreshUsageSummary() {
    try {
        usageSummary.value = await invoke<UsageSummary>('get_usage_summary', {
            startDate: null,
            endDate: null,
        });
    } catch (e) {
        console.error('拉取统计失败', e);
    }
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
        requestLogs,
        logsTotal,
        logsPage,
        logsPageSize,
        refreshUsageSummary,
        refreshRequestLogs,
        clearRequestLogs,
    };
}
