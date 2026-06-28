# XuYa Tools · 程序员日常开发工具箱

基于 **Vite 8 + Vue 3 + Tauri 2** 构建的跨平台桌面工具箱,精选程序员日常高频工具,数据全部本地处理,不上传服务器。

## ✨ 功能特性

- **卡片网格仪表盘** —— 类 it-tools/devtoys 风格,搜索 + 分类筛选 (快捷键 `Ctrl/Cmd+K`)
- **暗色优先设计** —— 支持浅色 / 深色 / 跟随系统三态切换,防闪烁
- **无边框窗口** —— 自定义标题栏、拖拽、最小化/最大化/关闭
- **系统托盘** —— 关闭窗口转隐藏到托盘,后台常驻
- **单实例** —— 重复启动时唤起已有窗口

### 内置工具

| 工具 | 说明 |
|------|------|
| **JSON 格式化** | 美化、压缩、转义、反转义,带错误行列定位与字符统计 |
| **编码转换** | Base64 / URL / HTML 实体 / Hex / Binary 双向转换 (UTF-8 安全) |
| **JWT 解析** | 解码 Header / Payload / Signature,查看算法与过期时间 |
| **时间戳转换** | Unix 时间戳 ↔ 可读时间互转,秒/毫秒,实时当前时间 |
| **UUID / 密码 / 哈希** | UUID v4、安全随机密码、MD5/SHA-1/SHA-256/384/512 哈希 |

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

## 🛠 技术栈

| 层 | 技术 |
|----|------|
| 框架 | Vue 3.5 (`<script setup lang="ts">`) |
| 构建 | Vite 8 |
| 路由 | Vue Router 4 (hash 模式) |
| 桌面 | Tauri 2 |
| 图标 | @lucide/vue |
| 语言 | TypeScript (strict) + Rust |

## 📁 项目结构

```
XuYa-Tools/
├── src/
│   ├── components/
│   │   ├── home/        # SearchBar, ToolCard
│   │   ├── layout/      # AppTitleBar, AppLayout, ToolShell
│   │   ├── ui/          # BaseButton, BaseCard, BaseInput, ToastHost
│   │   └── ...
│   ├── composables/     # useTheme, useWindow, useToast, useClipboard
│   ├── config/
│   │   └── tools.ts     # ⭐ 工具注册表 (单一数据源)
│   ├── router/          # 路由 (懒加载)
│   ├── styles/          # theme.css, base.css
│   └── views/
│       ├── HomeView.vue       # 仪表盘
│       └── tools/             # 五个工具页面
├── src-tauri/           # 极简 Rust 后端 (单实例/托盘/自启动/打开链接)
└── ...
```

## ➕ 添加新工具

工具箱采用**注册表驱动**,添加新工具只需三步:

1. 在 `src/config/tools.ts` 增加一条 `ToolMeta` 记录
2. 在 `src/router/index.ts` 增加一条懒加载路由
3. 在 `src/views/tools/` 实现组件 (套用 `ToolShell` 外壳)

## 📄 License

MIT
