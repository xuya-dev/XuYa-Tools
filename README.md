# XuYa Tools · 程序员日常开发工具箱

基于 **Vite 8 + Vue 3 + Tauri 2** 构建的跨平台桌面工具箱,精选程序员日常高频工具,数据全部本地处理,不上传服务器。

## ✨ 功能特性

- **双栏 Master-Detail 布局** —— 左侧分类导航 + 右侧工具面板,搜索 + 收藏置顶
- **暗色优先设计** —— 支持浅色 / 深色 / 跟随系统三态切换,内置防闪烁
- **无边框窗口** —— 自定义标题栏、拖拽、最小化 / 最大化 / 关闭
- **系统托盘** —— 关闭窗口转隐藏到托盘,后台常驻
- **单实例** —— 重复启动时唤起已有窗口
- **开机自启** —— 可在设置中开启登录时自动启动
- **工具状态持久化** —— 每个工具的输入自动保存,重启不丢失

### 内置工具 (19 个)

| 分类 | 工具 | 说明 |
|------|------|------|
| 格式转换 | **JSON 格式化** | 美化、压缩、转义、反转义,带错误行列定位与字符统计 |
|  | **正则表达式测试** | 实时匹配高亮、捕获组提取、标志位切换、常用速查 |
|  | **文本差异对比** | 双栏输入,行级 diff 高亮,统计增删改 |
|  | **Markdown 预览** | 实时分栏:左侧编辑,右侧预览,支持代码块与表格 |
|  | **SQL 格式化** | 美化 / 压缩 SQL,关键字大写,支持多方言,智能缩进 |
| 编码 / 生成 | **编码转换** | Base64 / URL / HTML 实体 / Hex / Binary 双向转换 (UTF-8 安全) |
|  | **颜色工具** | HEX / RGB / HSL 互转、色板生成、WCAG 对比度检测 |
|  | **二维码生成** | 文本 / URL 转 QR 码,可调尺寸与纠错级别,下载 PNG |
|  | **占位文本生成** | 生成 Lorem Ipsum / 中文占位文本,段落 / 句子 / 单词 |
| 加密 / 解析 | **JWT 解析** | 解码 Header / Payload / Signature,查看算法与过期时间 |
|  | **UUID / 密码 / 哈希** | UUID v4、安全随机密码、MD5 / SHA-1 / SHA-256 / 384 / 512 哈希 |
| 日期时间 | **时间戳转换** | Unix 时间戳 ↔ 可读时间互转,秒 / 毫秒,实时当前时间 |
|  | **Cron 表达式解析** | 解析 Cron 五段表达式,人类可读描述,未来执行时间预览 |
| 网络参考 | **HTTP 请求** | 发送 HTTP 请求测试接口,支持 Headers / Query / Body,完整展示响应 |
|  | **WebSocket 调试** | 连接 WebSocket 服务,双向收发消息,实时查看消息流 |
|  | **HTTP 状态码速查** | 全量 HTTP 状态码、含义与使用场景,可搜索过滤 |
|  | **IP / 子网计算器** | IPv4 子网划分:CIDR、掩码、网段范围、可用主机数一键计算 |
| 开发配置 | **CLI 配置管理** | 管理 Claude Code / Codex CLI 供应商配置,一键切换、编辑、查看 live 配置 |
|  | **反代 / 统计** | 启动本地代理接管 CLI,实时查看请求数据、Token 用量与费用日志 |

## 🚀 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) ≥ 20
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Tauri 2 [系统依赖](https://tauri.app/start/prerequisites/)

### 开发

```bash
npm install          # 安装前端依赖
npm run tauri dev    # 启动桌面应用 (会同时启动 Vite 开发服务器)
```

> 也可单独 `npm run dev` 在浏览器中预览 (窗口控制按钮会在非 Tauri 环境下安全降级)。

### 构建

```bash
npm run tauri build  # 打包为可执行文件 / 安装包
```

产物位于 `src-tauri/target/release/bundle/`。

### 测试与检查

```bash
npm run lint         # ESLint + Prettier 检查
npm run typecheck    # vue-tsc 类型检查
npm run test         # vitest 单元测试 (前端)
npm run test:rust    # cargo test (后端)
```

## 🛠 技术栈

| 层 | 技术 |
|----|------|
| 框架 | Vue 3.5 (`<script setup lang="ts">`) |
| 构建 | Vite 8 |
| 路由 | Vue Router 4 (hash 模式) |
| 桌面 | Tauri 2 |
| 图标 | @lucide/vue |
| 语言 | TypeScript (strict) + Rust |
| 数据 | rusqlite (bundled SQLite) |

## 📁 项目结构

```
XuYa-Tools/
├── src/
│   ├── components/
│   │   ├── cli/           # CliPanel, ProxyPanel, UsagePanel
│   │   ├── layout/        # AppTitleBar, AppLayout, ToolMasterDetail, ToolShell
│   │   └── ui/            # BaseButton, BaseCard, BaseInput, ToastHost
│   ├── composables/       # useTheme, useWindow, useToast, useClipboard,
│   │                      # useFavorites, useToolState, useHttpTool, useWsTool, useCliConfig
│   ├── config/
│   │   ├── tools.ts       # ⭐ 工具注册表 (单一数据源)
│   │   └── providerPresets.ts
│   ├── router/            # 路由 (懒加载)
│   ├── styles/            # theme.css, base.css
│   ├── utils/             # qrcode 等工具
│   └── views/
│       ├── SettingsView.vue
│       └── tools/         # 19 个工具页面
├── src-tauri/
│   ├── src/
│   │   ├── cli_config/    # Claude / Codex 配置读写、备份、模型拉取
│   │   ├── proxy/         # 本地反代服务器 + 接管 + 协议转换
│   │   ├── usage/         # 请求日志、统计聚合
│   │   ├── db/            # SQLite schema 与句柄
│   │   ├── http_client.rs # HTTP / WebSocket 请求工具后端
│   │   └── lib.rs         # 入口:插件、命令、托盘、单实例
│   └── capabilities/      # Tauri 权限白名单
└── ...
```

## ➕ 添加新工具

工具箱采用**注册表驱动**,添加新工具只需三步:

1. 在 `src/config/tools.ts` 增加一条 `ToolMeta` 记录
2. 在 `src/router/index.ts` 增加一条懒加载路由
3. 在 `src/views/tools/` 实现组件 (套用 `ToolShell` 外壳)

## 🔒 数据与隐私

- 所有工具输入仅保存在本地 `localStorage`,不上传任何服务器
- CLI 配置与请求日志保存在系统应用数据目录 (`appDataDir/XuYaTools/`)
- 反代服务仅绑定 `127.0.0.1`,不对外暴露

## 📄 License

[MIT](./LICENSE)
