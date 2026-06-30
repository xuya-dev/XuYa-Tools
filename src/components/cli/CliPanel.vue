<template>
    <div class="cli-panel">
        <!-- 当前 live 状态卡 -->
        <div class="cli-status-grid">
            <div class="cli-status-card" :class="{ active: claudeActiveProvider, 'proxy-on': claudeProxyOn }">
                <div class="cli-status-head">
                    <div class="cli-app-name-wrap">
                        <span v-if="iconSvg('claude')" class="cli-app-icon" v-html="iconSvg('claude')!"></span>
                        <span class="cli-app-name">Claude Code</span>
                    </div>
                    <div class="cli-status-badges">
                        <span v-if="claudeProxyOn" class="cli-badge proxy">代理中</span>
                        <span class="cli-badge" :class="claudeBadgeClass">{{ claudeBadgeText }}</span>
                    </div>
                </div>
                <div class="cli-status-row">
                    <span class="cli-status-label">当前 Provider</span>
                    <span class="cli-status-val">{{ claudeActiveProvider || '未匹配' }}</span>
                </div>
                <div class="cli-status-row">
                    <span class="cli-status-label">{{ claudeProxyOn ? '代理地址' : 'Base URL' }}</span>
                    <span class="cli-status-val mono">{{ claudeDisplayUrl || '官方默认' }}</span>
                </div>
                <div v-if="claudeModels.model_sonnet || claudeModels.model_opus || claudeModels.model_haiku || claudeModels.model" class="cli-status-row">
                    <span class="cli-status-label">当前模型</span>
                    <span class="cli-status-val model-pills">
                        <span v-if="claudeModels.model_sonnet" class="live-model-pill">
                            Sonnet {{ strip1M(claudeModels.model_sonnet) }}<span v-if="has1M(claudeModels.model_sonnet)" class="onem-badge">1M</span>
                        </span>
                        <span v-if="claudeModels.model_opus" class="live-model-pill">
                            Opus {{ strip1M(claudeModels.model_opus) }}<span v-if="has1M(claudeModels.model_opus)" class="onem-badge">1M</span>
                        </span>
                        <span v-if="claudeModels.model_haiku" class="live-model-pill">
                            Haiku {{ strip1M(claudeModels.model_haiku) }}
                        </span>
                        <span v-if="!claudeModels.model_sonnet && !claudeModels.model_opus && !claudeModels.model_haiku && claudeModels.model" class="live-model-pill">
                            {{ strip1M(claudeModels.model) }}<span v-if="has1M(claudeModels.model)" class="onem-badge">1M</span>
                        </span>
                    </span>
                </div>
                <div class="cli-status-row">
                    <span class="cli-status-label">配置路径</span>
                    <span class="cli-status-val mono small">{{ status?.claude.config_path || '未检测到' }}</span>
                </div>
            </div>

            <div class="cli-status-card" :class="{ active: codexActiveProvider, 'proxy-on': codexProxyOn }">
                <div class="cli-status-head">
                    <div class="cli-app-name-wrap">
                        <span v-if="iconSvg('openai')" class="cli-app-icon" v-html="iconSvg('openai')!"></span>
                        <span class="cli-app-name">Codex CLI</span>
                    </div>
                    <div class="cli-status-badges">
                        <span v-if="codexProxyOn" class="cli-badge proxy">代理中</span>
                        <span class="cli-badge" :class="codexBadgeClass">{{ codexBadgeText }}</span>
                    </div>
                </div>
                <div class="cli-status-row">
                    <span class="cli-status-label">当前 Provider</span>
                    <span class="cli-status-val">{{ codexActiveProvider || '未匹配' }}</span>
                </div>
                <div class="cli-status-row">
                    <span class="cli-status-label">{{ codexProxyOn ? '代理地址' : 'Base URL' }}</span>
                    <span class="cli-status-val mono">{{ codexDisplayUrl || '官方默认' }}</span>
                </div>
                <div v-if="codexModels" class="cli-status-row">
                    <span class="cli-status-label">当前模型</span>
                    <span class="cli-status-val">
                        <span class="live-model-pill">{{ codexModels }}</span>
                    </span>
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

        <!-- provider 列表 (按 CLI 分组) -->
        <div class="cli-providers">
            <div class="cli-providers-head">
                <div class="provider-tabs">
                    <button
                        class="provider-tab"
                        :class="{ active: providerTab === 'claude' }"
                        @click="providerTab = 'claude'"
                    >
                        <span v-if="iconSvg('claude')" class="tab-icon" v-html="iconSvg('claude')!"></span>
                        Claude
                    </button>
                    <button
                        class="provider-tab"
                        :class="{ active: providerTab === 'codex' }"
                        @click="providerTab = 'codex'"
                    >
                        <span v-if="iconSvg('openai')" class="tab-icon" v-html="iconSvg('openai')!"></span>
                        Codex
                    </button>
                </div>
                <div class="provider-search-wrap">
                    <input
                        v-model="providerSearch"
                        class="provider-search"
                        type="text"
                        placeholder="搜索..."
                        spellcheck="false"
                    />
                </div>
                <button class="cli-add-btn" @click="openAdd">+ 新增</button>
            </div>

            <div v-if="filteredProviders.length === 0" class="cli-empty">
                {{ providerSearch ? `没有匹配 "${providerSearch}" 的 Provider` : `暂无 ${providerTab === 'claude' ? 'Claude Code' : 'Codex CLI'} 的 Provider,点击「新增」创建` }}
            </div>

            <div v-else class="provider-card-grid">
                <div v-for="p in filteredProviders" :key="p.id" class="provider-card" :class="{ active: isCurrentForTab(p) }">
                    <span v-if="isCurrentForTab(p)" class="provider-current-badge">当前</span>
                    <div class="provider-card-head">
                        <div class="provider-icon" :style="{ color: p.icon_color || brandColor(p.icon) }">
                            <span v-if="hasIconImage(p.icon)" class="provider-icon-svg" v-html="iconSvg(p.icon)!"></span>
                            <span v-else>{{ iconText(p.name) }}</span>
                        </div>
                        <div class="provider-card-info">
                            <span class="provider-card-name">{{ p.name }}</span>
                            <span class="provider-card-cat" :class="p.category">{{ CATEGORY_LABELS[p.category] }}<span v-if="p.scope === 'both'" class="scope-badge">通用</span></span>
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

                    <!-- 余额展示区 -->
                    <div v-if="balanceCache[p.id]" class="balance-section">
                        <template v-if="balanceCache[p.id]!.result.success">
                            <!-- 余额制: 金额展示 + 进度条 -->
                            <template v-if="!balanceCache[p.id]!.result.isPlan">
                                <div
                                    v-for="(item, bi) in balanceCache[p.id]!.result.items"
                                    :key="bi"
                                    class="balance-amount-row"
                                >
                                    <template v-if="item.isValid">
                                        <div class="ba-header">
                                            <span class="ba-remaining" :class="{ low: (item.remaining ?? 0) < (item.total ?? 0) * 0.1 }">
                                                {{ item.remaining != null ? item.remaining.toFixed(2) : '-' }}
                                                <span class="ba-unit">{{ item.unit }}</span>
                                            </span>
                                            <span v-if="item.total != null && item.total > 0" class="ba-detail">
                                                已用 {{ item.used?.toFixed(2) }} / {{ item.total.toFixed(2) }}
                                            </span>
                                            <span v-else-if="item.used != null && item.used > 0" class="ba-detail">
                                                已用 {{ item.used.toFixed(2) }}
                                            </span>
                                        </div>
                                        <div v-if="item.total != null && item.total > 0" class="ba-bar">
                                            <div
                                                class="ba-bar-fill"
                                                :style="{ width: Math.min(100, (item.used ?? 0) / item.total * 100) + '%' }"
                                                :class="{ warn: (item.used ?? 0) / item.total > 0.9 }"
                                            ></div>
                                        </div>
                                    </template>
                                    <div v-else class="ba-invalid">
                                        <span class="ba-invalid-icon">⚠</span>
                                        {{ item.invalidMessage || 'API Key 失效' }}
                                    </div>
                                </div>
                            </template>
                            <!-- 套餐制: 多窗口百分比 -->
                            <template v-else>
                                <div class="plan-tiers">
                                    <div
                                        v-for="(item, bi) in balanceCache[p.id]!.result.items"
                                        :key="bi"
                                        class="plan-tier"
                                    >
                                        <template v-if="item.isValid">
                                            <div class="pt-header">
                                                <span class="pt-label">{{ item.label }}</span>
                                                <span class="pt-pct" :class="{ high: (item.used ?? 0) >= 80 }">{{ item.used?.toFixed(0) }}%</span>
                                            </div>
                                            <div class="pt-bar">
                                                <div
                                                    class="pt-bar-fill"
                                                    :style="{ width: Math.min(100, item.used ?? 0) + '%' }"
                                                    :class="{ warn: (item.used ?? 0) >= 80 }"
                                                ></div>
                                            </div>
                                            <div v-if="item.resetsAt" class="pt-reset">重置 {{ formatBalanceTime(item.resetsAt) }}</div>
                                        </template>
                                        <div v-else class="ba-invalid">
                                            <span class="ba-invalid-icon">⚠</span>
                                            {{ item.invalidMessage || '已用尽' }}
                                        </div>
                                    </div>
                                </div>
                            </template>
                        </template>
                        <!-- 查询失败 -->
                        <div v-else class="ba-error">{{ balanceCache[p.id]!.result.error }}</div>
                    </div>

                    <div class="provider-card-actions">
                        <button
                            class="cli-mini-btn"
                            :class="{ current: isCurrentForTab(p) }"
                            :disabled="isCurrentForTab(p)"
                            @click="onSwitch(providerTab, p.id)"
                        >{{ isCurrentForTab(p) ? '当前' : '切换' }}</button>
                        <button
                            v-if="p.base_url && p.category !== 'official'"
                            class="cli-mini-btn ghost"
                            :disabled="balanceLoading[p.id]"
                            @click="onQueryBalance(p)"
                        >
                            <span v-if="balanceLoading[p.id]" class="fetch-spinner"></span>
                            {{ balanceLoading[p.id] ? '' : '余额' }}
                        </button>
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

                        <!-- 编辑模式下: scope='both' 时展示 app-tab 切换器 (P1-1) -->
                        <div v-if="editor.isEdit && editor.form.scope === 'both'" class="app-tabs">
                            <button
                                class="app-tab"
                                :class="{ active: editor.appTab === 'claude' }"
                                @click="editor.appTab = 'claude'"
                            >
                                <span v-if="iconSvg('claude')" class="tab-icon" v-html="iconSvg('claude')!"></span>
                                Claude
                            </button>
                            <button
                                class="app-tab"
                                :class="{ active: editor.appTab === 'codex' }"
                                @click="editor.appTab = 'codex'"
                            >
                                <span v-if="iconSvg('openai')" class="tab-icon" v-html="iconSvg('openai')!"></span>
                                Codex
                            </button>
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
                                <div class="api-key-wrap">
                                    <input
                                        v-model="editor.form.api_key"
                                        class="mono"
                                        :type="apiKeyVisible ? 'text' : 'password'"
                                        autocomplete="off"
                                        :disabled="editor.form.category === 'official'"
                                        :placeholder="editor.form.category === 'official' ? '官方无需 API Key' : '输入 API Key，将自动填充到配置'"
                                    />
                                    <button
                                        v-if="editor.form.api_key && editor.form.category !== 'official'"
                                        type="button"
                                        class="key-toggle"
                                        @click="apiKeyVisible = !apiKeyVisible"
                                    >{{ apiKeyVisible ? '🙈' : '👁' }}</button>
                                </div>
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

                        <!-- ===== 分区: 模型配置 (仅 Codex) ===== -->
                        <div v-if="editor.appTab === 'codex'" class="form-section">
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
                            <label class="cli-field">
                                <span>模型 <span class="req">*</span></span>
                                <input v-model="editor.form.model" class="mono" list="xuya-fetched-models" placeholder="如 gpt-4o、o3、deepseek-chat" />
                                <span class="field-hint">Codex CLI 使用的主模型 ID</span>
                            </label>
                            <span v-if="fetchedModels.length" class="field-hint">✅ 已获取 {{ fetchedModels.length }} 个模型，可在输入框下拉选择</span>
                        </div>

                        <!-- ===== 高级选项折叠 ===== -->
                        <div class="advanced-toggle" @click="advancedOpen = !advancedOpen">
                            <span class="chevron" :class="{ open: advancedOpen }">▶</span>
                            <span>高级选项</span>
                            <span class="advanced-toggle-hint">认证字段、API 格式、User-Agent、原始配置文件、余额查询</span>
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

                            <!-- API 格式 (选项标签按 Tab 动态变化) -->
                            <label class="cli-field">
                                <span>API 格式</span>
                                <select v-model="editor.form.api_format">
                                    <option v-for="opt in apiFormatOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
                                </select>
                                <span class="field-hint">{{ editor.appTab === 'codex' ? 'Codex CLI 原生支持的格式直接直连,其他格式需启动代理转换' : 'Claude Code 原生支持 Anthropic,其他格式需启动代理转换' }}</span>
                            </label>

                            <!-- 自定义 User-Agent -->
                            <label class="cli-field">
                                <span>自定义 User-Agent（可选）</span>
                                <input v-model="editor.form.custom_user_agent" class="mono" placeholder="Mozilla/5.0 ..." />
                            </label>

                            <!-- Claude: settings.json 原始编辑 (高级, 与 Codex 的 auth.json/config.toml 对等) -->
                            <label v-if="editor.appTab === 'claude'" class="cli-field">
                                <span><span class="file-badge">settings.json</span>（高级，留空则自动生成）</span>
                                <textarea
                                    v-model="editor.form.claude_settings_json"
                                    class="cli-textarea mono"
                                    rows="8"
                                    :placeholder="claudeSettingsPreview"
                                ></textarea>
                                <span class="field-hint">上方为根据当前字段实时生成的预览。填入后完全覆盖自动生成的 Claude Code 配置</span>
                            </label>

                            <!-- Codex 原始配置文件 (高级用户) -->
                            <label v-if="editor.appTab === 'codex'" class="cli-field">
                                <span><span class="file-badge">auth.json</span>（高级，留空则自动生成）</span>
                                <textarea
                                    v-model="editor.form.codex_auth_json"
                                    class="cli-textarea mono"
                                    rows="6"
                                    :placeholder="codexAuthPreview"
                                ></textarea>
                                <span class="field-hint">上方为根据当前字段实时生成的预览。填入后完全覆盖自动生成的凭据</span>
                            </label>
                            <label v-if="editor.appTab === 'codex'" class="cli-field">
                                <span><span class="file-badge">config.toml</span>（高级，留空则自动生成）</span>
                                <textarea
                                    v-model="editor.form.codex_config_toml"
                                    class="cli-textarea mono"
                                    rows="8"
                                    :placeholder="codexConfigPreview"
                                ></textarea>
                                <span class="field-hint">上方为根据当前字段实时生成的预览。填入后完全覆盖自动生成的配置</span>
                            </label>

                            <!-- 余额查询类型 (仅自定义分类) -->
                            <label v-if="editor.form.category === 'custom'" class="cli-field">
                                <span>余额查询类型</span>
                                <select v-model="editor.form.quota_provider_type">
                                    <option value="">自动识别</option>
                                    <option value="sub2api">Sub2API</option>
                                    <option value="newapi">New API</option>
                                </select>
                                <span class="field-hint">内置供应商自动识别,自定义供应商可手动指定查询类型</span>
                            </label>

                            <!-- New API 额外字段 -->
                            <template v-if="editor.form.category === 'custom' && editor.form.quota_provider_type === 'newapi'">
                                <label class="cli-field">
                                    <span>访问令牌 <span class="req">*</span></span>
                                    <input
                                        v-model="editor.form.quota_access_token"
                                        class="mono"
                                        type="password"
                                        autocomplete="off"
                                        placeholder="New API System Access Token"
                                    />
                                    <span class="field-hint">New API 后台的 System Access Token</span>
                                </label>
                                <label class="cli-field">
                                    <span>用户 ID <span class="req">*</span></span>
                                    <input
                                        v-model="editor.form.quota_user_id"
                                        class="mono"
                                        placeholder="New API 用户 ID"
                                    />
                                    <span class="field-hint">New API 后台的用户 ID</span>
                                </label>
                            </template>
                        </div>
                    </div>

                    <div class="cli-modal-footer">
                        <button class="cli-modal-btn secondary" @click="closeEditor">取消</button>
                        <button
                            v-if="!editor.isEdit && editor.step === 'form'"
                            class="cli-modal-btn primary"
                            :disabled="!canSave"
                            @click="onSave"
                        >添加</button>
                        <button
                            v-else-if="editor.isEdit"
                            class="cli-modal-btn primary"
                            :disabled="!canSave"
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
import { useCliConfig, type CliProvider, type AppType, type FetchedModel } from '../../composables/useCliConfig';
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
    refreshProxyStatus,
    fetchModels,
    balanceCache,
    fetchBalance,
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
    return status.value?.claude.matched_provider_id || claudeActiveProvider.value ? 'on' : 'warn';
});
const claudeBadgeText = computed(() => {
    const s = status.value?.claude;
    if (!s?.installed) return '未安装';
    return s.matched_provider_id || claudeActiveProvider.value ? '已匹配' : '未匹配';
});
const codexBadgeClass = computed(() => {
    if (!status.value?.codex.installed) return 'off';
    return status.value?.codex.matched_provider_id || codexActiveProvider.value ? 'on' : 'warn';
});
const codexBadgeText = computed(() => {
    const s = status.value?.codex;
    if (!s?.installed) return '未安装';
    return s.matched_provider_id || codexActiveProvider.value ? '已匹配' : '未匹配';
});

// ---------- 代理回显 ----------
const proxyUrl = computed(() => {
    const p = proxyStatus.value;
    if (!p?.running || !p.port) return '';
    return `http://${p.address}:${p.port}`;
});

/** Claude 是否处于代理接管模式 */
const claudeProxyOn = computed(
    () => !!proxyStatus.value?.running && !!proxyStatus.value?.claude_taken_over,
);
/** Codex 是否处于代理接管模式 */
const codexProxyOn = computed(
    () => !!proxyStatus.value?.running && !!proxyStatus.value?.codex_taken_over,
);

/** Claude 当前 Provider 名称 (接管时用代理上游名, 否则用 live 匹配名) */
const claudeActiveProvider = computed(() =>
    claudeProxyOn.value
        ? (proxyStatus.value?.claude_provider_name || '')
        : (status.value?.claude.matched_provider_name || ''),
);

/** Codex 当前 Provider 名称 (接管时用代理上游名, 否则用 live 匹配名) */
const codexActiveProvider = computed(() =>
    codexProxyOn.value
        ? (proxyStatus.value?.codex_provider_name || '')
        : (status.value?.codex.matched_provider_name || ''),
);

/** Claude 展示的 URL (接管时显示代理地址, 否则显示 live base_url) */
const claudeDisplayUrl = computed(() =>
    claudeProxyOn.value ? proxyUrl.value : (status.value?.claude.base_url || ''),
);

/** Codex 展示的 URL */
const codexDisplayUrl = computed(() =>
    codexProxyOn.value ? proxyUrl.value : (status.value?.codex.base_url || ''),
);

/** Codex 当前模型 — 接管时从 provider 数据取, 否则从 live config 取 */
const codexModels = computed(() => {
    if (codexProxyOn.value) {
        const pid = proxyStatus.value?.codex_provider_id;
        const p = providers.value.find((item) => item.id === pid);
        return p?.model || '';
    }
    return status.value?.codex.model || '';
});

/**
 * Claude 当前模型 — 接管时从 provider 数据取 (live config 没变), 否则从 live config 取。
 * provider 的 sonnet_1m 等布尔值还原成 [1M] 后缀, 与 live config 格式一致。
 */
const claudeModels = computed(() => {
    if (claudeProxyOn.value) {
        const pid = proxyStatus.value?.claude_provider_id;
        const p = providers.value.find((item) => item.id === pid);
        if (p) {
            return {
                model_sonnet: p.sonnet_1m && p.model_sonnet ? `${p.model_sonnet}[1M]` : p.model_sonnet,
                model_opus: p.opus_1m && p.model_opus ? `${p.model_opus}[1M]` : p.model_opus,
                model_haiku: p.model_haiku,
                model: p.model,
            };
        }
        return { model_sonnet: '', model_opus: '', model_haiku: '', model: '' };
    }
    return {
        model_sonnet: status.value?.claude.model_sonnet || '',
        model_opus: status.value?.claude.model_opus || '',
        model_haiku: status.value?.claude.model_haiku || '',
        model: status.value?.claude.model || '',
    };
});

// ---------- Provider Tab (Claude / Codex) ----------
const providerTab = ref<'claude' | 'codex'>('claude');
const providerSearch = ref('');

/** 按 Tab + 搜索关键词过滤 provider */
const filteredProviders = computed(() => {
    const byTab = providers.value.filter((p) =>
        providerTab.value === 'claude'
            ? p.scope === 'claude' || p.scope === 'both'
            : p.scope === 'codex' || p.scope === 'both',
    );
    const kw = providerSearch.value.trim().toLowerCase();
    if (!kw) return byTab;
    return byTab.filter((p) =>
        [p.name, p.note, p.base_url, p.website_url, p.model]
            .some((f) => f?.toLowerCase().includes(kw)),
    );
});

/** 该 provider 是否为当前 Tab 对应 CLI 的活动 provider */
const isCurrentForTab = (p: CliProvider) => {
    if (providerTab.value === 'claude') {
        return status.value?.claude.matched_provider_id === p.id;
    }
    return status.value?.codex.matched_provider_id === p.id;
};

// ---------- 编辑器 (两步式: 预设首屏 -> 表单) ----------
const emptyForm = (tab: 'claude' | 'codex' = 'claude'): CliProvider => ({
    id: '', name: '', scope: 'claude', kind: 'relay', category: 'custom',
    base_url: '', api_key: '', model: '',
    model_sonnet: '', model_haiku: '', model_opus: '',
    sonnet_name: '', opus_name: '', haiku_name: '',
    sonnet_1m: false, opus_1m: false, haiku_1m: false,
    note: '', website_url: '',
    auth_field: 'ANTHROPIC_AUTH_TOKEN',
    api_format: tab === 'codex' ? 'openai_chat' : 'anthropic',
    custom_user_agent: '', models_url: '', preset_id: '',
    icon: '', icon_color: '', codex_auth_json: '', codex_config_toml: '', claude_settings_json: '',
    quota_provider_type: '', quota_access_token: '', quota_user_id: '',
    updated_at: 0,
});

const editor = reactive({
    visible: false,
    isEdit: false,
    /** 'preset' = 预设首屏, 'form' = 表单 */
    step: 'preset' as 'preset' | 'form',
    /** 当前应用 Tab */
    appTab: 'claude' as 'claude' | 'codex',
    form: emptyForm('claude'),
});

// 预设选择器状态
const presetSearch = ref('');
const advancedOpen = ref(false);
const apiKeyVisible = ref(false);

// ---------- 余额查询 ----------
const balanceLoading = reactive<Record<string, boolean>>({});
const lastPresetName = ref('');

// ---------- 表单校验 ----------
/** P1-1: Codex tab 追加 model 必填校验 */
const canSave = computed(() => {
    if (!editor.form.name.trim()) return false;
    if (editor.appTab === 'codex' && !editor.form.model.trim()) return false;
    return true;
});

/** API 格式选项 — 按 Tab 动态标签 (原生 vs 需代理转换) */
const apiFormatOptions = computed(() => {
    if (editor.appTab === 'codex') {
        return [
            { value: 'openai_responses', label: 'OpenAI Responses（原生）' },
            { value: 'openai_chat', label: 'OpenAI Chat Completions（原生）' },
            { value: 'anthropic', label: 'Anthropic Messages（需代理转换）' },
            { value: 'gemini_native', label: 'Gemini Native（需代理转换）' },
        ];
    }
    return [
        { value: 'anthropic', label: 'Anthropic Messages（原生）' },
        { value: 'openai_chat', label: 'OpenAI Chat Completions（需代理转换）' },
        { value: 'openai_responses', label: 'OpenAI Responses（需代理转换）' },
        { value: 'gemini_native', label: 'Gemini Native（需代理转换）' },
    ];
});

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

/** 从 auth.json 文本中提取 OPENAI_API_KEY (Codex tab 回退) */
function parseKeyFromAuthJson(authJson: string): string {
    try {
        const parsed = JSON.parse(authJson);
        return parsed.OPENAI_API_KEY || parsed.openai_api_key || '';
    } catch {
        return '';
    }
}

/** 从当前 base_url + api_key 拉取可用模型列表 */
async function onFetchModels() {
    const baseUrl = editor.form.base_url.trim();
    // P1-2: Codex tab 时若 api_key 为空, 从 auth.json 回退解析
    let apiKey = editor.form.api_key.trim();
    if (!apiKey && editor.appTab === 'codex' && editor.form.codex_auth_json.trim()) {
        apiKey = parseKeyFromAuthJson(editor.form.codex_auth_json).trim();
    }
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

/** Codex 双编辑器占位文本 — 根据当前表单字段实时生成预览 */

/** Claude settings.json 实时预览 */
const claudeSettingsPreview = computed(() => {
    const f = editor.form;
    const env: Record<string, string> = {};
    if (f.base_url) env.ANTHROPIC_BASE_URL = f.base_url;
    if (f.api_key) {
        const keyName = f.auth_field === 'ANTHROPIC_API_KEY' ? 'ANTHROPIC_API_KEY' : 'ANTHROPIC_AUTH_TOKEN';
        env[keyName] = f.api_key;
    }
    const sonnet = f.sonnet_1m && f.model_sonnet ? `${f.model_sonnet}[1M]` : f.model_sonnet;
    const opus = f.opus_1m && f.model_opus ? `${f.model_opus}[1M]` : f.model_opus;
    if (sonnet) env.ANTHROPIC_DEFAULT_SONNET_MODEL = sonnet;
    if (opus) env.ANTHROPIC_DEFAULT_OPUS_MODEL = opus;
    if (f.model_haiku) env.ANTHROPIC_DEFAULT_HAIKU_MODEL = f.model_haiku;
    if (f.sonnet_name && sonnet) env.ANTHROPIC_DEFAULT_SONNET_MODEL_NAME = f.sonnet_name;
    if (f.opus_name && opus) env.ANTHROPIC_DEFAULT_OPUS_MODEL_NAME = f.opus_name;
    if (f.haiku_name && f.model_haiku) env.ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME = f.haiku_name;
    const primary = f.model || sonnet || opus || f.model_haiku || '';
    if (primary) {
        env.ANTHROPIC_MODEL = primary.replace(/\[1M\]$/i, '');
    }
    return JSON.stringify({ env }, null, 2);
});

/** Codex auth.json 实时预览 */
const codexAuthPreview = computed(() => {
    const f = editor.form;
    const obj: Record<string, string> = {};
    if (f.api_key) obj.OPENAI_API_KEY = f.api_key;
    if (f.base_url) obj.OPENAI_BASE_URL = f.base_url;
    return JSON.stringify(
        Object.keys(obj).length ? obj : { OPENAI_API_KEY: 'sk-your-api-key-here' },
        null,
        2,
    );
});

/** Codex config.toml 实时预览 */
const codexConfigPreview = computed(() => {
    const f = editor.form;
    const model = f.model || f.model_sonnet || f.model_opus || f.model_haiku || 'gpt-5';
    const wireApi = f.api_format === 'openai_responses' ? 'responses' : 'chat';
    const providerName = f.name || 'custom';
    let text = `model_provider = "custom"\nmodel = "${model}"\n`;
    if (wireApi === 'responses') {
        text += 'model_reasoning_effort = "high"\ndisable_response_storage = true\n\n';
    } else {
        text += '\n';
    }
    text += '[model_providers.custom]\n';
    text += `name = "${providerName}"\n`;
    text += `base_url = "${f.base_url || 'https://api.example.com'}"\n`;
    text += `wire_api = "${wireApi}"\n`;
    text += 'requires_openai_auth = true\n';
    return text;
});

/** 当前 Tab 对应的预设列表 */
const availablePresets = computed(() => {
    if (editor.isEdit) return [];
    return editor.appTab === 'claude' ? CLAUDE_PRESETS : CODEX_PRESETS;
});

/** 搜索过滤后的预设 (增强: 匹配 name/id/baseUrl, 官方置顶) */
const filteredPresets = computed(() => {
    const kw = presetSearch.value.trim().toLowerCase();
    const list = availablePresets.value;
    const matched = kw
        ? list.filter((p) =>
              [p.name, p.id, p.baseUrl, p.websiteUrl]
                  .some((f) => f?.toLowerCase().includes(kw)),
          )
        : list;
    // 官方分类置顶
    return [...matched].sort((a, b) => {
        const aOff = a.category === 'official' ? 0 : 1;
        const bOff = b.category === 'official' ? 0 : 1;
        return aOff - bOff;
    });
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

// ---------- 余额查询 ----------
async function onQueryBalance(p: CliProvider) {
    balanceLoading[p.id] = true;
    try {
        const result = await fetchBalance(p);
        if (!result.success && result.error) {
            showToast(result.error, 'error');
        } else if (result.items.some((i) => !i.isValid)) {
            showToast('API Key 可能失效', 'error');
        }
    } finally {
        balanceLoading[p.id] = false;
    }
}

function formatBalanceTime(iso: string): string {
    try {
        const d = new Date(iso);
        return `${d.getMonth() + 1}/${d.getDate()} ${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}`;
    } catch {
        return iso;
    }
}

const onSwitch = async (app: AppType, id: string) => {
    const p = providers.value.find((item) => item.id === id);
    if (!p) return;
    // 前置校验: 非官方 + 非原生格式 + 未开代理 → 提示需先启动代理
    const isProxyOn = !!proxyStatus.value?.running;
    if (
        p.category !== 'official' &&
        p.api_format !== 'anthropic' &&
        !isProxyOn &&
        app === 'claude'
    ) {
        showToast(`该供应商使用 ${p.api_format} 格式,Claude Code 需先启动代理才能正常工作`, 'error');
        return;
    }
    try {
        const result = await switchProvider(app, id);
        if (result.success) {
            // 差异化提示
            const suffix = app === 'codex'
                ? ',请重启 Codex CLI 生效'
                : app === 'claude' && !claudeProxyOn.value
                    ? ''
                    : '';
            showToast(result.message + suffix, 'success');
        } else {
            showToast(result.message, 'error');
        }
    } catch (e) {
        showToast('切换失败: ' + e, 'error');
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
    refreshProxyStatus();
});

onUnmounted(() => {
    if (toastTimer) window.clearTimeout(toastTimer);
});
</script>

<style scoped>
.cli-panel {
    display: flex;
    flex-direction: column;
    gap: 18px;
}

/* ===== 通用 ===== */
.mono {
    font-family: var(--xuya-font-mono);
}

/* ===== 状态卡 ===== */
.cli-status-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
}

.cli-status-card {
    padding: 16px;
    border-radius: var(--xuya-radius-lg);
    background: var(--xuya-card-bg);
    border: 1px solid var(--xuya-border);
    box-shadow: var(--xuya-shadow-sm);
    display: flex;
    flex-direction: column;
    gap: 10px;
    transition: border-color var(--xuya-duration) var(--xuya-ease),
        box-shadow var(--xuya-duration) var(--xuya-ease);
}

.cli-status-card.active {
    border-color: var(--xuya-success);
    box-shadow: 0 0 0 1px var(--xuya-success-soft), var(--xuya-shadow);
}

.cli-status-card.proxy-on {
    border-color: var(--xuya-accent);
    box-shadow: 0 0 0 1px var(--xuya-accent-soft), var(--xuya-shadow);
}

.cli-status-badges {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
}

.cli-badge.proxy {
    background: var(--xuya-accent-gradient);
    color: #fff;
    font-weight: 600;
}

.cli-status-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.cli-app-name-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
}

.cli-app-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    line-height: 0;
}

.cli-app-icon :deep(svg) {
    width: 20px;
    height: 20px;
}

.cli-app-name {
    font-weight: 600;
    font-size: 14px;
    color: var(--xuya-text);
}

.cli-badge {
    font-size: 10.5px;
    font-weight: 600;
    padding: 3px 9px;
    border-radius: 999px;
    letter-spacing: 0.2px;
}

.cli-badge.on {
    background: var(--xuya-success-soft);
    color: var(--xuya-success);
}

.cli-badge.warn {
    background: var(--xuya-warn-soft);
    color: var(--xuya-warn);
}

.cli-badge.off {
    background: var(--xuya-border);
    color: var(--xuya-text-tertiary);
}

.cli-status-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    font-size: 12px;
    gap: 12px;
    line-height: 1.5;
}

.cli-status-label {
    color: var(--xuya-text-tertiary);
    flex-shrink: 0;
    font-size: 11.5px;
}

.cli-status-val {
    text-align: right;
    word-break: break-all;
    color: var(--xuya-text);
}

.cli-status-val.small {
    font-size: 10.5px;
    color: var(--xuya-text-tertiary);
}

.cli-status-val.model-pills {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
}

.live-model-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 10.5px;
    font-weight: 500;
    padding: 2px 8px;
    border-radius: var(--xuya-radius-sm);
    background: var(--xuya-input-bg);
    color: var(--xuya-text-secondary);
}

.onem-badge {
    display: inline-block;
    padding: 0 4px;
    border-radius: 3px;
    background: var(--xuya-warn-soft);
    color: var(--xuya-warn);
    font-weight: 700;
    font-size: 9px;
}

/* ===== 配置文件预览 ===== */
.config-viewer {
    border-radius: var(--xuya-radius-lg);
    background: var(--xuya-card-bg);
    border: 1px solid var(--xuya-border);
    box-shadow: var(--xuya-shadow-sm);
    overflow: hidden;
}

.config-viewer-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    gap: 8px;
    border-bottom: 1px solid var(--xuya-border-light);
}

.config-viewer-tabs {
    display: inline-flex;
    background: var(--xuya-input-bg);
    border-radius: var(--xuya-radius);
    padding: 3px;
    gap: 2px;
}

.config-tab {
    padding: 6px 16px;
    font-size: 12px;
    font-weight: 500;
    border: none;
    background: transparent;
    color: var(--xuya-text-secondary);
    border-radius: var(--xuya-radius-sm);
    cursor: pointer;
    transition: all var(--xuya-duration-fast) var(--xuya-ease);
}

.config-tab:hover:not(.active) {
    color: var(--xuya-text);
}

.config-tab.active {
    background: var(--xuya-bg-elevated);
    color: var(--xuya-accent);
    font-weight: 600;
    box-shadow: var(--xuya-shadow-sm);
}

.config-viewer-actions {
    display: flex;
    gap: 6px;
}

.config-viewer-body {
    padding: 12px;
}

.config-viewer-path {
    font-size: 10.5px;
    color: var(--xuya-text-tertiary);
    padding: 0 4px 8px;
    word-break: break-all;
}

.config-viewer-code {
    margin: 0;
    padding: 14px;
    max-height: 280px;
    overflow: auto;
    background: var(--xuya-input-bg);
    border: 1px solid var(--xuya-border-light);
    border-radius: var(--xuya-radius);
    font-family: var(--xuya-font-mono);
    font-size: 11.5px;
    line-height: 1.6;
    color: var(--xuya-text);
    white-space: pre-wrap;
    word-break: break-all;
}

/* ===== Mini 按钮 (卡片操作 + 预览) ===== */
.cli-mini-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 5px 11px;
    font-size: 11px;
    font-weight: 500;
    border-radius: var(--xuya-radius-sm);
    cursor: pointer;
    background: var(--xuya-accent-soft);
    color: var(--xuya-accent);
    border: 1px solid transparent;
    transition: all var(--xuya-duration-fast) var(--xuya-ease);
    white-space: nowrap;
}

.cli-mini-btn:hover:not(:disabled):not(.current) {
    background: var(--xuya-accent-soft-strong);
}

.cli-mini-btn.current {
    background: var(--xuya-success);
    color: #fff;
    cursor: default;
}

.cli-mini-btn.ghost {
    background: transparent;
    color: var(--xuya-text-secondary);
    border-color: var(--xuya-border);
}

.cli-mini-btn.ghost:hover:not(:disabled) {
    background: var(--xuya-input-bg);
    color: var(--xuya-text);
    border-color: var(--xuya-border-strong);
}

.cli-mini-btn.danger {
    background: transparent;
    color: var(--xuya-danger);
    border-color: var(--xuya-border);
}

.cli-mini-btn.danger:hover {
    background: var(--xuya-danger-soft);
    border-color: var(--xuya-danger);
}

.cli-mini-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

/* ===== Provider 列表 ===== */
.cli-providers-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
}

.provider-tabs {
    display: inline-flex;
    gap: 2px;
    padding: 3px;
    background: var(--xuya-input-bg);
    border-radius: var(--xuya-radius);
    border: 1px solid var(--xuya-border-light);
}

.provider-tab {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 7px 16px;
    font-size: 13px;
    font-weight: 500;
    border-radius: var(--xuya-radius-sm);
    cursor: pointer;
    background: transparent;
    color: var(--xuya-text-secondary);
    border: none;
    transition: all var(--xuya-duration-fast) var(--xuya-ease);
}

.provider-tab:not(.active):hover {
    color: var(--xuya-text);
}

.provider-tab.active {
    background: var(--xuya-bg-elevated);
    color: var(--xuya-accent);
    font-weight: 600;
    box-shadow: var(--xuya-shadow-sm);
}

.scope-badge {
    display: inline-block;
    margin-left: 4px;
    padding: 0 5px;
    border-radius: 3px;
    background: var(--xuya-accent-soft);
    color: var(--xuya-accent);
    font-size: 9px;
    font-weight: 600;
}

.provider-search-wrap {
    flex: 1;
    max-width: 200px;
}
.provider-search {
    width: 100%;
    padding: 5px 10px;
    font-size: 12px;
    border-radius: var(--xuya-radius);
    border: 1px solid var(--xuya-border);
    background: var(--xuya-input-bg);
    color: var(--xuya-text);
    transition: border-color var(--xuya-duration-fast) var(--xuya-ease),
        box-shadow var(--xuya-duration-fast) var(--xuya-ease);
}
.provider-search:focus {
    outline: none;
    border-color: var(--xuya-accent);
    box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.provider-search::placeholder {
    color: var(--xuya-text-tertiary);
}

.cli-add-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 6px 14px;
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--xuya-radius);
    cursor: pointer;
    background: var(--xuya-accent-gradient);
    color: #fff;
    border: none;
    transition: transform var(--xuya-duration-fast) var(--xuya-ease),
        box-shadow var(--xuya-duration-fast) var(--xuya-ease);
    box-shadow: 0 2px 8px var(--xuya-accent-glow);
}

.cli-add-btn:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 14px var(--xuya-accent-glow);
}

.cli-add-btn:active {
    transform: translateY(0);
}

.cli-empty {
    padding: 32px 20px;
    text-align: center;
    font-size: 12.5px;
    color: var(--xuya-text-tertiary);
    background: var(--xuya-card-bg);
    border: 1px dashed var(--xuya-border);
    border-radius: var(--xuya-radius-lg);
}

/* Provider 卡片网格 */
.provider-card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 12px;
}

.provider-card {
    position: relative;
    padding: 16px;
    border-radius: var(--xuya-radius-lg);
    background: var(--xuya-card-bg);
    border: 1px solid var(--xuya-border);
    box-shadow: var(--xuya-shadow-sm);
    display: flex;
    flex-direction: column;
    gap: 12px;
    transition: transform var(--xuya-duration) var(--xuya-ease),
        box-shadow var(--xuya-duration) var(--xuya-ease),
        border-color var(--xuya-duration) var(--xuya-ease);
    overflow: hidden;
}

.provider-card::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--xuya-accent-gradient);
    opacity: 0.4;
    transition: opacity var(--xuya-duration) var(--xuya-ease);
}

.provider-card:hover {
    transform: translateY(-2px);
    box-shadow: var(--xuya-shadow-hover);
    border-color: var(--xuya-accent);
}

.provider-card:hover::before {
    opacity: 1;
}

.provider-card.active {
    border-color: var(--xuya-success);
    box-shadow: 0 0 0 1px var(--xuya-success-soft), var(--xuya-shadow);
}

.provider-card.active::before {
    background: linear-gradient(180deg, var(--xuya-success), color-mix(in srgb, var(--xuya-success) 60%, #000));
    opacity: 1;
}

.provider-current-badge {
    position: absolute;
    top: 0;
    right: 0;
    background: var(--xuya-success);
    color: #fff;
    font-size: 10px;
    font-weight: 600;
    padding: 3px 12px;
    border-radius: 0 var(--xuya-radius-lg) 0 var(--xuya-radius-sm);
    z-index: 1;
}

.provider-card-head {
    display: flex;
    align-items: center;
    gap: 12px;
}

/* 图标圆标 (provider 卡片 + 选中条复用) */
.provider-icon {
    width: 38px;
    height: 38px;
    border-radius: var(--xuya-radius);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 15px;
    font-weight: 700;
    flex-shrink: 0;
    overflow: hidden;
    background: var(--xuya-input-bg);
}

.provider-icon.sm {
    width: 28px;
    height: 28px;
    border-radius: var(--xuya-radius-sm);
    font-size: 12px;
}

.provider-icon-svg {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 70%;
    height: 70%;
    line-height: 0;
    pointer-events: none;
    user-select: none;
}

.provider-icon-svg :deep(svg) {
    width: 100%;
    height: 100%;
}

.provider-card-info {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
    flex: 1;
}

.provider-card-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--xuya-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.provider-card-cat {
    font-size: 10.5px;
    color: var(--xuya-text-tertiary);
    font-weight: 500;
}

/* 分类着色 (card + preset + selected bar 共享) */
.provider-card-cat.official,
.provider-card-cat.cn_official,
.preset-tile-cat.official,
.preset-tile-cat.cn_official,
.selected-preset-cat.official,
.selected-preset-cat.cn_official {
    color: var(--xuya-success);
    background: var(--xuya-success-soft);
    padding: 1px 6px;
    border-radius: var(--xuya-radius-sm);
    width: fit-content;
}

.provider-card-cat.third_party,
.provider-card-cat.aggregator,
.provider-card-cat.cloud_provider,
.preset-tile-cat.third_party,
.preset-tile-cat.aggregator,
.preset-tile-cat.cloud_provider,
.selected-preset-cat.third_party,
.selected-preset-cat.aggregator,
.selected-preset-cat.cloud_provider {
    color: var(--xuya-accent);
    background: var(--xuya-accent-soft);
    padding: 1px 6px;
    border-radius: var(--xuya-radius-sm);
    width: fit-content;
}

.provider-card-url {
    font-size: 11px;
    color: var(--xuya-text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* 模型摘要 */
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
    padding: 2px 8px;
    border-radius: var(--xuya-radius-sm);
    background: var(--xuya-input-bg);
    color: var(--xuya-text-secondary);
    font-weight: 500;
    line-height: 1.5;
}

.model-tag.sonnet {
    background: rgba(217, 119, 87, 0.13);
    color: #d97757;
}

.model-tag.opus {
    background: rgba(139, 92, 246, 0.13);
    color: #8b5cf6;
}

.model-tag.haiku {
    background: var(--xuya-success-soft);
    color: var(--xuya-success);
}

.model-tag.default {
    background: var(--xuya-input-bg);
    color: var(--xuya-text-tertiary);
    font-style: italic;
}

.model-1m-mark {
    font-weight: 700;
    color: var(--xuya-warn);
}

/* 卡片操作区: 主操作占满, 编辑/删除靠右 */
/* 余额展示区 */
.balance-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 12px;
    background: var(--xuya-bg-subtle, var(--xuya-input-bg));
    border-radius: var(--xuya-radius);
    border: 1px solid var(--xuya-border-light);
}

/* 余额制: 金额 */
.balance-amount-row {
    display: flex;
    flex-direction: column;
    gap: 5px;
}
.ba-header {
    display: flex;
    align-items: baseline;
    gap: 8px;
}
.ba-remaining {
    font-size: 18px;
    font-weight: 700;
    color: var(--xuya-success);
    font-family: var(--xuya-font-mono);
    letter-spacing: -0.3px;
}
.ba-remaining.low {
    color: var(--xuya-danger);
}
.ba-unit {
    font-size: 11px;
    font-weight: 500;
    opacity: 0.7;
    margin-left: 2px;
}
.ba-detail {
    font-size: 11px;
    color: var(--xuya-text-tertiary);
    font-family: var(--xuya-font-mono);
}
.ba-bar {
    height: 4px;
    background: var(--xuya-border);
    border-radius: 2px;
    overflow: hidden;
}
.ba-bar-fill {
    height: 100%;
    background: var(--xuya-success);
    border-radius: 2px;
    transition: width var(--xuya-duration) var(--xuya-ease);
}
.ba-bar-fill.warn {
    background: var(--xuya-danger);
}

/* 套餐制: 多窗口 */
.plan-tiers {
    display: flex;
    flex-direction: column;
    gap: 7px;
}
.plan-tier {
    display: flex;
    flex-direction: column;
    gap: 3px;
}
.pt-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
}
.pt-label {
    font-size: 11px;
    color: var(--xuya-text-secondary);
    font-weight: 500;
}
.pt-pct {
    font-size: 12px;
    font-weight: 700;
    color: var(--xuya-accent);
    font-family: var(--xuya-font-mono);
}
.pt-pct.high {
    color: var(--xuya-danger);
}
.pt-bar {
    height: 4px;
    background: var(--xuya-border);
    border-radius: 2px;
    overflow: hidden;
}
.pt-bar-fill {
    height: 100%;
    background: var(--xuya-accent);
    border-radius: 2px;
    transition: width var(--xuya-duration) var(--xuya-ease);
}
.pt-bar-fill.warn {
    background: var(--xuya-danger);
}
.pt-reset {
    font-size: 9.5px;
    color: var(--xuya-text-tertiary);
    margin-top: 1px;
}

/* 失效 / 错误 */
.ba-invalid {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11.5px;
    color: var(--xuya-danger);
    font-weight: 500;
}
.ba-invalid-icon {
    font-size: 13px;
}
.ba-error {
    font-size: 11.5px;
    color: var(--xuya-text-tertiary);
    text-align: center;
    padding: 4px 0;
}

.provider-card-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding-top: 10px;
    margin-top: auto;
    border-top: 1px solid var(--xuya-border-light);
}

.provider-card-actions .cli-mini-btn:not(.ghost):not(.danger) {
    flex: 1;
    justify-content: center;
    min-width: 0;
}

.provider-card-actions .cli-mini-btn.ghost,
.provider-card-actions .cli-mini-btn.danger {
    flex: 0 0 auto;
    padding: 5px 10px;
}

/* ===== 弹窗 ===== */
.cli-modal-overlay {
    position: fixed;
    inset: 0;
    background: var(--xuya-overlay);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}

.cli-modal-card {
    width: 600px;
    max-width: 92vw;
    max-height: 86vh;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--xuya-bg-elevated);
    border: 1px solid var(--xuya-border);
    border-radius: var(--xuya-radius-xl);
    padding: 22px 24px;
    box-shadow: var(--xuya-shadow-xl);
    scrollbar-width: thin;
    scrollbar-color: var(--xuya-border-strong) transparent;
}

.cli-modal-card::-webkit-scrollbar {
    width: 6px;
}

.cli-modal-card::-webkit-scrollbar-track {
    background: transparent;
    margin: 12px 0;
}

.cli-modal-card::-webkit-scrollbar-thumb {
    background: var(--xuya-border-strong);
    border-radius: 3px;
}

.cli-modal-card::-webkit-scrollbar-thumb:hover {
    background: var(--xuya-text-tertiary);
}

/* 弹窗加宽 (预设选择器) */
.cli-modal-large {
    width: 600px;
}

.cli-modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 18px;
}

.cli-modal-header h4 {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    color: var(--xuya-text);
}

.cli-back-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    font-weight: 500;
    color: var(--xuya-accent);
    background: var(--xuya-accent-soft);
    border: 1px solid var(--xuya-accent-soft-strong);
    border-radius: 999px;
    padding: 5px 14px;
    cursor: pointer;
    transition: all var(--xuya-duration-fast) var(--xuya-ease);
    white-space: nowrap;
    flex-shrink: 0;
}

.cli-back-btn:hover {
    background: var(--xuya-accent-soft-strong);
    transform: translateX(-2px);
}

.cli-back-btn:active {
    transform: translateX(0);
}

.cli-modal-body {
    display: flex;
    flex-direction: column;
    gap: 14px;
}

/* ===== 应用 Tab ===== */
.app-tabs {
    display: flex;
    gap: 2px;
    padding: 4px;
    background: var(--xuya-input-bg);
    border-radius: var(--xuya-radius);
    border: 1px solid var(--xuya-border-light);
}

.app-tab {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 8px;
    font-size: 13px;
    font-weight: 500;
    border-radius: var(--xuya-radius-sm);
    cursor: pointer;
    background: transparent;
    color: var(--xuya-text-secondary);
    border: none;
    transition: all var(--xuya-duration-fast) var(--xuya-ease);
    position: relative;
}

.app-tab:not(.active):hover {
    color: var(--xuya-text);
}

.app-tab.active {
    background: var(--xuya-bg-elevated);
    color: var(--xuya-accent);
    font-weight: 600;
    box-shadow: var(--xuya-shadow-sm);
}

.app-tab .tab-icon {
    width: 16px;
    height: 16px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    opacity: 0.6;
    transition: opacity var(--xuya-duration-fast) var(--xuya-ease);
}

.app-tab .tab-icon :deep(svg) {
    width: 16px;
    height: 16px;
}

.app-tab.active .tab-icon {
    opacity: 1;
}

.app-tab.active::after {
    content: '';
    position: absolute;
    bottom: 2px;
    left: 30%;
    right: 30%;
    height: 2px;
    background: var(--xuya-accent-gradient);
    border-radius: 1px;
}

/* ===== 已选预设提示条 ===== */
.selected-preset-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--xuya-radius);
    background: var(--xuya-accent-soft);
    border: 1px solid var(--xuya-accent-soft-strong);
    font-size: 13px;
    color: var(--xuya-text);
}

.selected-preset-cat {
    font-size: 10px;
    font-weight: 500;
}

/* ===== 预设选择 ===== */
.preset-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
}

.preset-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--xuya-text);
}

.preset-search {
    width: 200px;
    padding: 6px 10px;
    font-size: 12px;
    border-radius: var(--xuya-radius-sm);
    border: 1px solid var(--xuya-border);
    background: var(--xuya-input-bg);
    color: var(--xuya-text);
    transition: border-color var(--xuya-duration-fast) var(--xuya-ease),
        box-shadow var(--xuya-duration-fast) var(--xuya-ease);
}

.preset-search:focus {
    outline: none;
    border-color: var(--xuya-accent);
    box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

.preset-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
    gap: 8px;
    max-height: 340px;
    overflow-y: auto;
    padding: 2px;
}

.preset-tile {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--xuya-radius);
    border: 1px solid var(--xuya-border);
    background: var(--xuya-card-bg);
    cursor: pointer;
    transition: transform var(--xuya-duration-fast) var(--xuya-ease),
        border-color var(--xuya-duration-fast) var(--xuya-ease),
        background var(--xuya-duration-fast) var(--xuya-ease);
}

.preset-tile:hover {
    border-color: var(--xuya-accent);
    background: var(--xuya-accent-soft);
    transform: translateY(-1px);
}

.preset-tile:active {
    transform: translateY(0);
}

.preset-tile-icon {
    width: 32px;
    height: 32px;
    border-radius: var(--xuya-radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 13px;
    font-weight: 700;
    flex-shrink: 0;
    overflow: hidden;
    background: var(--xuya-input-bg);
}

.preset-tile-text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
    flex: 1;
}

.preset-tile-name {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--xuya-text);
    line-height: 1.3;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.preset-tile-cat {
    font-size: 10px;
    font-weight: 500;
    color: var(--xuya-text-tertiary);
    width: fit-content;
}

/* ===== 表单 ===== */
.form-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border-radius: var(--xuya-radius);
    background: var(--xuya-card-bg);
    border: 1px solid var(--xuya-border);
}

.form-section-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 13px;
    font-weight: 600;
    color: var(--xuya-text);
    padding-left: 12px;
    position: relative;
}

.form-section-title::before {
    content: '';
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 14px;
    background: var(--xuya-accent-gradient);
    border-radius: 2px;
}

.form-grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
}

.req {
    color: var(--xuya-danger);
    font-weight: 600;
}

.field-hint {
    font-size: 10.5px;
    color: var(--xuya-text-tertiary);
    line-height: 1.5;
}

.field-hint code {
    padding: 1px 5px;
    border-radius: 3px;
    background: var(--xuya-input-bg);
    font-family: var(--xuya-font-mono);
    font-size: 10px;
}

.cli-field {
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-size: 12px;
}

.cli-field > span {
    color: var(--xuya-text-secondary);
    font-weight: 500;
    display: inline-flex;
    align-items: center;
    gap: 5px;
}

.cli-field input,
.cli-field select {
    padding: 8px 11px;
    border-radius: var(--xuya-radius-sm);
    border: 1px solid var(--xuya-border);
    background: var(--xuya-input-bg);
    color: var(--xuya-text);
    font-size: 13px;
    transition: border-color var(--xuya-duration-fast) var(--xuya-ease),
        box-shadow var(--xuya-duration-fast) var(--xuya-ease);
}

.cli-field input:focus,
.cli-field select:focus {
    outline: none;
    border-color: var(--xuya-accent);
    box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

.cli-field input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.cli-field select {
    cursor: pointer;
}

.api-key-wrap {
    position: relative;
    display: flex;
    align-items: center;
}
.api-key-wrap input {
    width: 100%;
    padding: 8px 36px 8px 11px;
    border-radius: var(--xuya-radius-sm);
    border: 1px solid var(--xuya-border);
    background: var(--xuya-input-bg);
    color: var(--xuya-text);
    font-size: 13px;
    transition: border-color var(--xuya-duration-fast) var(--xuya-ease),
        box-shadow var(--xuya-duration-fast) var(--xuya-ease);
}
.api-key-wrap input:focus {
    outline: none;
    border-color: var(--xuya-accent);
    box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}
.api-key-wrap input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}
.api-key-wrap input::placeholder {
    color: var(--xuya-text-tertiary);
}
.key-toggle {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    padding: 2px;
    opacity: 0.5;
    transition: opacity var(--xuya-duration-fast) var(--xuya-ease);
}
.key-toggle:hover {
    opacity: 1;
}

.get-key-link {
    font-size: 11px;
    color: var(--xuya-accent);
    text-decoration: none;
    align-self: flex-start;
}

.get-key-link:hover {
    text-decoration: underline;
}

.file-badge {
    display: inline-block;
    padding: 1px 7px;
    border-radius: var(--xuya-radius-sm);
    background: var(--xuya-input-bg);
    color: var(--xuya-text);
    font-family: var(--xuya-font-mono);
    font-size: 11px;
}

/* Codex textarea */
.cli-textarea {
    font-family: var(--xuya-font-mono);
    font-size: 12px;
    padding: 10px 12px;
    border-radius: var(--xuya-radius-sm);
    border: 1px solid var(--xuya-border);
    background: var(--xuya-input-bg);
    color: var(--xuya-text);
    resize: vertical;
    line-height: 1.55;
    transition: border-color var(--xuya-duration-fast) var(--xuya-ease),
        box-shadow var(--xuya-duration-fast) var(--xuya-ease);
}

.cli-textarea:focus {
    outline: none;
    border-color: var(--xuya-accent);
    box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

/* 获取模型按钮 */
.fetch-models-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 11px;
    font-size: 11px;
    font-weight: 500;
    border-radius: var(--xuya-radius-sm);
    cursor: pointer;
    border: 1px solid var(--xuya-accent-soft-strong);
    background: var(--xuya-accent-soft);
    color: var(--xuya-accent);
    transition: all var(--xuya-duration-fast) var(--xuya-ease);
}

.fetch-models-btn:not(:disabled):hover {
    background: var(--xuya-accent-soft-strong);
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
    to {
        transform: rotate(360deg);
    }
}

/* 模型角色行 */
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

.model-role-row input.model-input-main {
    flex: 2;
    min-width: 0;
}

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
    border-radius: var(--xuya-radius-sm);
    border: 1px solid var(--xuya-border);
    background: var(--xuya-input-bg);
    color: var(--xuya-text);
    font-size: 12px;
    transition: border-color var(--xuya-duration-fast) var(--xuya-ease),
        box-shadow var(--xuya-duration-fast) var(--xuya-ease);
}

.model-role-row input:focus {
    outline: none;
    border-color: var(--xuya-accent);
    box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

.model-role-tag {
    width: 56px;
    flex-shrink: 0;
    text-align: center;
    font-size: 11px;
    font-weight: 600;
    padding: 5px 0;
    border-radius: var(--xuya-radius-sm);
}

.model-role-tag.sonnet {
    background: rgba(217, 119, 87, 0.15);
    color: #d97757;
}

.model-role-tag.opus {
    background: rgba(139, 92, 246, 0.15);
    color: #8b5cf6;
}

.model-role-tag.haiku {
    background: var(--xuya-success-soft);
    color: var(--xuya-success);
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

.model-1m-toggle input[type='checkbox'] {
    width: 13px;
    height: 13px;
    accent-color: var(--xuya-warn);
    cursor: pointer;
    margin: 0;
}

.model-1m-label {
    font-size: 10.5px;
    font-weight: 700;
    color: var(--xuya-text-tertiary);
    transition: color var(--xuya-duration-fast) var(--xuya-ease);
}

.model-1m-toggle input:checked + .model-1m-label {
    color: var(--xuya-warn);
}

/* 高级选项折叠 */
.advanced-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 4px 8px;
    font-size: 12px;
    font-weight: 600;
    color: var(--xuya-text-secondary);
    cursor: pointer;
    user-select: none;
    border-top: 1px solid var(--xuya-border-light);
    margin-top: 4px;
    transition: color var(--xuya-duration-fast) var(--xuya-ease);
}

.advanced-toggle:hover {
    color: var(--xuya-text);
}

.chevron {
    display: inline-block;
    font-size: 10px;
    transition: transform var(--xuya-duration) var(--xuya-ease);
    color: var(--xuya-text-tertiary);
}

.chevron.open {
    transform: rotate(90deg);
}

.advanced-toggle-hint {
    font-weight: 400;
    font-size: 10.5px;
    color: var(--xuya-text-tertiary);
    margin-left: auto;
}

.advanced-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

/* 弹窗底部 */
.cli-modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 18px;
    padding-top: 16px;
    border-top: 1px solid var(--xuya-border-light);
}

.cli-modal-btn {
    padding: 8px 20px;
    border-radius: var(--xuya-radius);
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    border: none;
    transition: transform var(--xuya-duration-fast) var(--xuya-ease),
        box-shadow var(--xuya-duration-fast) var(--xuya-ease),
        opacity var(--xuya-duration-fast) var(--xuya-ease);
}

.cli-modal-btn.secondary {
    background: transparent;
    color: var(--xuya-text-secondary);
    border: 1px solid var(--xuya-border);
}

.cli-modal-btn.secondary:hover {
    background: var(--xuya-input-bg);
    color: var(--xuya-text);
}

.cli-modal-btn.primary {
    background: var(--xuya-accent-gradient);
    color: #fff;
    box-shadow: 0 2px 8px var(--xuya-accent-glow);
}

.cli-modal-btn.primary:not(:disabled):hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 14px var(--xuya-accent-glow);
}

.cli-modal-btn.primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* Toast */
.cli-toast {
    position: fixed;
    bottom: 28px;
    left: 50%;
    transform: translateX(-50%);
    padding: 10px 22px;
    border-radius: var(--xuya-radius);
    font-size: 13px;
    font-weight: 500;
    z-index: 1001;
    box-shadow: var(--xuya-shadow-xl);
}

.cli-toast.success {
    background: var(--xuya-success);
    color: #fff;
}

.cli-toast.error {
    background: var(--xuya-danger);
    color: #fff;
}

/* 过渡 */
.fade-enter-active,
.fade-leave-active {
    transition: opacity var(--xuya-duration) var(--xuya-ease);
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
