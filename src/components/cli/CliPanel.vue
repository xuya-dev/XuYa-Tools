<template>
    <div class="cli-panel">
        <!-- 当前 live 状态卡 -->
        <div class="cli-status-grid">
            <div class="cli-status-card" :class="{ active: status?.claude.matched_provider_id }">
                <div class="cli-status-head">
                    <div class="cli-app-name-wrap">
                        <span v-if="iconSvg('claude')" class="cli-app-icon" v-html="iconSvg('claude')!"></span>
                        <span class="cli-app-name">Claude Code</span>
                    </div>
                    <span class="cli-badge" :class="claudeBadgeClass">{{ claudeBadgeText }}</span>
                </div>
                <div class="cli-status-row">
                    <span class="cli-status-label">当前 Provider</span>
                    <span class="cli-status-val">{{ status?.claude.matched_provider_name || '未匹配' }}</span>
                </div>
                <div class="cli-status-row">
                    <span class="cli-status-label">Base URL</span>
                    <span class="cli-status-val mono">{{ status?.claude.base_url || '官方默认' }}</span>
                </div>
                <div v-if="status?.claude.model_sonnet || status?.claude.model_opus || status?.claude.model_haiku || status?.claude.model" class="cli-status-row">
                    <span class="cli-status-label">当前模型</span>
                    <span class="cli-status-val model-pills">
                        <span v-if="status?.claude.model_sonnet" class="live-model-pill">
                            Sonnet {{ strip1M(status.claude.model_sonnet) }}<span v-if="has1M(status.claude.model_sonnet)" class="onem-badge">1M</span>
                        </span>
                        <span v-if="status?.claude.model_opus" class="live-model-pill">
                            Opus {{ strip1M(status.claude.model_opus) }}<span v-if="has1M(status.claude.model_opus)" class="onem-badge">1M</span>
                        </span>
                        <span v-if="status?.claude.model_haiku" class="live-model-pill">
                            Haiku {{ strip1M(status.claude.model_haiku) }}
                        </span>
                        <span v-if="!status?.claude.model_sonnet && !status?.claude.model_opus && !status?.claude.model_haiku && status?.claude.model" class="live-model-pill">
                            {{ strip1M(status.claude.model) }}<span v-if="has1M(status.claude.model)" class="onem-badge">1M</span>
                        </span>
                    </span>
                </div>
                <div class="cli-status-row">
                    <span class="cli-status-label">配置路径</span>
                    <span class="cli-status-val mono small">{{ status?.claude.config_path || '未检测到' }}</span>
                </div>
            </div>

            <div class="cli-status-card" :class="{ active: status?.codex.matched_provider_id }">
                <div class="cli-status-head">
                    <div class="cli-app-name-wrap">
                        <span v-if="iconSvg('openai')" class="cli-app-icon" v-html="iconSvg('openai')!"></span>
                        <span class="cli-app-name">Codex CLI</span>
                    </div>
                    <span class="cli-badge" :class="codexBadgeClass">{{ codexBadgeText }}</span>
                </div>
                <div class="cli-status-row">
                    <span class="cli-status-label">当前 Provider</span>
                    <span class="cli-status-val">{{ status?.codex.matched_provider_name || '未匹配' }}</span>
                </div>
                <div class="cli-status-row">
                    <span class="cli-status-label">Base URL</span>
                    <span class="cli-status-val mono">{{ status?.codex.base_url || '官方默认' }}</span>
                </div>
                <div class="cli-status-row">
                    <span class="cli-status-label">配置路径</span>
                    <span class="cli-status-val mono small">{{ status?.codex.config_path || '未检测到' }}</span>
                </div>
            </div>
        </div>

        <!-- 配置文件预览 -->
        <div class="config-viewer">
            <div class="config-viewer-head">
                <div class="config-viewer-tabs">
                    <button
                        class="config-tab"
                        :class="{ active: configViewer.app === 'claude' }"
                        @click="loadConfigViewer('claude')"
                    >Claude 配置</button>
                    <button
                        class="config-tab"
                        :class="{ active: configViewer.app === 'codex' }"
                        @click="loadConfigViewer('codex')"
                    >Codex 配置</button>
                </div>
                <div v-if="configViewer.content" class="config-viewer-actions">
                    <button class="cli-mini-btn ghost" :disabled="configViewer.loading" @click="loadConfigViewer(configViewer.app, true)">
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/></svg>
                        刷新
                    </button>
                    <button class="cli-mini-btn ghost" @click="onOpenConfig(configViewer.app)">
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
                        在编辑器中打开
                    </button>
                </div>
            </div>
            <div v-if="configViewer.open" class="config-viewer-body">
                <div v-if="configViewer.path" class="config-viewer-path mono">{{ configViewer.path }}</div>
                <pre class="config-viewer-code"><code v-if="configViewer.loading">加载中…</code><code v-else>{{ configViewer.content || '（无内容 / 文件不存在）' }}</code></pre>
            </div>
        </div>

        <!-- provider 列表 (卡片式) -->
        <div class="cli-providers">
            <div class="cli-providers-head">
                <span>已保存的 Provider</span>
                <button class="cli-add-btn" @click="openAdd">+ 新增</button>
            </div>

            <div v-if="providers.length === 0" class="cli-empty">
                暂无保存的 Provider，点击「新增」创建第一个
            </div>

            <div v-else class="provider-card-grid">
                <div v-for="p in providers" :key="p.id" class="provider-card" :class="{ active: isActiveProvider(p) }">
                    <span v-if="isActiveProvider(p)" class="provider-current-badge">当前</span>
                    <div class="provider-card-head">
                        <div class="provider-icon" :style="{ color: p.icon_color || brandColor(p.icon) }">
                            <span v-if="hasIconImage(p.icon)" class="provider-icon-svg" v-html="iconSvg(p.icon)!"></span>
                            <span v-else>{{ iconText(p.name) }}</span>
                        </div>
                        <div class="provider-card-info">
                            <span class="provider-card-name">{{ p.name }}</span>
                            <span class="provider-card-cat" :class="p.category">{{ CATEGORY_LABELS[p.category] }} · {{ scopeLabel(p.scope) }}</span>
                        </div>
                    </div>
                    <div v-if="p.base_url" class="provider-card-url mono">{{ p.base_url }}</div>
                    <div v-else class="provider-card-url">官方登录</div>
                    <!-- 模型摘要 -->
                    <div v-if="modelSummary(p).length" class="provider-models">
                        <span v-for="(m, i) in modelSummary(p)" :key="i" class="model-tag" :class="m.role.toLowerCase()">
                            {{ m.role }}: {{ m.model }}<span v-if="m.oneM" class="model-1m-mark"> ·1M</span>
                        </span>
                    </div>
                    <div v-else class="provider-models">
                        <span class="model-tag default">默认模型</span>
                    </div>
                    <div class="provider-card-actions">
                        <button
                            v-if="appliesClaude(p)"
                            class="cli-mini-btn"
                            :class="{ current: status?.claude.matched_provider_id === p.id }"
                            @click="onSwitch('claude', p.id)"
                        >切换到 Claude</button>
                        <button
                            v-if="appliesCodex(p)"
                            class="cli-mini-btn"
                            :class="{ current: status?.codex.matched_provider_id === p.id }"
                            @click="onSwitch('codex', p.id)"
                        >切换到 Codex</button>
                        <button
                            v-if="p.base_url"
                            class="cli-mini-btn ghost"
                            :disabled="!proxyStatus?.running"
                            @click="onSetTarget(p)"
                        >设为上游</button>
                        <button class="cli-mini-btn ghost" @click="onEdit(p)">编辑</button>
                        <button class="cli-mini-btn danger" @click="onDelete(p)">删除</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- 新增/编辑弹窗 (两步式: 预设首屏 -> 表单) -->
        <Transition name="fade">
            <div v-if="editor.visible" class="cli-modal-overlay" @click.self="closeEditor">
                <div class="cli-modal-card cli-modal-large">
                    <div class="cli-modal-header">
                        <h4 v-if="editor.isEdit">编辑 Provider</h4>
                        <h4 v-else-if="editor.step === 'preset'">添加新 Provider</h4>
                        <h4 v-else>配置 Provider</h4>
                        <button v-if="!editor.isEdit && editor.step === 'form'" class="cli-back-btn" @click="editor.step = 'preset'">← 返回预设</button>
                    </div>


                    <!-- 步骤 1: 应用 Tab + 预设选择 -->
                    <div v-if="!editor.isEdit && editor.step === 'preset'" class="cli-modal-body">
                        <!-- 应用切换 Tab -->
                        <div class="app-tabs">
                            <button
                                class="app-tab"
                                :class="{ active: editor.appTab === 'claude' }"
                                @click="switchAppTab('claude')"
                            >
                                <span v-if="iconSvg('claude')" class="tab-icon" v-html="iconSvg('claude')!"></span>
                                Claude Code
                            </button>
                            <button
                                class="app-tab"
                                :class="{ active: editor.appTab === 'codex' }"
                                @click="switchAppTab('codex')"
                            >
                                <span v-if="iconSvg('openai')" class="tab-icon" v-html="iconSvg('openai')!"></span>
                                Codex CLI
                            </button>
                        </div>

                        <div class="preset-header">
                            <span class="preset-title">选择预设</span>
                            <input
                                v-model="presetSearch"
                                class="preset-search"
                                placeholder="搜索预设..."
                            />
                        </div>
                        <div class="preset-grid">
                            <div
                                v-for="preset in filteredPresets"
                                :key="preset.id"
                                class="preset-tile"
                                @click="selectPreset(preset)"
                            >
                                <div class="preset-tile-icon" :style="{ color: brandColor(preset.icon) }">
                                    <span v-if="hasIconImage(preset.icon)" class="provider-icon-svg" v-html="iconSvg(preset.icon)!"></span>
                                    <span v-else>{{ iconText(preset.name) }}</span>
                                </div>
                                <div class="preset-tile-text">
                                    <span class="preset-tile-name">{{ preset.name }}</span>
                                    <span class="preset-tile-cat" :class="preset.category">{{ CATEGORY_LABELS[preset.category] }}</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- 步骤 2: 表单 -->
                    <div v-else class="cli-modal-body">
                        <!-- 已选预设提示 -->
                        <div v-if="!editor.isEdit && editor.form.preset_id" class="selected-preset-bar">
                            <div class="provider-icon sm" :style="{ color: editor.form.icon_color || brandColor(editor.form.icon) }">
                                <span v-if="hasIconImage(editor.form.icon)" class="provider-icon-svg" v-html="iconSvg(editor.form.icon)!"></span>
                                <span v-else>{{ iconText(editor.form.name) }}</span>
                            </div>
                            <span>{{ selectedPreset?.name || editor.form.name }}</span>
                            <span class="selected-preset-cat" :class="editor.form.category">{{ CATEGORY_LABELS[editor.form.category] }}</span>
                        </div>

                        <!-- ===== 分区: 基础信息 ===== -->
                        <div class="form-section">
                            <div class="form-section-title">基础信息</div>
                            <div class="form-grid-2">
                                <label class="cli-field">
                                    <span>供应商名称 <span class="req">*</span></span>
                                    <input v-model="editor.form.name" placeholder="例如 我的 Claude 中转" />
                                </label>
                                <label class="cli-field">
                                    <span>备注</span>
                                    <input v-model="editor.form.note" placeholder="例如 公司专用账号" />
                                </label>
                            </div>

                            <label class="cli-field">
                                <span>官网链接</span>
                                <input v-model="editor.form.website_url" class="mono" placeholder="https://example.com（可选）" />
                            </label>

                            <!-- API Key (官方分类禁用) -->
                            <label class="cli-field">
                                <span>API Key</span>
                                <input
                                    v-model="editor.form.api_key"
                                    class="mono"
                                    type="password"
                                    :disabled="editor.form.category === 'official'"
                                    :placeholder="editor.form.category === 'official' ? '官方无需 API Key' : '输入 API Key，将自动填充到配置'"
                                />
                                <a v-if="selectedPreset?.apiKeyUrl" :href="selectedPreset.apiKeyUrl" target="_blank" class="get-key-link">获取 API Key ↗</a>
                            </label>

                            <!-- Base URL (官方/云厂商隐藏) -->
                            <label v-if="editor.form.category !== 'official' && editor.form.category !== 'cloud_provider'" class="cli-field">
                                <span>API 端点地址</span>
                                <input v-model="editor.form.base_url" class="mono" placeholder="https://your-api-endpoint.com" />
                                <span class="field-hint">💡 填写兼容对应协议的服务端点地址，不要以斜杠结尾</span>
                            </label>
                        </div>

                        <!-- ===== 分区: 模型配置 (仅 Claude) ===== -->
                        <div v-if="editor.appTab === 'claude'" class="form-section">
                            <div class="form-section-title">
                                <span>模型配置</span>
                                <button
                                    class="fetch-models-btn"
                                    :disabled="modelsLoading || !editor.form.base_url || editor.form.category === 'official'"
                                    @click="onFetchModels"
                                >
                                    <span v-if="modelsLoading" class="fetch-spinner"></span>
                                    {{ modelsLoading ? '获取中…' : '🔄 获取模型' }}
                                </button>
                            </div>
                            <!-- 模型候选 datalist (获取后填充, 手输也可联想) -->
                            <datalist :id="MODEL_DATALIST_ID">
                                <option v-for="m in fetchedModels" :key="m.id" :value="m.id">{{ m.ownedBy || '' }}</option>
                            </datalist>

                            <!-- 三角色模型行 (实际模型 + 显示名 + 1M, Sonnet/Opus 支持 1M, Haiku 不支持) -->
                            <div class="model-role-grid">
                                <div class="model-role-row">
                                    <span class="model-role-tag sonnet">Sonnet</span>
                                    <input
                                        v-model="editor.form.model_sonnet"
                                        class="mono model-input-main"
                                        list="xuya-fetched-models"
                                        placeholder="claude-sonnet-4-20250514"
                                    />
                                    <input
                                        v-model="editor.form.sonnet_name"
                                        class="model-input-name"
                                        placeholder="显示名"
                                        title="显示名称 (影响 Claude Code /model 菜单, 可选)"
                                    />
                                    <label class="model-1m-toggle" title="声明支持 1M 上下文 (写入模型名 [1M] 后缀, Claude Code 原生识别)">
                                        <input v-model="editor.form.sonnet_1m" type="checkbox" />
                                        <span class="model-1m-label">1M</span>
                                    </label>
                                </div>
                                <div class="model-role-row">
                                    <span class="model-role-tag opus">Opus</span>
                                    <input
                                        v-model="editor.form.model_opus"
                                        class="mono model-input-main"
                                        list="xuya-fetched-models"
                                        placeholder="claude-opus-4-20250514"
                                    />
                                    <input
                                        v-model="editor.form.opus_name"
                                        class="model-input-name"
                                        placeholder="显示名"
                                        title="显示名称 (影响 Claude Code /model 菜单, 可选)"
                                    />
                                    <label class="model-1m-toggle" title="声明支持 1M 上下文">
                                        <input v-model="editor.form.opus_1m" type="checkbox" />
                                        <span class="model-1m-label">1M</span>
                                    </label>
                                </div>
                                <div class="model-role-row">
                                    <span class="model-role-tag haiku">Haiku</span>
                                    <input
                                        v-model="editor.form.model_haiku"
                                        class="mono model-input-main"
                                        list="xuya-fetched-models"
                                        placeholder="claude-haiku-4-20250514"
                                    />
                                    <input
                                        v-model="editor.form.haiku_name"
                                        class="model-input-name"
                                        placeholder="显示名"
                                        title="显示名称 (影响 Claude Code /model 菜单, 可选)"
                                    />
                                </div>
                            </div>
                            <span class="field-hint">💡 实际模型 = 发给上游的模型 ID（可勾选 1M 启用百万上下文）；显示名 = Claude Code <code>/model</code> 菜单里的友好名称（可选）</span>

                            <label class="cli-field">
                                <span>默认兜底模型（可选）</span>
                                <input v-model="editor.form.model" class="mono" list="xuya-fetched-models" placeholder="留空则按 Sonnet → Opus → Haiku 取主模型" />
                                <span class="field-hint">💡 Claude Code 按角色分别使用模型 (ANTHROPIC_DEFAULT_*_MODEL)。勾选 1M 会给对应模型名追加 [1M] 后缀，启用百万 token 上下文窗口。</span>
                            </label>
                            <span v-if="fetchedModels.length" class="field-hint">✅ 已获取 {{ fetchedModels.length }} 个模型，可在输入框下拉选择</span>
                        </div>

                        <!-- ===== 分区: Codex 凭据 (仅 Codex) ===== -->
                        <div v-if="editor.appTab === 'codex'" class="form-section">
                            <div class="form-section-title">Codex 凭据</div>
                            <label class="cli-field">
                                <span><span class="file-badge">auth.json</span> <span class="req">*</span></span>
                                <textarea
                                    v-model="editor.form.codex_auth_json"
                                    class="cli-textarea mono"
                                    rows="6"
                                    :placeholder='codexAuthPlaceholder'
                                ></textarea>
                                <span class="field-hint">Codex 凭据配置，API Key 会自动填入 OPENAI_API_KEY</span>
                            </label>
                            <label class="cli-field">
                                <span><span class="file-badge">config.toml</span></span>
                                <textarea
                                    v-model="editor.form.codex_config_toml"
                                    class="cli-textarea mono"
                                    rows="8"
                                    :placeholder='codexConfigPlaceholder'
                                ></textarea>
                                <span class="field-hint">Codex 行为配置，留空则使用默认</span>
                            </label>
                        </div>

                        <!-- ===== 高级选项折叠 ===== -->
                        <div class="advanced-toggle" @click="advancedOpen = !advancedOpen">
                            <span class="chevron" :class="{ open: advancedOpen }">▶</span>
                            <span>高级选项</span>
                            <span class="advanced-toggle-hint">认证字段、API 格式、User-Agent</span>
                        </div>
                        <div v-if="advancedOpen" class="advanced-section form-section">
                            <!-- Claude 认证字段 -->
                            <label v-if="editor.appTab === 'claude'" class="cli-field">
                                <span>认证字段</span>
                                <select v-model="editor.form.auth_field">
                                    <option value="ANTHROPIC_AUTH_TOKEN">ANTHROPIC_AUTH_TOKEN（默认）</option>
                                    <option value="ANTHROPIC_API_KEY">ANTHROPIC_API_KEY</option>
                                </select>
                                <span class="field-hint">选择写入配置的认证环境变量名</span>
                            </label>

                            <!-- API 格式 -->
                            <label class="cli-field">
                                <span>API 格式</span>
                                <select v-model="editor.form.api_format">
                                    <option value="anthropic">Anthropic Messages（原生）</option>
                                    <option value="openai_chat">OpenAI Chat Completions（需转换）</option>
                                    <option value="openai_responses">OpenAI Responses API（需转换）</option>
                                    <option value="gemini_native">Gemini Native（需转换）</option>
                                </select>
                                <span class="field-hint">选择供应商 API 的输入格式</span>
                            </label>

                            <!-- 自定义 User-Agent -->
                            <label class="cli-field">
                                <span>自定义 User-Agent（可选）</span>
                                <input v-model="editor.form.custom_user_agent" class="mono" placeholder="Mozilla/5.0 ..." />
                            </label>
                        </div>
                    </div>

                    <div class="cli-modal-footer">
                        <button class="cli-modal-btn secondary" @click="closeEditor">取消</button>
                        <button
                            v-if="!editor.isEdit && editor.step === 'form'"
                            class="cli-modal-btn primary"
                            :disabled="!editor.form.name.trim()"
                            @click="onSave"
                        >添加</button>
                        <button
                            v-else-if="editor.isEdit"
                            class="cli-modal-btn primary"
                            :disabled="!editor.form.name.trim()"
                            @click="onSave"
                        >保存</button>
                    </div>
                </div>
            </div>
        </Transition>

        <!-- 结果提示 toast -->
        <Transition name="fade">
            <div v-if="toast.visible" class="cli-toast" :class="toast.type">{{ toast.message }}</div>
        </Transition>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue';
import { useCliConfig, type CliProvider, type AppType, type ProviderScope, type FetchedModel } from '../../composables/useCliConfig';
import {
    CLAUDE_PRESETS,
    CODEX_PRESETS,
    CATEGORY_LABELS,
    brandColor,
    iconText,
    iconSvg,
    hasIconImage,
    type ProviderPreset,
} from '../../config/providerPresets';

const {
    providers,
    status,
    refreshAll,
    saveProvider,
    deleteProvider,
    switchProvider,
    newProviderTemplate,
    proxyStatus,
    switchProxyTarget,
    fetchModels,
    getLiveConfig,
    openConfigFile,
} = useCliConfig();

// ---------- 配置文件预览 ----------
const configViewer = reactive<{ app: 'claude' | 'codex'; open: boolean; loading: boolean; path: string; content: string }>({
    app: 'claude',
    open: false,
    loading: false,
    path: '',
    content: '',
});

async function loadConfigViewer(app: 'claude' | 'codex', force = false) {
    // 切换 tab 时若已是打开状态则刷新; 否则首次点击展开
    if (configViewer.app === app && configViewer.open && !force) {
        // 同一 tab 再次点击 -> 折叠
        configViewer.open = false;
        return;
    }
    configViewer.app = app;
    configViewer.open = true;
    configViewer.loading = true;
    configViewer.content = '';
    configViewer.path = '';
    try {
        const res = await getLiveConfig(app);
        configViewer.path = res.path;
        configViewer.content = res.content;
    } catch (e) {
        configViewer.content = `读取失败: ${e}`;
    } finally {
        configViewer.loading = false;
    }
}

async function onOpenConfig(app: 'claude' | 'codex') {
    try {
        await openConfigFile(app);
        showToast('已在系统编辑器中打开', 'success');
    } catch (e) {
        showToast(`打开失败: ${e}`, 'error');
    }
}

// ---------- 徽标文案 ----------
const claudeBadgeClass = computed(() => {
    if (!status.value?.claude.installed) return 'off';
    return status.value?.claude.matched_provider_id ? 'on' : 'warn';
});
const claudeBadgeText = computed(() => {
    const s = status.value?.claude;
    if (!s?.installed) return '未安装';
    return s.matched_provider_id ? '已匹配' : '未匹配';
});
const codexBadgeClass = computed(() => {
    if (!status.value?.codex.installed) return 'off';
    return status.value?.codex.matched_provider_id ? 'on' : 'warn';
});
const codexBadgeText = computed(() => {
    const s = status.value?.codex;
    if (!s?.installed) return '未安装';
    return s.matched_provider_id ? '已匹配' : '未匹配';
});

// ---------- 适用范围判定 ----------
const appliesClaude = (p: CliProvider) => p.scope === 'claude' || p.scope === 'both';
const appliesCodex = (p: CliProvider) => p.scope === 'codex' || p.scope === 'both';

/** 该 provider 是否为某 app 的当前活动 provider (用于卡片高亮) */
const isActiveProvider = (p: CliProvider) => {
    return status.value?.claude.matched_provider_id === p.id
        || status.value?.codex.matched_provider_id === p.id;
};

const scopeLabel = (s: ProviderScope) => ({ claude: 'Claude', codex: 'Codex', both: '通用' }[s]);

// ---------- 编辑器 (两步式: 预设首屏 -> 表单) ----------
const emptyForm = (): CliProvider => ({
    id: '', name: '', scope: 'claude', kind: 'relay', category: 'custom',
    base_url: '', api_key: '', model: '',
    model_sonnet: '', model_haiku: '', model_opus: '',
    sonnet_name: '', opus_name: '', haiku_name: '',
    sonnet_1m: false, opus_1m: false, haiku_1m: false,
    note: '', website_url: '',
    auth_field: 'ANTHROPIC_AUTH_TOKEN', api_format: 'anthropic',
    custom_user_agent: '', models_url: '', preset_id: '',
    icon: '', icon_color: '', codex_auth_json: '', codex_config_toml: '',
    updated_at: 0,
});

const editor = reactive({
    visible: false,
    isEdit: false,
    /** 'preset' = 预设首屏, 'form' = 表单 */
    step: 'preset' as 'preset' | 'form',
    /** 当前应用 Tab */
    appTab: 'claude' as 'claude' | 'codex',
    form: emptyForm(),
});

// 预设选择器状态
const presetSearch = ref('');
const advancedOpen = ref(false);
const lastPresetName = ref('');

// ---------- 模型获取 ----------
const fetchedModels = ref<FetchedModel[]>([]);
const modelsLoading = ref(false);
/** datalist id (页面唯一) */
const MODEL_DATALIST_ID = 'xuya-fetched-models';

// ---------- 1M 上下文 & 模型展示 helpers ----------
/** 剥离模型名末尾的 [1M] 后缀 (展示用) */
function strip1M(model: string): string {
    if (!model) return '';
    const m = model.trimEnd();
    return m.toLowerCase().endsWith('[1m]') ? m.slice(0, -4).trimEnd() : m;
}
/** 判断模型名是否带 [1M] 后缀 */
function has1M(model: string): boolean {
    return !!model && model.trimEnd().toLowerCase().endsWith('[1m]');
}
/** 提取 provider 的模型摘要 (角色 + 模型名, 用于卡片展示) */
function modelSummary(p: CliProvider): { role: string; model: string; oneM: boolean }[] {
    const out: { role: string; model: string; oneM: boolean }[] = [];
    if (p.model_sonnet) out.push({ role: 'Sonnet', model: strip1M(p.model_sonnet), oneM: p.sonnet_1m || has1M(p.model_sonnet) });
    if (p.model_opus) out.push({ role: 'Opus', model: strip1M(p.model_opus), oneM: p.opus_1m || has1M(p.model_opus) });
    if (p.model_haiku) out.push({ role: 'Haiku', model: strip1M(p.model_haiku), oneM: p.haiku_1m || has1M(p.model_haiku) });
    if (out.length === 0 && p.model) out.push({ role: '默认', model: strip1M(p.model), oneM: has1M(p.model) });
    return out;
}

/** 从当前 base_url + api_key 拉取可用模型列表 */
async function onFetchModels() {
    const baseUrl = editor.form.base_url.trim();
    const apiKey = editor.form.api_key.trim();
    if (!baseUrl) {
        showToast('请先填写 API 端点地址', 'error');
        return;
    }
    if (!apiKey) {
        showToast('请先填写 API Key', 'error');
        return;
    }
    modelsLoading.value = true;
    try {
        const models = await fetchModels(
            baseUrl,
            apiKey,
            editor.form.custom_user_agent || undefined,
            editor.form.models_url || undefined,
        );
        fetchedModels.value = models;
        if (models.length === 0) {
            showToast('端点未返回任何模型', 'success');
        } else {
            showToast(`已获取 ${models.length} 个模型`, 'success');
        }
    } catch (e) {
        const msg = String(e);
        if (msg.includes('HTTP 401') || msg.includes('HTTP 403')) {
            showToast('认证失败：API Key 无效或无权限', 'error');
        } else if (msg.includes('All candidates failed') || msg.includes('HTTP 404') || msg.includes('HTTP 405')) {
            showToast('未找到模型接口，该供应商可能未开放 /v1/models', 'error');
        } else if (msg.includes('timeout') || msg.includes('timed out')) {
            showToast('请求超时，请检查端点地址或网络', 'error');
        } else {
            showToast('获取模型失败：' + msg.slice(0, 80), 'error');
        }
        fetchedModels.value = [];
    } finally {
        modelsLoading.value = false;
    }
}

/** Codex 双编辑器占位文本 */
const codexAuthPlaceholder = JSON.stringify({ OPENAI_API_KEY: 'sk-your-api-key-here' }, null, 2);
const codexConfigPlaceholder = `model = "gpt-5"
model_provider = "custom"`;

/** 当前 Tab 对应的预设列表 */
const availablePresets = computed(() => {
    if (editor.isEdit) return [];
    return editor.appTab === 'claude' ? CLAUDE_PRESETS : CODEX_PRESETS;
});

/** 搜索过滤后的预设 */
const filteredPresets = computed(() => {
    const kw = presetSearch.value.trim().toLowerCase();
    if (!kw) return availablePresets.value;
    return availablePresets.value.filter((p) => p.name.toLowerCase().includes(kw));
});

/** 当前选中的预设对象 */
const selectedPreset = computed(() => {
    if (!editor.form.preset_id) return null;
    return [...CLAUDE_PRESETS, ...CODEX_PRESETS].find((p) => p.id === editor.form.preset_id) ?? null;
});

/** 切换应用 Tab, 清空搜索 */
const switchAppTab = (tab: 'claude' | 'codex') => {
    editor.appTab = tab;
    presetSearch.value = '';
};

/** 选中预设 -> 自动填充字段 -> 跳到表单步骤 */
const selectPreset = (preset: ProviderPreset) => {
    editor.form.preset_id = preset.id;
    editor.form.name = preset.name;
    lastPresetName.value = preset.name;
    editor.form.base_url = preset.baseUrl;
    editor.form.website_url = preset.websiteUrl;
    editor.form.category = preset.category;
    editor.form.kind = preset.category === 'official' ? 'official' : 'relay';
    editor.form.icon = preset.icon || '';
    editor.form.icon_color = brandColor(preset.icon);
    editor.form.scope = preset.scope === 'both' ? editor.appTab : preset.scope;
    editor.form.models_url = preset.modelsUrl || '';
    if (preset.authField) editor.form.auth_field = preset.authField;
    if (preset.apiFormat) editor.form.api_format = preset.apiFormat;
    if (preset.model) editor.form.model = preset.model;
    editor.step = 'form';
};

const openAdd = async () => {
    editor.form = await newProviderTemplate();
    editor.isEdit = false;
    editor.step = 'preset';
    editor.appTab = 'claude';
    presetSearch.value = '';
    advancedOpen.value = false;
    lastPresetName.value = '';
    fetchedModels.value = [];
    editor.visible = true;
};

const onEdit = (p: CliProvider) => {
    editor.form = { ...p };
    editor.isEdit = true;
    editor.appTab = p.scope === 'codex' ? 'codex' : 'claude';
    advancedOpen.value = false;
    fetchedModels.value = [];
    editor.visible = true;
};

const closeEditor = () => {
    editor.visible = false;
};

const onSave = async () => {
    try {
        await saveProvider(editor.form);
        showToast('已保存', 'success');
        closeEditor();
    } catch (e) {
        showToast('保存失败: ' + e, 'error');
    }
};

const onDelete = async (p: CliProvider) => {
    if (!confirm(`确定删除「${p.name}」吗？`)) return;
    try {
        await deleteProvider(p.id);
        showToast('已删除', 'success');
    } catch (e) {
        showToast('删除失败: ' + e, 'error');
    }
};

const onSwitch = async (app: AppType, id: string) => {
    try {
        const result = await switchProvider(app, id);
        showToast(result.message, result.success ? 'success' : 'error');
    } catch (e) {
        showToast('切换失败: ' + e, 'error');
    }
};

// ---------- 代理 (设为上游, 由 provider 卡片调用) ----------
const onSetTarget = async (p: CliProvider) => {
    try {
        await switchProxyTarget(p);
        showToast(`上游已设为 ${p.name}`, 'success');
    } catch (e) {
        showToast('设置上游失败: ' + e, 'error');
    }
};

// ---------- toast ----------
const toast = reactive({ visible: false, message: '', type: 'success' });
let toastTimer: number | undefined;
const showToast = (message: string, type: 'success' | 'error' = 'success') => {
    toast.message = message;
    toast.type = type;
    toast.visible = true;
    if (toastTimer) window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => { toast.visible = false; }, 2500);
};

onMounted(() => {
    refreshAll();
});

onUnmounted(() => {
    if (toastTimer) window.clearTimeout(toastTimer);
});
</script>

<style scoped>
.cli-panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.cli-refresh-btn {
    padding: 4px 12px;
    font-size: 12px;
    border-radius: 6px;
    cursor: pointer;
    background: var(--xuya-card-bg, transparent);
    color: var(--xuya-text, inherit);
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.2));
    transition: background 0.15s;
}

.cli-refresh-btn:not(:disabled):hover {
    background: rgba(59, 130, 246, 0.1);
    color: #3b82f6;
}

.cli-refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* 状态卡网格 */
.cli-status-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
}

.cli-status-card {
    padding: 12px;
    border-radius: 12px;
    background: var(--xuya-card-bg, rgba(127, 127, 127, 0.08));
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.15));
    transition: border-color 0.2s, box-shadow 0.2s;
}

.cli-status-card.active {
    border-color: #10b981;
    box-shadow: 0 0 0 1px rgba(16, 185, 129, 0.2), 0 4px 12px rgba(16, 185, 129, 0.08);
}

.cli-status-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
}

.cli-app-name-wrap {
    display: flex;
    align-items: center;
    gap: 6px;
}

.cli-app-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    line-height: 0;
}

.cli-app-icon :deep(svg) {
    width: 18px;
    height: 18px;
}

.cli-app-name {
    font-weight: 600;
    font-size: 14px;
}

.cli-badge {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 8px;
}

.cli-badge.on {
    background: rgba(16, 185, 129, 0.18);
    color: #10b981;
}

.cli-badge.warn {
    background: rgba(245, 158, 11, 0.18);
    color: #f59e0b;
}

.cli-badge.off {
    background: rgba(127, 127, 127, 0.18);
    color: #888;
}

.cli-status-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
    margin-top: 6px;
    gap: 8px;
}

.cli-status-label {
    color: var(--xuya-text-secondary, #888);
    flex-shrink: 0;
}

.cli-status-val {
    text-align: right;
    word-break: break-all;
}

.cli-status-val.small {
    font-size: 10px;
    opacity: 0.7;
}

/* 状态卡当前模型 pills */
.cli-status-val.model-pills {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 3px;
}

.live-model-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 10.5px;
    font-weight: 500;
    padding: 1px 7px;
    border-radius: 5px;
    background: rgba(127, 127, 127, 0.1);
    color: var(--xuya-text, #444);
}

.onem-badge {
    display: inline-block;
    padding: 0 4px;
    border-radius: 3px;
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
    font-weight: 700;
    font-size: 9px;
}

/* 配置文件预览 */
.config-viewer {
    border-radius: 10px;
    background: var(--xuya-card-bg, rgba(127, 127, 127, 0.08));
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.15));
    overflow: hidden;
}
.config-viewer-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 8px 0;
    gap: 8px;
}
.config-viewer-tabs {
    display: inline-flex;
    background: var(--xuya-input-bg, rgba(127, 127, 127, 0.08));
    border-radius: 8px;
    padding: 2px;
    gap: 2px;
}
.config-tab {
    padding: 5px 14px;
    font-size: 12px;
    border: none;
    background: transparent;
    color: var(--xuya-text-secondary, #888);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.12s;
}
.config-tab.active {
    background: var(--xuya-bg-elevated, #fff);
    color: var(--xuya-accent, #3b82f6);
    font-weight: 600;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}
.config-viewer-actions {
    display: flex;
    gap: 6px;
    padding-bottom: 8px;
}
.config-viewer-body {
    padding: 0 12px 12px;
}
.config-viewer-path {
    font-size: 10.5px;
    color: var(--xuya-text-tertiary, #888);
    padding: 6px 2px 8px;
    word-break: break-all;
}
.config-viewer-code {
    margin: 0;
    padding: 12px;
    max-height: 280px;
    overflow: auto;
    background: var(--xuya-input-bg, rgba(127, 127, 127, 0.06));
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.12));
    border-radius: 8px;
    font-family: var(--xuya-font-mono, 'Consolas', 'Monaco', monospace);
    font-size: 11.5px;
    line-height: 1.55;
    color: var(--xuya-text, inherit);
    white-space: pre-wrap;
    word-break: break-all;
}

/* 代理面板 */
.proxy-panel {
    padding: 14px;
    border-radius: 10px;
    background: var(--xuya-card-bg, rgba(127, 127, 127, 0.08));
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.15));
}

.proxy-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
}

.proxy-head-info {
    display: flex;
    align-items: center;
    gap: 8px;
}

.proxy-title {
    font-weight: 600;
    font-size: 14px;
}

.proxy-status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #888;
    transition: all 0.3s;
}

.proxy-status-dot.on {
    background: #34c759;
    box-shadow: 0 0 6px rgba(52, 199, 89, 0.6);
    animation: proxy-pulse 2s ease-in-out infinite;
}

@keyframes proxy-pulse {
    0%, 100% { box-shadow: 0 0 6px rgba(52, 199, 89, 0.6); }
    50% { box-shadow: 0 0 10px rgba(52, 199, 89, 0.9); }
}

.proxy-status-text {
    font-size: 12px;
    color: var(--xuya-text-secondary, #888);
}

.proxy-toggle-btn {
    padding: 6px 18px;
    font-size: 12px;
    font-weight: 500;
    border-radius: 7px;
    cursor: pointer;
    background: linear-gradient(135deg, #3b82f6, #6366f1);
    color: #fff;
    border: none;
    transition: transform 0.15s ease, box-shadow 0.15s ease, opacity 0.15s;
    box-shadow: 0 2px 6px rgba(59, 130, 246, 0.3);
}

.proxy-toggle-btn:not(:disabled):hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 10px rgba(59, 130, 246, 0.4);
}

.proxy-toggle-btn.running {
    background: linear-gradient(135deg, #ef4444, #f97316);
    box-shadow: 0 2px 6px rgba(239, 68, 68, 0.3);
}

.proxy-toggle-btn.running:not(:disabled):hover {
    box-shadow: 0 4px 10px rgba(239, 68, 68, 0.4);
}

.proxy-toggle-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* 统计面板 */
.usage-panel {
    padding: 14px;
    border-radius: 10px;
    background: var(--xuya-card-bg, rgba(127, 127, 127, 0.08));
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.15));
}

.usage-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
}

.usage-title {
    font-weight: 600;
    font-size: 14px;
}

.usage-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    margin-bottom: 14px;
}

.usage-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 10px 4px;
    border-radius: 10px;
    background: var(--xuya-bg, rgba(127, 127, 127, 0.05));
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.08));
    transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.usage-stat:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 10px rgba(0, 0, 0, 0.06);
}

.usage-stat-val {
    font-size: 16px;
    font-weight: 700;
}

.usage-stat-val.ok {
    color: #34c759;
}

.usage-stat-val.err {
    color: #ef4444;
}

.usage-stat-label {
    font-size: 10px;
    opacity: 0.6;
    margin-top: 2px;
}

.logs-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 8px;
    padding-top: 10px;
    border-top: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.1));
}

.logs-head-actions {
    display: flex;
    align-items: center;
    gap: 10px;
}

.logs-count {
    font-weight: 400;
    font-size: 11px;
    opacity: 0.6;
}

.logs-page {
    font-size: 10px;
    opacity: 0.6;
}

.logs-table {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 200px;
    overflow-y: auto;
}

.logs-row {
    display: grid;
    grid-template-columns: 1fr 0.7fr 1.2fr 0.6fr 0.8fr 0.6fr;
    gap: 6px;
    padding: 4px 6px;
    font-size: 11px;
    border-radius: 4px;
    align-items: center;
}

.logs-row-head {
    font-weight: 600;
    opacity: 0.6;
    font-size: 10px;
    border-bottom: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.1));
    padding-bottom: 4px;
    margin-bottom: 2px;
}

.logs-row:not(.logs-row-head) {
    background: rgba(127, 127, 127, 0.04);
}

.logs-row .ok {
    color: #34c759;
}

.logs-row .err {
    color: #ef4444;
}

.logs-row .small {
    font-size: 10px;
    opacity: 0.8;
}

.proxy-detail {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 0;
    border-top: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.1));
}

.proxy-detail-row {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
}

.proxy-detail-label {
    color: var(--xuya-text-secondary, #888);
}

.proxy-takeover {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 10px;
    margin-top: 4px;
    border-top: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.1));
}

.proxy-takeover-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 12px;
}

/* 内部 switch (CliPanel scoped 自带, 不依赖父组件) */
.cli-panel .switch {
    position: relative;
    display: inline-block;
    width: 36px;
    height: 20px;
}

.cli-panel .switch input {
    opacity: 0;
    width: 0;
    height: 0;
}

.cli-panel .slider {
    position: absolute;
    cursor: pointer;
    inset: 0;
    background-color: rgba(127, 127, 127, 0.4);
    border-radius: 20px;
    transition: 0.3s;
}

.cli-panel .slider::before {
    content: "";
    position: absolute;
    height: 14px;
    width: 14px;
    left: 3px;
    bottom: 3px;
    background-color: #fff;
    border-radius: 50%;
    transition: 0.3s;
}

.cli-panel .switch input:checked + .slider {
    background-color: #34c759;
}

.cli-panel .switch input:checked + .slider::before {
    transform: translateX(16px);
}

/* provider 列表 */
.cli-providers-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 8px;
}

.cli-add-btn {
    padding: 5px 12px;
    font-size: 12px;
    font-weight: 500;
    border-radius: 7px;
    cursor: pointer;
    background: linear-gradient(135deg, #3b82f6, #6366f1);
    color: #fff;
    border: none;
    transition: transform 0.15s ease, box-shadow 0.15s ease, opacity 0.15s;
    box-shadow: 0 2px 6px rgba(59, 130, 246, 0.3);
}

.cli-add-btn:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 10px rgba(59, 130, 246, 0.4);
}

.cli-add-btn:active {
    transform: translateY(0);
    opacity: 0.9;
}

.cli-empty {
    padding: 20px;
    text-align: center;
    font-size: 12px;
    color: var(--xuya-text-secondary, #888);
}

.cli-provider-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.cli-provider-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-radius: 8px;
    background: var(--xuya-card-bg, rgba(127, 127, 127, 0.06));
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.1));
    gap: 12px;
}

.cli-provider-main {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    flex: 1;
}

.cli-provider-name {
    font-weight: 600;
    font-size: 13px;
}

.cli-provider-tags {
    display: flex;
    gap: 6px;
}

.cli-tag {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
    background: rgba(127, 127, 127, 0.15);
}

.cli-tag.kind {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
}

.cli-provider-url {
    font-size: 11px;
    opacity: 0.7;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.cli-provider-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
}

.cli-mini-btn {
    padding: 4px 10px;
    font-size: 11px;
    border-radius: 6px;
    cursor: pointer;
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
    transition: all 0.15s;
}

.cli-mini-btn:hover {
    background: rgba(59, 130, 246, 0.25);
}

.cli-mini-btn.current {
    background: #10b981;
    color: #fff;
}

.cli-mini-btn.ghost {
    background: transparent;
    color: var(--xuya-text-secondary, #888);
}

.cli-mini-btn.danger {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
}

.cli-mini-btn.danger:hover {
    background: rgba(239, 68, 68, 0.25);
}

/* 弹窗 */
.cli-modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}

.cli-modal-card {
    width: 440px;
    max-width: 90vw;
    max-height: 80vh;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--xuya-bg, #fff);
    border-radius: 12px;
    padding: 20px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    /* 滚动条不破坏圆角: 细滚动条 + 留出右侧内边距 */
    scrollbar-width: thin;
    scrollbar-color: rgba(127, 127, 127, 0.35) transparent;
}

/* Webkit 滚动条细化, 避免右侧方角突兀 */
.cli-modal-card::-webkit-scrollbar {
    width: 6px;
}
.cli-modal-card::-webkit-scrollbar-track {
    background: transparent;
    margin: 12px 0;
}
.cli-modal-card::-webkit-scrollbar-thumb {
    background: rgba(127, 127, 127, 0.35);
    border-radius: 3px;
}
.cli-modal-card::-webkit-scrollbar-thumb:hover {
    background: rgba(127, 127, 127, 0.55);
}

/* 加宽版弹窗 (含预设选择器) */
.cli-modal-large {
    width: 600px;
}

/* 预设选择器 */
.preset-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 4px;
}

.preset-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
}

.preset-title {
    font-size: 13px;
    font-weight: 600;
}

.preset-search {
    width: 180px;
    padding: 4px 8px;
    font-size: 12px;
    border-radius: 6px;
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.2));
    background: var(--xuya-input-bg, transparent);
    color: var(--xuya-text, inherit);
}

.preset-hint {
    font-size: 11px;
    padding: 6px 10px;
    border-radius: 6px;
    background: rgba(59, 130, 246, 0.1);
    color: #3b82f6;
}

.preset-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 8px;
    max-height: 320px;
    overflow-y: auto;
    padding: 2px;
}

/* 应用切换 Tab (segmented control + 品牌图标) */
.app-tabs {
    display: flex;
    gap: 2px;
    padding: 4px;
    background: rgba(127, 127, 127, 0.1);
    border-radius: 10px;
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.08));
}

.app-tab {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 9px 8px;
    font-size: 13px;
    font-weight: 500;
    border-radius: 7px;
    cursor: pointer;
    background: transparent;
    color: var(--xuya-text-secondary, #888);
    border: none;
    transition: all 0.18s ease;
    position: relative;
}

.app-tab:not(.active):hover {
    background: rgba(127, 127, 127, 0.08);
    color: var(--xuya-text, #444);
}

.app-tab .tab-icon {
    width: 16px;
    height: 16px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    opacity: 0.6;
    transition: opacity 0.15s;
}

.app-tab .tab-icon :deep(svg) {
    width: 16px;
    height: 16px;
}

.app-tab.active {
    background: var(--xuya-bg, #fff);
    color: #3b82f6;
    font-weight: 600;
    box-shadow: 0 2px 6px rgba(59, 130, 246, 0.15);
}

.app-tab.active .tab-icon {
    opacity: 1;
}

.app-tab.active::after {
    content: "";
    position: absolute;
    bottom: 2px;
    left: 30%;
    right: 30%;
    height: 2px;
    background: linear-gradient(90deg, #3b82f6, #6366f1);
    border-radius: 1px;
}

/* 返回按钮 */
/* 返回预设按钮 (圆角胶囊 + 箭头 + hover 效果) */
.cli-back-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    font-weight: 500;
    color: #3b82f6;
    background: rgba(59, 130, 246, 0.08);
    border: 1px solid rgba(59, 130, 246, 0.25);
    border-radius: 18px;
    padding: 5px 14px;
    cursor: pointer;
    transition: all 0.18s ease;
    white-space: nowrap;
    flex-shrink: 0;
}

.cli-back-btn:hover {
    background: rgba(59, 130, 246, 0.16);
    border-color: rgba(59, 130, 246, 0.5);
    transform: translateX(-2px);
}

.cli-back-btn:active {
    transform: translateX(0);
}

/* 已选预设提示条 */
.selected-preset-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 8px;
    background: rgba(59, 130, 246, 0.08);
    font-size: 13px;
}

.selected-preset-cat {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
    background: rgba(127, 127, 127, 0.15);
}

/* 图标圆标 (provider 卡片 + 选中条复用) */
/* 不设默认底色: 图标为 currentColor 单色, 由容器 color 指定品牌原色, 原样展示 */
.provider-icon {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    font-weight: 700;
    flex-shrink: 0;
    overflow: hidden;
    background: rgba(127, 127, 127, 0.06);
}

.provider-icon.sm {
    width: 28px;
    height: 28px;
    border-radius: 7px;
    font-size: 12px;
}

/* 内联 SVG 图标 (currentColor 继承容器 color = 品牌原色) */
.provider-icon-svg {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 72%;
    height: 72%;
    line-height: 0;
    pointer-events: none;
    user-select: none;
}

.provider-icon-svg :deep(svg) {
    width: 100%;
    height: 100%;
}

/* Codex 双编辑器 textarea */
.cli-textarea {
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 12px;
    padding: 10px;
    border-radius: 6px;
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.2));
    background: var(--xuya-input-bg, transparent);
    color: var(--xuya-text, inherit);
    resize: vertical;
    line-height: 1.5;
}

/* provider 卡片网格 */
.provider-card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 10px;
}

.provider-card {
    position: relative;
    padding: 16px 14px 14px 18px;
    border-radius: 12px;
    background: var(--xuya-card-bg, rgba(127, 127, 127, 0.06));
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.1));
    display: flex;
    flex-direction: column;
    gap: 10px;
    transition: transform 0.18s ease, box-shadow 0.18s ease, border-color 0.2s;
    overflow: hidden;
}

/* 左侧品牌色 accent 竖条 */
.provider-card::before {
    content: "";
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 4px;
    background: linear-gradient(180deg, #3b82f6, #6366f1);
    opacity: 0.5;
    transition: opacity 0.2s;
}

.provider-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 10px 24px rgba(0, 0, 0, 0.1);
    border-color: rgba(59, 130, 246, 0.4);
}

.provider-card:hover::before {
    opacity: 1;
}

.provider-card.active {
    border-color: #10b981;
    box-shadow: 0 0 0 1px rgba(16, 185, 129, 0.25), 0 6px 16px rgba(16, 185, 129, 0.1);
}

.provider-card.active::before {
    background: linear-gradient(180deg, #10b981, #059669);
    opacity: 1;
}

/* 「当前」角标 (右上圆角贴边, 左下圆角, 整体对称) */
.provider-current-badge {
    position: absolute;
    top: 0;
    right: 0;
    background: linear-gradient(135deg, #10b981, #059669);
    color: #fff;
    font-size: 10px;
    font-weight: 600;
    padding: 3px 12px;
    border-radius: 0 12px 0 12px;
    z-index: 1;
}

/* 模型摘要 tag 行 */
.provider-models {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
}

.model-tag {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 10.5px;
    padding: 2px 7px;
    border-radius: 5px;
    background: rgba(127, 127, 127, 0.1);
    color: var(--xuya-text, #444);
    font-weight: 500;
    line-height: 1.5;
}

.model-tag.sonnet { background: rgba(217, 119, 87, 0.13); color: #d97757; }
.model-tag.opus { background: rgba(139, 92, 246, 0.13); color: #8b5cf6; }
.model-tag.haiku { background: rgba(16, 185, 129, 0.13); color: #10b981; }
.model-tag.default { background: rgba(127, 127, 127, 0.12); color: var(--xuya-text-secondary, #888); font-style: italic; }

.model-1m-mark {
    font-weight: 700;
    color: #f59e0b;
}

.provider-card-head {
    display: flex;
    align-items: center;
    gap: 10px;
}

.provider-card-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
}

.provider-card-name {
    font-size: 14px;
    font-weight: 600;
}

.provider-card-cat {
    font-size: 10px;
    opacity: 0.7;
}

.provider-card-url {
    font-size: 11px;
    opacity: 0.6;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* 额度展示区 */
.provider-quota {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 6px 8px;
    border-radius: 8px;
    background: rgba(127, 127, 127, 0.06);
    font-size: 11px;
}

.quota-loading {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    opacity: 0.7;
}

.mini-spinner {
    width: 10px;
    height: 10px;
    border: 1.5px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: fetch-spin 0.7s linear infinite;
}

.quota-pill {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: 5px;
    font-weight: 500;
    white-space: nowrap;
}

.quota-pill.ok {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
}

.quota-pill.warn {
    background: rgba(245, 158, 11, 0.15);
    color: #f59e0b;
}

.quota-pill.danger,
.quota-pill.err,
.quota-pill.invalid {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
}

.provider-card-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
}

.preset-tile {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px;
    border-radius: 10px;
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.15));
    background: var(--xuya-card-bg, rgba(127, 127, 127, 0.04));
    cursor: pointer;
    transition: transform 0.15s ease, border-color 0.15s, background 0.15s, box-shadow 0.15s;
}

.preset-tile:hover {
    border-color: #3b82f6;
    background: rgba(59, 130, 246, 0.06);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.12);
}

.preset-tile:active {
    transform: translateY(0);
}

.preset-tile-icon {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    font-weight: 700;
    flex-shrink: 0;
    overflow: hidden;
    background: rgba(127, 127, 127, 0.06);
}

.preset-tile-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
}

.preset-tile-name {
    font-size: 12px;
    font-weight: 600;
    line-height: 1.3;
}

.preset-tile-cat {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
    align-self: flex-start;
    background: rgba(127, 127, 127, 0.15);
    color: var(--xuya-text-secondary, #888);
}

.preset-tile-cat.official,
.preset-tile-cat.cn_official {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
}

.preset-tile-cat.third_party,
.preset-tile-cat.aggregator {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
}

/* 表单网格 (两列) */
.form-grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
}

.req {
    color: #ef4444;
    font-weight: 600;
}

.field-hint {
    font-size: 10px;
    opacity: 0.6;
    line-height: 1.4;
}

/* ===== 表单分区卡片 ===== */
.form-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
    border-radius: 10px;
    background: var(--xuya-card-bg, rgba(127, 127, 127, 0.04));
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.1));
}

.form-section-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 13px;
    font-weight: 600;
    padding-left: 10px;
    position: relative;
}

.form-section-title::before {
    content: "";
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 14px;
    background: linear-gradient(180deg, #3b82f6, #6366f1);
    border-radius: 2px;
}

/* ===== 获取模型按钮 ===== */
.fetch-models-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 500;
    border-radius: 6px;
    cursor: pointer;
    border: 1px solid rgba(59, 130, 246, 0.3);
    background: rgba(59, 130, 246, 0.08);
    color: #3b82f6;
    transition: all 0.15s ease;
}

.fetch-models-btn:not(:disabled):hover {
    background: rgba(59, 130, 246, 0.16);
    border-color: rgba(59, 130, 246, 0.5);
}

.fetch-models-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
}

.fetch-spinner {
    width: 11px;
    height: 11px;
    border: 1.5px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: fetch-spin 0.7s linear infinite;
}

@keyframes fetch-spin {
    to { transform: rotate(360deg); }
}

/* ===== 模型角色行 ===== */
.model-role-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.model-role-row {
    display: flex;
    align-items: center;
    gap: 8px;
}

/* 实际模型输入框 (主要, 占更多空间) */
.model-role-row input.model-input-main {
    flex: 2;
    min-width: 0;
}

/* 显示名输入框 (较窄, 斜体占位) */
.model-role-row input.model-input-name {
    flex: 1;
    min-width: 80px;
    max-width: 130px;
    font-size: 11.5px;
    border-style: dashed;
    opacity: 0.85;
}

.model-role-row input.model-input-name:focus {
    opacity: 1;
    border-style: solid;
}

.model-role-row input {
    padding: 7px 10px;
    border-radius: 6px;
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.2));
    background: var(--xuya-input-bg, transparent);
    color: var(--xuya-text, inherit);
    font-size: 12px;
    transition: border-color 0.15s, opacity 0.15s;
}

.model-role-row input:focus {
    outline: none;
    border-color: #3b82f6;
}

.model-role-tag {
    width: 56px;
    flex-shrink: 0;
    text-align: center;
    font-size: 11px;
    font-weight: 600;
    padding: 4px 0;
    border-radius: 5px;
}

.model-role-tag.sonnet {
    background: rgba(217, 119, 87, 0.15);
    color: #d97757;
}

.model-role-tag.haiku {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
}

.model-role-tag.opus {
    background: rgba(139, 92, 246, 0.15);
    color: #8b5cf6;
}

/* 1M 上下文切换 */
.model-1m-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    flex-shrink: 0;
    user-select: none;
}

.model-1m-toggle input[type="checkbox"] {
    width: 13px;
    height: 13px;
    accent-color: #f59e0b;
    cursor: pointer;
    margin: 0;
}

.model-1m-label {
    font-size: 10.5px;
    font-weight: 700;
    color: var(--xuya-text-secondary, #888);
    transition: color 0.15s;
}

.model-1m-toggle input:checked + .model-1m-label {
    color: #f59e0b;
}

/* Codex 文件名徽标 */
.file-badge {
    display: inline-block;
    padding: 1px 7px;
    border-radius: 4px;
    background: rgba(127, 127, 127, 0.15);
    color: var(--xuya-text, inherit);
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 11px;
}

.get-key-link {
    font-size: 11px;
    color: #3b82f6;
    text-decoration: none;
    align-self: flex-start;
}

/* 高级选项折叠 */
.advanced-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 0 8px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    user-select: none;
    border-top: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.1));
    margin-top: 4px;
}

.chevron {
    display: inline-block;
    font-size: 10px;
    transition: transform 0.2s ease;
    color: var(--xuya-text-secondary, #888);
}

.chevron.open {
    transform: rotate(90deg);
}

.advanced-toggle-hint {
    font-weight: 400;
    font-size: 10px;
    opacity: 0.5;
    margin-left: auto;
}

.advanced-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

/* provider 列表分类标签着色 */
.cli-tag.official,
.cli-tag.cn_official {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
}

.cli-tag.third_party,
.cli-tag.aggregator {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
}

.cli-modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 16px;
}

.cli-modal-header h4 {
    margin: 0;
}

.cli-modal-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.cli-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
}

.cli-field span {
    color: var(--xuya-text-secondary, #888);
}

.cli-field input,
.cli-field select {
    padding: 8px 10px;
    border-radius: 6px;
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.2));
    background: var(--xuya-input-bg, transparent);
    color: var(--xuya-text, inherit);
    font-size: 13px;
    transition: border-color 0.15s, box-shadow 0.15s;
}

.cli-field input:focus,
.cli-field select:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.12);
}

.cli-modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
}

.cli-modal-btn {
    padding: 8px 18px;
    border-radius: 7px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    border: none;
    transition: transform 0.15s ease, box-shadow 0.15s ease, opacity 0.15s;
}

.cli-modal-btn.secondary {
    background: transparent;
    color: var(--xuya-text-secondary, #888);
    border: 1px solid var(--xuya-border, rgba(127, 127, 127, 0.2));
}

.cli-modal-btn.secondary:hover {
    background: rgba(127, 127, 127, 0.08);
}

.cli-modal-btn.primary {
    background: linear-gradient(135deg, #3b82f6, #6366f1);
    color: #fff;
    box-shadow: 0 2px 6px rgba(59, 130, 246, 0.3);
}

.cli-modal-btn.primary:not(:disabled):hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 10px rgba(59, 130, 246, 0.4);
}

.cli-modal-btn.primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* toast */
.cli-toast {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    padding: 10px 20px;
    border-radius: 8px;
    font-size: 13px;
    z-index: 1001;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
}

.cli-toast.success {
    background: #10b981;
    color: #fff;
}

.cli-toast.error {
    background: #ef4444;
    color: #fff;
}

.mono {
    font-family: 'Consolas', 'Monaco', monospace;
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.2s;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
