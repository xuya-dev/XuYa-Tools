// XuYa Tools - 程序员日常开发工具箱
// 后端:单实例 + 系统托盘 + 关闭转隐藏 + 自启动 + 打开链接
//      + Claude/Codex CLI 配置管理 + 本地反代 + 请求统计

use std::path::PathBuf;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

// ==================== 业务模块 ====================
mod cli_config;
use cli_config::types::{
    ApiFormat, AppType, AuthField, CliProvider, ProviderCategory, ProviderKind, ProviderScope,
};
use cli_config::CliConfigService;

mod proxy;
use proxy::ProxyService;

mod db;
mod usage;

mod http_client;
use http_client::WsManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 第二个实例启动时:唤起主窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        // CLI 配置切换服务
        .manage(CliConfigService::new(cli_data_dir()))
        // 本地代理服务 (内部持有统计数据库,与 CLI 服务解耦)
        .manage(ProxyService::new(cli_data_dir()))
        // WebSocket 连接管理器
        .manage(WsManager::new())
        .invoke_handler(tauri::generate_handler![
            // ---- CLI 配置 ----
            get_cli_status,
            get_cli_live_config,
            open_cli_config_file,
            list_cli_providers,
            save_cli_provider,
            delete_cli_provider,
            switch_cli_provider,
            get_cli_widget_snapshot,
            new_cli_provider_template,
            fetch_cli_models,
            // ---- 本地代理 ----
            start_cli_proxy,
            stop_cli_proxy,
            get_cli_proxy_status,
            set_cli_takeover,
            // ---- 请求统计 ----
            get_usage_summary,
            get_request_logs,
            clear_request_logs,
            get_daily_stats,
            fetch_balance,
            // ---- HTTP / WebSocket 请求工具 ----
            http_client::http_request,
            http_client::ws_connect,
            http_client::ws_send,
            http_client::ws_disconnect,
        ])
        .setup(|app| {
            // 注入 AppHandle 给代理服务, 用于 emit 告警事件
            app.state::<ProxyService>().set_app_handle(app.handle().clone());

            // 应用重启后自动恢复代理 (如果上次退出时代理在运行)
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let proxy = handle.state::<ProxyService>();
                    let cli = handle.state::<CliConfigService>();
                    proxy.inner().restore_on_startup(cli.inner()).await;
                });
            }

            // 构建系统托盘:仅一个"退出"菜单项
            let quit_item = MenuItem::with_id(app, "quit", "退出 XuYa Tools", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_item])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(
                    app.default_window_icon()
                        .expect("应用缺少默认窗口图标,无法创建托盘")
                        .clone(),
                )
                .tooltip("XuYa Tools")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    if event.id() == "quit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键单击:显示并聚焦主窗口
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            #[cfg(debug_assertions)]
            {
                let autostart = app.autolaunch();
                println!("[xuya-tools] autostart enabled = {:?}", autostart.is_enabled().ok());
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // 点击关闭按钮时:不退出,仅隐藏到托盘
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ==================== CLI 配置切换命令 ====================

#[tauri::command]
fn get_cli_status(
    svc: State<'_, CliConfigService>,
) -> Result<cli_config::types::CliStatus, String> {
    Ok(svc.detect_status())
}

/// 读取某 app 的 live 配置文件内容 (用于前端预览)。返回 { path, content }。
#[derive(serde::Serialize)]
struct LiveConfigContent {
    path: String,
    content: String,
}

#[tauri::command]
fn get_cli_live_config(
    svc: State<'_, CliConfigService>,
    app_type: String,
) -> Result<LiveConfigContent, String> {
    let app = AppType::from_str(&app_type).ok_or_else(|| format!("未知 app 类型: {app_type}"))?;
    let (path, content) = svc.read_live_config(app);
    Ok(LiveConfigContent { path, content })
}

/// 在系统默认编辑器中打开配置文件 (通过 opener 插件)
#[tauri::command]
async fn open_cli_config_file(app_type: String) -> Result<(), String> {
    let app = AppType::from_str(&app_type).ok_or_else(|| format!("未知 app 类型: {app_type}"))?;
    let path = match app {
        AppType::Claude => cli_config::claude::claude_settings_path(),
        AppType::Codex => cli_config::codex::codex_config_path(),
    };
    if !path.exists() {
        return Err(format!("配置文件不存在: {}", path.display()));
    }
    let path_str = path.to_string_lossy().to_string();
    tauri_plugin_opener::open_path(&path_str, None::<&str>)
        .map_err(|e| format!("打开失败: {e}"))?;
    Ok(())
}

#[tauri::command]
fn list_cli_providers(svc: State<'_, CliConfigService>) -> Result<Vec<CliProvider>, String> {
    Ok(svc.list_providers())
}

#[tauri::command]
fn save_cli_provider(
    svc: State<'_, CliConfigService>,
    provider: CliProvider,
) -> Result<CliProvider, String> {
    Ok(svc.save_provider(provider))
}

#[tauri::command]
fn delete_cli_provider(svc: State<'_, CliConfigService>, id: String) -> Result<bool, String> {
    Ok(svc.delete_provider(&id))
}

/// 从当前 provider 同步代理上游 (Claude + Codex 分别同步各自的 provider)
async fn sync_proxy_from_current(cli: &CliConfigService, proxy: &ProxyService) {
    for app in [AppType::Claude, AppType::Codex] {
        if let Some(p) = cli.current_provider(app) {
            if !p.base_url.is_empty() {
                let _ = proxy
                    .set_target(
                        app,
                        p.id,
                        p.name,
                        p.base_url,
                        p.api_key,
                        p.api_format.as_str().to_string(),
                    )
                    .await;
            }
        }
    }
}

#[tauri::command]
async fn switch_cli_provider(
    cli: State<'_, CliConfigService>,
    proxy: State<'_, ProxyService>,
    app_type: String,
    provider_id: String,
) -> Result<cli_config::types::SwitchResult, String> {
    let app = AppType::from_str(&app_type).ok_or_else(|| format!("未知 app 类型: {app_type}"))?;

    let provider = cli
        .get(&provider_id)
        .ok_or_else(|| "未找到 provider".to_string())?;
    if !provider.applies_to(app) {
        return Ok(cli_config::types::SwitchResult {
            success: false,
            message: format!("该 provider 不适用于 {}", app.as_str()),
            backup_path: String::new(),
        });
    }

    // 检查代理接管状态: 接管模式下 live config 始终指向代理, 切换只改代理上游
    let proxy_status = proxy.status().await;
    let is_taken_over = proxy_status.running
        && match app {
            AppType::Claude => proxy_status.claude_taken_over,
            AppType::Codex => proxy_status.codex_taken_over,
        };

    if is_taken_over {
        // 接管模式: base_url 保持代理地址不变, 但模型必须更新 (否则 CLI 发旧模型名)
        cli.set_current_only(app, &provider_id);
        match app {
            AppType::Claude => {
                let _ = cli_config::claude::update_live_models(&provider);
            }
            AppType::Codex => {
                let _ = cli_config::codex::update_live_model(&provider);
            }
        }
        // 同步该 app 的代理上游 (per-app target)
        if !provider.base_url.is_empty() {
            let _ = proxy
                .set_target(
                    app,
                    provider.id.clone(),
                    provider.name.clone(),
                    provider.base_url.clone(),
                    provider.api_key.clone(),
                    provider.api_format.as_str().to_string(),
                )
                .await;
        }
        return Ok(cli_config::types::SwitchResult {
            success: true,
            message: format!(
                "已切换 {} 上游为 {} (接管模式)",
                app.as_str(),
                provider.name
            ),
            backup_path: String::new(),
        });
    }

    // 正常模式: 写 live config
    let result = cli.switch(app, &provider_id);
    if result.success {
        // 同步该 app 的代理上游
        if !provider.base_url.is_empty() {
            let _ = proxy
                .set_target(
                    app,
                    provider.id.clone(),
                    provider.name.clone(),
                    provider.base_url.clone(),
                    provider.api_key.clone(),
                    provider.api_format.as_str().to_string(),
                )
                .await;
        }
    }
    Ok(result)
}

/// 仪表盘快照: 一次性返回 CLI 当前状态摘要
#[derive(serde::Serialize)]
struct CliWidgetSnapshot {
    claude_provider: Option<String>,
    claude_base_url: String,
    claude_installed: bool,
    codex_provider: Option<String>,
    codex_base_url: String,
    codex_installed: bool,
    proxy_running: bool,
    today_requests: u64,
    today_errors: u64,
}

#[tauri::command]
async fn get_cli_widget_snapshot(
    cli: State<'_, CliConfigService>,
    proxy: State<'_, ProxyService>,
) -> Result<CliWidgetSnapshot, String> {
    let status = cli.detect_status();
    let proxy_status = proxy.status().await;
    let (today_requests, today_errors, _today_latency) = if let Some(db) = proxy.db() {
        usage::stats::today_summary(db).unwrap_or((0, 0, 0))
    } else {
        (0, 0, 0)
    };
    Ok(CliWidgetSnapshot {
        claude_provider: status.claude.matched_provider_name.clone(),
        claude_base_url: status.claude.base_url.clone(),
        claude_installed: status.claude.installed,
        codex_provider: status.codex.matched_provider_name.clone(),
        codex_base_url: status.codex.base_url.clone(),
        codex_installed: status.codex.installed,
        proxy_running: proxy_status.running,
        today_requests,
        today_errors,
    })
}

/// 前端用于构造新 provider 的辅助命令 (生成空壳 + id)
#[tauri::command]
fn new_cli_provider_template() -> Result<CliProvider, String> {
    Ok(CliProvider {
        id: String::new(),
        name: String::new(),
        scope: ProviderScope::Claude,
        kind: ProviderKind::Relay,
        category: ProviderCategory::Custom,
        base_url: String::new(),
        api_key: String::new(),
        model: String::new(),
        model_sonnet: String::new(),
        model_haiku: String::new(),
        model_opus: String::new(),
        sonnet_name: String::new(),
        opus_name: String::new(),
        haiku_name: String::new(),
        sonnet_1m: false,
        opus_1m: false,
        haiku_1m: false,
        note: String::new(),
        website_url: String::new(),
        auth_field: AuthField::AnthropicAuthToken,
        api_format: ApiFormat::Anthropic,
        custom_user_agent: String::new(),
        models_url: String::new(),
        preset_id: String::new(),
        icon: String::new(),
        icon_color: String::new(),
        codex_auth_json: String::new(),
        codex_config_toml: String::new(),
        claude_settings_json: String::new(),
        updated_at: 0,
    })
}

#[tauri::command]
async fn fetch_cli_models(
    base_url: String,
    api_key: String,
    custom_user_agent: Option<String>,
    models_url: Option<String>,
) -> Result<Vec<cli_config::model_fetch::FetchedModel>, String> {
    cli_config::model_fetch::fetch_models(
        &base_url,
        &api_key,
        custom_user_agent.as_deref(),
        models_url.as_deref(),
    )
    .await
}

// ==================== 本地代理命令 ====================

#[tauri::command]
async fn start_cli_proxy(
    cli: State<'_, CliConfigService>,
    proxy: State<'_, ProxyService>,
) -> Result<proxy::types::ProxyServerInfo, String> {
    let info = proxy.start().await?;
    // 启动后自动把两个 CLI 的当前 provider 设为各自上游
    sync_proxy_from_current(&cli, &proxy).await;
    Ok(info)
}

#[tauri::command]
async fn stop_cli_proxy(svc: State<'_, ProxyService>) -> Result<(), String> {
    svc.stop().await
}

#[tauri::command]
async fn get_cli_proxy_status(
    svc: State<'_, ProxyService>,
) -> Result<proxy::types::ProxyStatus, String> {
    Ok(svc.status().await)
}

#[tauri::command]
async fn set_cli_takeover(
    svc: State<'_, ProxyService>,
    app_type: String,
    enabled: bool,
) -> Result<proxy::types::TakeoverResult, String> {
    let app = AppType::from_str(&app_type).ok_or_else(|| format!("未知 app 类型: {app_type}"))?;
    Ok(svc.set_takeover(app, enabled).await)
}

// ==================== 请求统计命令 ====================

#[tauri::command]
fn get_usage_summary(
    svc: State<'_, ProxyService>,
    start_date: Option<i64>,
    end_date: Option<i64>,
) -> Result<usage::types::UsageSummary, String> {
    let db = svc.db().ok_or("数据库未初始化")?;
    usage::stats::get_summary(db, start_date, end_date)
}

#[tauri::command]
fn get_request_logs(
    svc: State<'_, ProxyService>,
    filters: usage::types::LogFilters,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<usage::types::PaginatedLogs, String> {
    let db = svc.db().ok_or("数据库未初始化")?;
    usage::stats::get_logs(db, filters, page.unwrap_or(1), page_size.unwrap_or(20))
}

#[tauri::command]
fn clear_request_logs(svc: State<'_, ProxyService>) -> Result<(), String> {
    let db = svc.db().ok_or("数据库未初始化")?;
    usage::stats::clear_logs(db)
}

#[tauri::command]
fn get_daily_stats(
    svc: State<'_, ProxyService>,
    start_date: Option<i64>,
    end_date: Option<i64>,
) -> Result<Vec<usage::types::DailyStat>, String> {
    let db = svc.db().ok_or("数据库未初始化")?;
    usage::stats::get_daily_stats(db, start_date, end_date)
}

/// CLI 配置数据目录: 优先用应用数据目录, 回退到 home / exe 同级
fn cli_data_dir() -> PathBuf {
    if let Some(dir) = dirs::data_dir() {
        return dir.join("XuYaTools");
    }
    if let Some(dir) = dirs::home_dir() {
        return dir.join(".xuya-tools");
    }
    PathBuf::from("./data")
}

// ==================== 余额查询命令 ====================

#[tauri::command]
async fn fetch_balance(
    base_url: String,
    api_key: String,
) -> Result<cli_config::balance::BalanceResult, String> {
    Ok(cli_config::balance::get_balance(&base_url, &api_key).await)
}
