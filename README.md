# AuroraFairy

AI 智能体桌面客户端 | AI Agent Desktop Client

**Fairy** — 你的智能桌面精灵。

## 项目简介

AuroraFairy（产品名：**Fairy**）是一个基于 Tauri v2 构建的 AI 智能体桌面应用，集成了文件向量记忆系统、智能体技能系统、多渠道消息接入、安全规则引擎、本地模型管理、浏览器自动化等功能。应用采用 Vue 3 + TypeScript 作为前端框架，Rust 作为后端运行时，提供流畅的跨平台桌面体验。

<br />

用户交流
![用户交流二维码](public/qrcode.jpg?width=200)

<br />

### 技术栈

| 类别 | 技术 |
| --- | --- |
| 前端框架 | Vue 3 + TypeScript |
| 桌面框架 | Tauri v2 |
| 后端语言 | Rust |
| 构建工具 | Vite |
| 向量数据库 | Qdrant（内置） |
| 状态管理 | 模块级单例模式 |
| 持久化 | IndexedDB + localStorage |
| 代码编辑器 | CodeMirror 6 |

---

## 核心功能详解

### 1. 智能体系统 (Agent System)

#### 1.1 ReAct 推理循环

ReAct (Reasoning + Acting) 是智能体的核心推理引擎，实现了思考与行动的交替执行：

**核心流程** (`reactLoop.ts`):

```
用户消息 → 模型推理 → 工具调用 → 执行结果 → 模型再推理 → ... → 最终回复
```

**关键特性**:

- **迭代控制**: 支持最大迭代次数限制，防止无限循环
- **工具历史截断**: 当工具执行结果超过 20000 tokens 时自动二次截断
- **空响应重试**: 最多重试 2 次处理模型返回空内容的情况
- **流式输出**: 支持实时 chunk 输出和推理过程输出
- **工作目录越界检测**: 自动检测并申请临时访问授权
- **系统提示词装配器**: 动态组合系统提示词，集成记忆上下文

#### 1.2 工具执行引擎 (FairyDo)

`FairyDo` 是工具执行的核心管理器 (`fairyDo.ts`):

**功能特性**:

| 功能 | 说明 |
| --- | --- |
| 工具注册 | 支持静态工具定义和虚拟处理器 |
| 安全验证 | Shell 命令危险模式检测、路径穿越防护 |
| 超时控制 | 支持针对不同执行器的独立超时配置 |
| 结果序列化 | 自动将执行结果转为字符串，支持 JSON 格式化 |
| 聚合工具 | 支持 file_manager、task_manager 等聚合工具的分发 |

**执行器列表** (ExecutorName):

| 执行器 | 功能 |
| --- | --- |
| `shell_execute` | 执行 Shell 命令 |
| `file_read` | 读取文件内容 |
| `file_write` | 写入文件 |
| `file_edit` | 编辑文件 (SearchReplace 模式) |
| `file_delete` | 删除文件/目录 |
| `file_glob` | 按模式搜索文件 |
| `file_grep` | 内容搜索 |

#### 1.3 系统工具集 (FairySysTools)

系统内置的核心工具 (`FairySysTools.ts`):

| 工具名称 | 功能描述 |
| --- | --- |
| **执行 Shell** | 执行系统命令，支持 bash/cmd/powershell |
| **文件管理** | 文件的读、写、编辑、删除、glob、grep 操作 |
| **任务管理** | 创建任务清单、更新状态、委派子任务 |
| **浏览器控制** | 基于 FairyAction 的浏览器自动化 |
| **记忆搜索** | 搜索过往记忆，第一信息来源 |
| **触桥** | 与 FAP (FairyAction Package) 应用交互 |
| **角色配置** | 读取/修改智能体角色身份配置 |

#### 1.4 技能系统 (Skills)

可扩展的技能加载与管理架构 (`src/agent/skills/`):

| 模块 | 职责 |
| --- | --- |
| `skillLoader` | 技能加载器，从文件系统或远程加载技能 |
| `skillManager` | 技能管理器，注册、启用、禁用技能 |
| `skillSelector` | 技能选择器，根据上下文选择合适的技能 |
| `skillInjector` | 技能注入器，将技能注入到智能体执行上下文 |
| `skillEvolver` | 技能演化，动态优化和改进技能 |
| `types` | 技能系统类型定义 |

#### 1.5 子智能体 (SubAgent)

支持多智能体协作架构 (`subAgent.ts`):

- 独立执行上下文
- 可配置的工具白名单
- 最大迭代次数限制
- 独立的消息历史

---

### 2. 文件记忆系统 (FBM)

FBM (Fairy Brain Memory) 是基于向量的文件记忆检索系统：

#### 2.1 核心架构

```
FBM
├── QdrantStore           # 向量存储 (Qdrant 客户端)
├── DirectoryManager      # 目录管理器
├── BlockLifecycleManager # 记忆块生命周期管理
├── MemoryConsolidator    # 记忆整合器
├── MemoryRetriever       # 记忆检索器
├── MemoryReorganizer     # 记忆重组器
└── KeywordExtractor      # 关键词提取器
```

#### 2.2 核心组件

**QdrantStore** (`qdrant-store.ts`):
- 向量存储与检索
- 支持 BM25 全文搜索
- 批量嵌入处理
- 记忆块和目录双集合管理
- 通过 Tauri 内置 qdrant 二进制启动

**MemoryConsolidator** (`memory-consolidator.ts`):
- 将对话消息转化为记忆块
- 关键词提取
- 记忆整合与去重

**MemoryRetriever** (`memory-retriever.ts`):
- 语义检索
- 结果重排序 (rerank)
- 最小分数阈值过滤
- 上下文感知的检索增强

**MemoryReorganizer** (`memory-reorganizer.ts`):
- 定期记忆重组
- 优化记忆结构

**DirectoryManager** (`directory-manager.ts`):
- 目录结构管理
- 基于目录的检索

#### 2.3 核心文件模板

| 文件 | 用途 |
| --- | --- |
| `SOUL.md` | 灵魂设定，定义智能体核心人格 |
| `SYSPROMPT.md` | 系统提示词模板 |
| `HABIT.md` | 习惯记录 |
| `ABOUTUSER.md` | 关于用户的信息 |
| `REBIRTH.md` | 重生/重置配置 |

#### 2.4 检索配置

```typescript
interface RetrievalConfig {
  retrievalTopK: number       // 检索返回 top K 结果
  minScore: number            // 最小相关性分数
  refineResults: boolean      // 是否重排序结果
  directoryThreshold: number  // 目录检索阈值
}
```

---

### 3. 渠道集成系统 (Channel Integration)

多消息渠道的接入与统一处理 (`src/agent/channelBridge.ts`):

#### 3.1 渠道架构

```typescript
interface ChannelBridge {
  readonly channelId: string
  readonly displayName: string
  readonly badgeIcon: ChannelBadgeIcon

  start(deps: DispatcherDeps): Promise<void>
  stop(): void
  connect(accountId: string, config: ChannelAccountConfig): Promise<void>
  disconnect(accountId: string): Promise<void>
  getStatus(accountId: string): Promise<string>
  onStatusChange(callback: StatusChangeCallback): Promise<UnlistenFn>
  sendReply(accountId: string, externalChatId: string, messageText: string, replyContext: ReplyContext): Promise<void>
  getConfigFields(): ConfigFieldDefinition[]
  validateConfig(config: Record<string, unknown>): { valid: boolean; errors?: Record<string, string> }
}
```

#### 3.2 已集成的渠道

| 渠道 | 说明 | 相关文件 |
| --- | --- | --- |
| **飞书 (Feishu)** | 企业协作平台集成 | `feishuBridge.ts` |
| **微信 (WeChat)** | 即时通讯集成 | `weixinBridge.ts` |

#### 3.3 消息调度

`MessageDispatcher` 统一处理来自不同渠道的消息，提供:

- 消息路由
- 统一格式转换
- 状态回调

---

### 4. 上下文管理 (Context Management)

#### 4.1 Token 估算

`tokenEstimator.ts` - 精确的 Token 计数:

- 支持多种模型 (GPT-4, Claude, Gemini 等)
- 特殊令牌处理
- 消息批量估算

#### 4.2 对话摘要

`conversationSummarizer.ts` - 智能对话压缩:

**特性**:
- 支持增量摘要（合并已有摘要）
- 保留关键事实、用户意图、实体信息
- 时间线清晰

#### 4.3 上下文构建

`contextManager.ts` - 智能上下文管理:

```typescript
interface ContextBuildOptions {
  systemPrompt: string
  memoryContext: string
  messages: Array<...>
  conversationSummary?: string
  conversationId: string
  providerId: string
}
```

**策略**:
- 默认上下文窗口: 128000 tokens
- 保留最近 10 轮对话
- 摘要压缩旧消息
- 保留 4096 tokens 用于输出

---

### 5. API 提供者配置

支持多种 LLM 提供者 (`settings.ts`):

| 协议 | 提供者 |
| --- | --- |
| OpenAI 兼容 | OpenAI, DashScope, 自定义 |
| Anthropic | Claude 系列 |
| Google | Gemini 系列 |
| 自定义 | 支持任意兼容 API |

```typescript
interface ApiProvider {
  id: string
  displayName: string
  baseUrl: string
  apiKey: string
  model: string
  protocol: 'openai' | 'anthropic' | 'google' | 'custom'
  isDefault?: boolean
  thinkingEnabled?: boolean
  supportsTools?: boolean
  contextWindowSize?: number
}
```

---

### 6. 安全规则系统

`security-rules.json` - 可配置的安全规则:

**规则结构**:
```json
{
  "version": 1,
  "builtinOverrides": {},
  "customRules": []
}
```

**内置规则类别**:
| 类别 | 说明 |
| --- | --- |
| `shell` | Shell 命令危险模式 |
| `file` | 文件操作危险模式 |
| `network` | 网络请求危险模式 |

---

### 7. FAP 触桥系统 (TouchBridge)

FAP (FairyAction Package) 应用管理系统 (`fap.rs`, `TouchBridgePage.vue`):

| 功能 | 说明 |
| --- | --- |
| 应用管理 | 安装、卸载、列出 FAP 应用 |
| 消息桥接 | 与 FAP 应用双向通信 |
| 生命周期 | 启动、停止 FAP 桥接服务 |

---

### 8. 本地模型管理

支持本地 LLM 的下载、部署与推理 (`local_models.rs`, `LocalModelsPage.vue`):

| 功能 | 说明 |
| --- | --- |
| 环境检测 | 检查 OpenVINO / Transformers 运行环境 |
| 模型搜索 | 搜索可下载的开源模型 |
| 模型下载 | 支持断点续传的模型下载 |
| 模型部署 | 本地部署启动推理服务 |
| IR 转换 | 模型转换为 OpenVINO IR 格式 |
| 依赖管理 | 自动安装 Python 依赖 |

---

### 9. 调试面板 (Debug Console)

内置实时调试日志查看器 (`debug.html` + `debug.ts`):

- 实时捕获应用日志
- 按级别过滤 (request / response / error / info)
- 自动滚动与展开/折叠详情
- 日志计数与一键清空

---

## 页面导航

```
NavigationSidebar (主侧栏)
├── 💬 聊天 (Chat)
│   └── 聊天窗口
├── ⚙️ 控制 (Control)
│   ├── 渠道管理 (Channels)
│   ├── 定时任务 (Scheduled Tasks)
│   ├── 生命管理 (Life)
│   └── 安全规则 (Security Rules)
├── 🤖 智能体 (Agent)
│   ├── 工作区 (Workspace)
│   ├── 角色 (Role)
│   ├── 技能 (Skills)
│   ├── 工具 (Tools)
│   ├── 记忆 (Memory)
│   └── 应用管理 (TouchBridge)
└── ⚡ 设置 (Settings)
    ├── 模型 (Models)
    ├── 本地模型 (Local Models)
    ├── 个性化 (Personalization)
    ├── 杂项 (Misc)
    └── 关于 (About)
```

| 页面 | 组件 | 说明 |
| --- | --- | --- |
| 主聊天 | `ChatWindow.vue` | 核心对话界面 |
| 聊天列表 | `ChatSidebar.vue` | 对话历史列表 |
| 聊天内容 | `ChatPageContent.vue` | 聊天区域整合布局 |
| 智能体工具 | `AgentToolsPage.vue` | 配置 Agent 可用工具 |
| 记忆管理 | `AgentMemoryPage.vue` | FBM 记忆检索与统计 |
| 技能管理 | `SkillsPage.vue` | 技能加载与配置 |
| 安全规则 | `SecurityRulesPage.vue` | 配置安全规则 |
| 渠道管理 | `ChannelsPage.vue` | 飞书/微信渠道配置 |
| 本地模型 | `LocalModelsPage.vue` | 本地 LLM 下载与部署 |
| 角色配置 | `RolePage.vue` | Agent 角色身份配置 |
| 个性化 | `PersonalizationPage.vue` | 主题、语言等 |
| 应用设置 | `SettingsPage.vue` | API 提供者、全局设置 |
| 杂项设置 | `MiscSettingsPage.vue` | 工作目录、额外配置 |
| 应用管理 | `TouchBridgePage.vue` | FAP 应用安装与管理 |
| 任务面板 | `TaskPanel.vue` | 任务编排与执行展示 |
| 工具调用块 | `ToolCallBlock.vue` | 工具调用可视化展示 |
| 工具审批 | `ToolApprovalDialog.vue` | 危险操作审批对话框 |
| 关于 | `AboutPage.vue` | 应用信息与开源鸣谢 |

---

## 项目结构

```
AuroraFairy/
├── src/                                # 前端源代码 (Vue 3 + TypeScript)
│   ├── agent/                          # 智能体核心模块
│   │   ├── skills/                     # 技能系统
│   │   │   ├── skillLoader.ts          # 技能加载器
│   │   │   ├── skillManager.ts         # 技能管理器
│   │   │   ├── skillSelector.ts        # 技能选择器
│   │   │   ├── skillInjector.ts        # 技能注入器
│   │   │   ├── skillEvolver.ts         # 技能演化
│   │   │   └── types.ts                # 类型定义
│   │   ├── fairyDo.ts                  # 工具执行引擎 ⭐
│   │   ├── fairyTask.ts                # 任务管理
│   │   ├── reactLoop.ts                # ReAct 推理循环 ⭐
│   │   ├── subAgent.ts                 # 子智能体
│   │   ├── FairySysTools.ts            # 系统工具集 ⭐
│   │   ├── channelBridge.ts            # 渠道桥接接口
│   │   ├── channelRegistry.ts          # 渠道注册表
│   │   ├── contextManager.ts           # 上下文管理 ⭐
│   │   ├── conversationSummarizer.ts   # 对话摘要 ⭐
│   │   ├── tokenEstimator.ts           # Token 估算
│   │   ├── messageDispatcher.ts        # 消息调度
│   │   ├── systemPromptAssembler.ts    # 系统提示词装配
│   │   ├── virtualHandlers.ts          # 虚拟工具处理器
│   │   ├── feishuBridge.ts             # 飞书渠道
│   │   └── weixinBridge.ts             # 微信渠道
│   │
│   ├── stores/                         # 状态管理 (模块单例模式)
│   │   ├── settings.ts                 # 应用配置 (localStorage) ⭐
│   │   ├── db.ts                       # IndexedDB 封装
│   │   ├── chat.ts                     # API 通信层 (流式/非流式) ⭐
│   │   ├── conversation.ts             # 数据结构定义 + 纯函数
│   │   ├── conversationStore.ts        # 对话管理运行时
│   │   ├── useConversationStore.ts     # 对话组合式逻辑
│   │   ├── fbmStore.ts                 # FBM 记忆系统 ⭐
│   │   ├── memoryReorganizer.ts        # 记忆重组逻辑
│   │   ├── toolsStore.ts               # 工具管理
│   │   ├── channelStore.ts             # 渠道状态
│   │   ├── securityStore.ts            # 安全规则
│   │   ├── localModels.ts              # 本地模型管理
│   │   ├── miscSettings.ts             # 杂项设置
│   │   ├── attachment.ts               # 附件管理
│   │   └── debugStore.ts               # 调试日志
│   │
│   ├── components/                     # Vue 组件
│   │   ├── AppLayout.vue               # 主布局（导航 + 内容区）
│   │   ├── NavigationSidebar.vue       # 主侧栏导航
│   │   ├── SubNavigation.vue           # 二级导航
│   │   ├── ChatWindow.vue              # 主聊天窗口
│   │   ├── ChatSidebar.vue             # 聊天侧边栏
│   │   ├── ChatPageContent.vue         # 聊天页整体布局
│   │   ├── MessageItem.vue             # 消息项
│   │   ├── ToolCallBlock.vue           # 工具调用块
│   │   ├── TaskPanel.vue               # 任务面板
│   │   ├── TaskOrchestrationBlock.vue  # 任务编排展示
│   │   ├── ToolApprovalDialog.vue      # 工具审批对话框
│   │   ├── BaseDialog.vue              # 通用对话框
│   │   ├── BaseSelect.vue              # 通用选择器
│   │   ├── AgentToolsPage.vue          # 智能体工具配置
│   │   ├── AgentMemoryPage.vue         # 记忆管理
│   │   ├── SkillsPage.vue              # 技能管理
│   │   ├── SecurityRulesPage.vue       # 安全规则配置
│   │   ├── ChannelsPage.vue            # 渠道管理
│   │   ├── LocalModelsPage.vue         # 本地模型
│   │   ├── PersonalizationPage.vue     # 个性化设置
│   │   ├── RolePage.vue                # 角色配置
│   │   ├── SettingsPage.vue            # 应用设置
│   │   ├── MiscSettingsPage.vue        # 杂项设置
│   │   ├── TouchBridgePage.vue         # FAP 应用管理
│   │   ├── AboutPage.vue               # 关于页面
│   │   └── OpensourceDialog.vue        # 开源许可对话框
│   │
│   ├── composables/                    # Vue 组合式函数
│   │   ├── useTheme.ts                 # 主题管理
│   │   ├── useModelManager.ts          # 模型管理
│   │   ├── useTTS.ts                   # 语音合成
│   │   ├── useMarkdown.ts              # Markdown 渲染
│   │   └── useToolWatcher.ts           # 工具文件监视
│   │
│   ├── types/                          # 类型定义
│   │   ├── tool.ts                     # 工具类型
│   │   ├── message.ts                  # 消息类型
│   │   ├── channel.ts                  # 渠道类型
│   │   ├── task.ts                     # 任务类型
│   │   └── security.ts                 # 安全类型
│   │
│   ├── assets/                         # 静态资源
│   │   ├── styles/
│   │   │   └── themes.css              # 主题 CSS 变量
│   │   └── vue.svg
│   │
│   ├── App.vue                         # 根组件
│   ├── main.ts                         # 入口
│   ├── debug.ts                        # 调试面板脚本
│   └── vite-env.d.ts                   # Vite 类型声明
│
├── src-tauri/                          # Rust 后端源代码
│   ├── src/
│   │   ├── commands/                   # Tauri 命令模块
│   │   │   ├── shell.rs                # Shell 执行
│   │   │   ├── file.rs                 # 文件操作
│   │   │   ├── watcher.rs              # 文件监视器
│   │   │   ├── security_rules.rs       # 安全规则管理
│   │   │   ├── tts.rs                  # 语音合成
│   │   │   ├── local_models.rs         # 本地模型管理
│   │   │   ├── browser.rs              # 浏览器自动化 ⭐
│   │   │   ├── proxy.rs                # API 代理转发
│   │   │   ├── fbm_fs.rs               # FBM 文件系统
│   │   │   ├── qdrant_manager.rs       # Qdrant 向量数据库管理
│   │   │   ├── feishu.rs               # 飞书 API
│   │   │   ├── weixin.rs               # 微信 API
│   │   │   ├── fap.rs                  # FAP 触桥系统
│   │   │   └── mod.rs                  # 模块入口
│   │   ├── lib.rs                      # 库入口（应用核心初始化）
│   │   └── main.rs                     # 程序入口
│   │
│   ├── binaries/                       # 内置二进制
│   │   ├── x86_64/
│   │   │   ├── fairy-action.exe        # 浏览器自动化引擎 (x64)
│   │   │   └── qdrant.exe              # 向量数据库 (x64)
│   │   ├── aarch64/
│   │   │   ├── fairy-action.exe        # 浏览器自动化引擎 (ARM64)
│   │   │   └── qdrant.exe              # 向量数据库 (ARM64)
│   │   ├── fairy-action.exe            # 默认架构
│   │   └── qdrant.exe                  # 默认架构
│   │
│   ├── capabilities/
│   │   └── default.json                # Tauri 权限声明
│   ├── icons/                          # 全平台应用图标
│   ├── Cargo.toml                      # Rust 依赖
│   ├── tauri.conf.json                 # Tauri 配置
│   └── build.rs                        # 构建脚本
│
├── scripts/                            # 构建与工具脚本
│   ├── build-fairy-action.mjs          # 构建 fairy-action
│   ├── prepare-binaries.mjs            # 准备二进制文件
│   ├── increment-version.mjs           # 版本号递增
│   ├── convert_model.py                # 模型转换 (OpenVINO IR)
│   ├── download_model.py               # 模型下载
│   ├── openvino_server.py              # OpenVINO 推理服务
│   └── transformers_server.py          # Transformers 推理服务
│
├── public/                             # 静态资源
│   ├── fairyicon.svg                   # 应用图标 SVG
│   └── qrcode.jpg                      # 用户交流二维码
│
├── debug.html                          # 调试面板页面
├── index.html                          # 入口 HTML
├── security-rules.json                 # 安全规则配置文件
├── package.json                        # Node 依赖
├── vite.config.ts                      # Vite 构建配置
├── tsconfig.json                       # TypeScript 配置
└── LICENSE                             # 开源许可
```

---

## 架构设计

### 状态管理

项目采用 **模块级单例模式** 管理状态，而非 Pinia 或 Vuex：

```
stores/
├── settings.ts     → localStorage (应用配置持久化)
├── db.ts           → IndexedDB (大数据持久化)
├── conversation.ts → 数据结构定义 + 纯函数
├── chat.ts         → API 通信层 (无状态)
├── fbmStore.ts     → 单例对象 (FBM 记忆系统)
├── conversationStore.ts → 对话运行时状态
└── ...
```

**架构特点**:
- 无 Pinia / Vuex 依赖
- 纯 TypeScript 模块级状态管理
- 分层设计：类型定义、持久化、业务逻辑分离
- IndexedDB 存储对话历史、附件、工具定义
- localStorage 存储配置设定

### 路由系统

项目未使用 vue-router，采用 **条件渲染 + 异步组件** 实现页面切换：

```
NavigationSidebar (一级导航)
    ↓
SubNavigation (二级导航)
    ↓
动态异步加载组件 → 通过 v-if/v-else 渲染
```

### 双通信模式

AI API 调用支持两种模式：
1. **前端直连**: 浏览器直接调用 AI 提供商 API
2. **后端代理**: 通过 Tauri Rust 后端转发请求，支持流式 SSE 推送

---

## Rust 后端命令一览

| 命令模块 | 功能 |
| --- | --- |
| `shell_execute` | 系统命令执行 (bash/cmd/powershell) |
| `file_read/write/edit/delete/glob/grep` | 文件系统操作 |
| `browser_start/execute/stop` | 浏览器自动化控制 |
| `proxy_chat/proxy_chat_stream` | AI API 代理转发 (流式/非流式) |
| `qdrant_start/stop/status` | Qdrant 向量数据库管理 |
| `fbm_*` | FBM 文件系统操作 (mkdir/write/read/unlink/readdir/stat/watch) |
| `feishu_*` | 飞书渠道 (connect/disconnect/reply/status) |
| `weixin_*` | 微信渠道 (qrcode/connect/disconnect/reply/status) |
| `fap_bridge_*` | FAP 触桥 (start/send/stop) + FAP 安装/卸载/列表 |
| `tts_generate/list_voices` | 语音合成 |
| `check_environment/search/download/deploy/*` | 本地模型全生命周期管理 |
| `security_rules_*` | 安全规则 CRUD |
| `start_tool_watcher` | 工具文件变更监视 |
| `scan_tools/save/delete_tool_file` | 工具文件管理 |

---

## 环境要求

| 依赖 | 版本要求 |
| --- | --- |
| Node.js | >= 18 |
| pnpm | >= 8 |
| Rust | >= 1.70 |
| Windows | 10/11 (主要支持) |

---

## 安装与构建

### 安装依赖

```bash
pnpm install
```

### 开发运行

```bash
# 构建 fairy-action 并启动 Vite 开发服务器
pnpm dev

# 启动 Tauri 开发模式（完整桌面应用）
pnpm tauri dev
```

### 构建应用

```bash
# 构建 Windows 安装包
pnpm tauri build

# 针对特定架构构建
pnpm tauri:build:x64    # x86_64 架构
pnpm tauri:build:arm64  # ARM64 架构
```

### 前端单独构建

```bash
pnpm build  # 构建前端资源
pnpm tauri build  # 完整构建
```

### 清理

```bash
pnpm clean  # 清理 dist 和 cargo 构建
```

---

## 内置二进制说明

| 二进制 | 路径 | 用途 |
| --- | --- | --- |
| `qdrant.exe` | `src-tauri/binaries/` | 轻量级向量数据库，用于 FBM 记忆存储 |
| `fairy-action.exe` | `src-tauri/binaries/` | 浏览器自动化执行引擎，支持无头模式 |

---

## 配置与数据

### 设置存储

| 存储位置 | 用途 |
| --- | --- |
| `localStorage['aurorafairy-settings']` | API 提供商、角色设定、FBM 配置 |
| `localStorage['aurorafairy-theme-settings']` | 主题（亮色/暗色）、主色 |
| `localStorage['aurorafairy-tts-settings']` | 语音合成参数 |
| IndexedDB (`aurorafairy-db` v6) | 对话记录、附件、工具定义 |

### 数据目录

应用运行时会自动创建以下目录结构：

```
AppData/
├── FairyTool/           # 用户自定义工具目录
├── FairyWorkSpace/      # 智能体工作空间
├── models/              # 本地模型存储
└── config.json          # 应用配置文件（工作目录、模型目录等）
```

### 安全规则

配置文件: `security-rules.json`

---

## 开源许可

本项目基于多项优秀开源项目构建，详见应用内的「关于」页面 → 「开源鸣谢」。

---

© 2024-2026 AuroraFairy. All rights reserved. v0.1.17
