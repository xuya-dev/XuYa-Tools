# AGENTS.md

本文件为 AI 协作助手 (opencode / Claude Code / Codex 等) 提供项目协作规范。
人类贡献者也建议遵循同样约定。

## 项目简介

XuYa Tools 是基于 **Vite 8 + Vue 3.5 + Tauri 2** 的跨平台桌面程序员工具箱。
前端 TypeScript (strict),后端 Rust,本地数据存储,不上传任何服务器。

## 开发环境要求

- Node.js ≥ 20
- Rust stable (`rustup default stable`)
- Tauri 2 [系统依赖](https://tauri.app/start/prerequisites/)
  - Windows: WebView2 Runtime (预装于 Win11)
  - macOS: 无额外依赖
  - Linux: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev libgtk-3-dev`

## 常用命令

### 前端

| 命令 | 用途 |
|------|------|
| `npm install` | 安装依赖 |
| `npm run dev` | 仅启动 Vite,浏览器预览 (无 Tauri 运行时,窗口控件会降级) |
| `npm run tauri dev` | 启动完整桌面应用 (前端 + Rust) |
| `npm run build` | 前端生产构建 (vue-tsc + vite build) |
| `npm run typecheck` | 仅类型检查,不产出 |
| `npm run lint` | ESLint 检查 (`--max-warnings=0`,任何 warning 都会失败) |
| `npm run lint:fix` | ESLint 自动修复可修复的问题 |
| `npm run format` | Prettier 格式化 |
| `npm run format:check` | Prettier 格式检查 (CI 用) |
| `npm run test` | vitest 单次运行所有单元测试 |
| `npm run test:watch` | vitest watch 模式 |
| `npm run test:coverage` | 生成覆盖率报告 |

### Rust 后端

| 命令 | 用途 |
|------|------|
| `npm run test:rust` | cargo test |
| `npm run fmt:rust` | cargo fmt 自动格式化 |
| `npm run clippy` | cargo clippy (`-D warnings` 任何告警都失败) |
| `cargo check --manifest-path src-tauri/Cargo.toml` | 仅编译检查 (最快) |

## 提交信息规范

使用双语 emoji 前缀,一行格式:

```
<emoji> <English Type>|<中文类型> <简短中文描述>
```

类型映射:

- `✨ Features|新功能`
- `🐛 Bug Fixes|Bug 修复`
- `♻️ Refactoring|代码重构`
- `⚡ Performance|性能优化`
- `📝 Documentation|文档`
- `✅ Tests|测试`
- `🎨 Style|格式`
- `🔧 Chores|杂务`

示例:`✨ Features|新功能 新增 SQL 格式化工具`

## 代码约定

### 通用

- TypeScript strict 模式,不允许隐式 any
- **不要添加注释**,除非用户明确要求 (业务代码自描述优先)
- 不主动提交,除非用户要求 (`git commit` 由用户触发)

### 前端

- Vue 3 `<script setup lang="ts">` 风格,Composition API
- 路径别名 `@/` 指向 `src/`
- 单一数据源:`src/config/tools.ts` 是工具注册表,新增工具需:
  1. 在 `src/config/tools.ts` 增加一条 `ToolMeta`
  2. 在 `src/router/index.ts` 增加一条懒加载路由
  3. 在 `src/views/tools/` 实现组件,套用 `ToolShell` 外壳
- 状态持久化用 `useToolState(toolId, key, default)` composable
- 通知用 `useToast()`,主题用 `useTheme()`,窗口控制用 `useWindow()`
- UI 基础组件在 `src/components/ui/` (BaseButton / BaseCard / BaseInput / ToastHost)
- CSS 变量全部以 `--xuya-` 前缀,定义在 `src/styles/theme.css`
- **禁止在 computed 中产生副作用** (写 ref)。需要时把 ref 也作为 computed 派生
- 错误处理优先用 `?` 传播,UI 层用 toast 提示

### Rust 后端

- 模块组织:`cli_config/` / `proxy/` / `usage/` / `db/` / `http_client.rs`
- 状态用 `tauri::State<T>` 注入,服务在 `lib.rs::run()` 中 `.manage()`
- 命令用 `#[tauri::command]`,前端通过 `invoke('cmd_name', { ... })` 调用
- **生产代码禁止 `.unwrap()`**,改用 `.expect("描述")` 或 `?` 传播错误
- Mutex 锁中毒用 `.expect("xxx 锁被毒化")` 给出明确诊断
- 文件 I/O 用原子写 (写临时文件 + rename)
- 所有用户可见字符串用中文
- 单元测试紧贴被测模块,`#[cfg(test)] mod tests { ... }`

### 测试

- 前端:`src/**/__tests__/*.test.ts`,vitest + happy-dom
- Rust:`#[cfg(test)] mod tests` 紧贴被测代码
- 覆盖率关注 `src/composables/` 与 `src/utils/` (纯逻辑层)

## 文件结构

```
src/
├── components/
│   ├── cli/           # CliPanel / ProxyPanel / UsagePanel
│   ├── layout/        # AppLayout / AppTitleBar / ToolMasterDetail / ToolShell
│   └── ui/            # BaseButton / BaseCard / BaseInput / ToastHost
├── composables/       # useTheme / useWindow / useToast / useFavorites / useToolState ...
├── config/            # tools.ts (注册表) / providerPresets.ts
├── router/            # index.ts (懒加载路由)
├── styles/            # theme.css / base.css
├── utils/             # qrcode.ts 等纯函数工具
└── views/
    ├── SettingsView.vue
    └── tools/         # 19 个工具页面

src-tauri/src/
├── cli_config/        # Claude / Codex 配置读写、备份、模型拉取
├── proxy/             # 本地反代服务器 + 接管 + 协议转换
├── usage/             # 请求日志、统计聚合
├── db/                # SQLite schema 与句柄
├── http_client.rs     # HTTP / WebSocket 请求工具后端
└── lib.rs             # 入口:插件、命令、托盘、单实例
```

## 数据与隐私

- 工具输入持久化在前端 `localStorage`,键前缀 `xuya_state_`
- 收藏列表键 `xuya_favorites`,主题键 `xuya_theme_mode`
- CLI 配置与请求日志在 `appDataDir/XuYaTools/` (SQLite + JSON)
- 反代服务仅绑定 `127.0.0.1`,不对外暴露
