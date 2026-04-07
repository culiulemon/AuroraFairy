# AuroraFairy

基于多模型协作架构的桌面 AI 助手

## 特性

- **多模型协作系统** - 索引模型智能调度 + 主模型执行任务
- **工具/触桥调度** - 支持文件操作、系统命令、API 调用等多种工具
- **桌面原生体验** - 基于 Tauri 构建的跨平台桌面应用
- **代码编辑支持** - 集成 CodeMirror 6 提供优质代码编辑体验

## 技术栈

- **前端框架**: Vue 3 + TypeScript + Vite
- **桌面框架**: Tauri v2 (Rust)
- **编辑器组件**: CodeMirror 6
- **状态管理**: Pinia
- **插件生态**: @tauri-apps/plugin-* 系列

## 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+
- pnpm 8+

### 安装依赖

```bash
pnpm install
```

### 运行开发服务器

```bash
pnpm tauri dev
```

### 构建生产版本

```bash
pnpm tauri build
```

## 项目结构

```
AuroraFairy/
├── src/                      # Vue 前端源码
│   ├── components/           # Vue 组件
│   │   ├── AgentToolsPage.vue
│   │   ├── AppLayout.vue
│   │   ├── ChatPageContent.vue
│   │   ├── ChatSidebar.vue
│   │   ├── ChatWindow.vue
│   │   ├── MessageItem.vue
│   │   ├── NavigationSidebar.vue
│   │   ├── SettingsPage.vue
│   │   └── SubNavigation.vue
│   ├── stores/               # Pinia 状态管理
│   │   ├── attachment.ts
│   │   ├── chat.ts
│   │   ├── conversation.ts
│   │   ├── conversationStore.ts
│   │   ├── db.ts
│   │   ├── settings.ts
│   │   ├── toolsStore.ts
│   │   └── useConversationStore.ts
│   ├── types/                # TypeScript 类型定义
│   │   └── tool.ts
│   ├── assets/               # 静态资源
│   ├── App.vue
│   └── main.ts
├── src-tauri/                # Rust 后端源码
│   ├── src/
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── capabilities/         # Tauri 权限配置
│   ├── icons/                 # 应用图标
│   └── tauri.conf.json       # Tauri 配置文件
├── public/                    # 公共资源
├── index.html
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## 架构设计

AuroraFairy 采用多模型协作架构，核心流程如下：

```
用户提出需求
       │
       ▼
┌─────────────────────┐
│  Agent: 组装提示词    │
│  [基础工具索引] +     │
│  [触桥索引] +         │
│  [用户需求]          │
└─────────────────────┘
       │
       ▼
┌─────────────────────┐
│  中间模型（索引模型） │
│  - 根据需求检索触桥   │
│  - 循环下钻定位      │
│  - 返回触桥地址      │
└─────────────────────┘
       │
       ▼
┌─────────────────────┐
│  Agent: 拼装触桥     │
│  [触桥内容] +        │
│  [用户需求]          │
└─────────────────────┘
       │
       ▼
┌─────────────────────┐
│  主模型             │
│  - 构建执行指令      │
│  - 返回可执行指令    │
└─────────────────────┘
       │
       ▼
┌─────────────────────┐
│  Agent: 执行        │
│  - 调用工具/触桥    │
│  - 收集执行结果     │
└─────────────────────┘
```

## 相关文档

- [系统提示词](提示词.md)
- [项目灵感](项目灵感.md)
