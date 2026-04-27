# AuroraFairy

AI 智能体桌面客户端 | AI Agent Desktop Client

## 项目简介

AuroraFairy 是一个基于 Tauri v2 构建的 AI 智能体桌面应用，集成了文件向量存储、智能体技能系统、多渠道消息接入、安全规则引擎等功能。应用采用 Vue 3 + TypeScript 作为前端框架，Rust 作为后端运行时，提供流畅的跨平台桌面体验。

<br />

用户交流
![用户交流二维码](public/qrcode.jpg?width=200)

<br />

### 技术栈

| 类别    | 技术                 |
| ----- | ------------------ |
| 前端框架  | Vue 3 + TypeScript |
| 桌面框架  | Tauri v2           |
| 后端语言  | Rust               |
| 构建工具  | Vite               |
| 向量数据库 | Qdrant (内置)        |
| 状态管理  | Pinia (隐式)         |

***

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

**关键类型**:

```typescript
interface ReActConfig {
  maxIterations?: number      // 最大迭代次数
  tools: Tool[]              // 可用工具列表
  extraAllowedPaths?: string[] // 额外允许访问的路径
  onTurnStart?: (turnNumber: number) => void
  onChunk?: (chunk: string) => void
  onReasoningChunk?: (reasoning: string) => void
  onToolExecuting?: (toolCallId: string, tool: string, input: Record<string, unknown>) => void
  onToolResult?: (tool: string, result: ToolResult) => void
  onApproveAccess?: (toolName: string, targetPath: string) => Promise<boolean>
  signal?: AbortSignal
}
```

#### 1.2 工具执行引擎 (FairyDo)

`FairyDo` 是工具执行的核心管理器 (`fairyDo.ts`):

**功能特性**:

| 功能    | 说明                                      |
| ----- | --------------------------------------- |
| 工具注册  | 支持静态工具定义和虚拟处理器                          |
| 安全验证  | Shell 命令危险模式检测、路径穿越防护                   |
| 超时控制  | 支持针对不同执行器的独立超时配置                        |
| 结果序列化 | 自动将执行结果转为字符串，支持 JSON 格式化                |
| 聚合工具  | 支持 file\_manager、task\_manager 等聚合工具的分发 |

**内置安全规则** (BUILTIN\_SECURITY\_RULES):

```typescript
// Shell 类危险命令防护
{ pattern: 'rm -rf /', category: 'shell' }           // 删除根目录
{ pattern: 'mkfs', category: 'shell' }                // 格式化文件系统
{ pattern: ':(){ :|:& };:', category: 'shell' }      // Fork 炸弹
{ pattern: 'chmod -R 777 /', category: 'shell' }     // 全局提权
{ pattern: 'dd if=/dev/zero', category: 'shell' }    // 磁盘覆写
{ pattern: '| sh', category: 'shell' }                // 管道到 shell
// ... 更多规则见 security-rules.json
```

**执行器列表** (ExecutorName):

| 执行器             | 功能                      |
| --------------- | ----------------------- |
| `shell_execute` | 执行 Shell 命令             |
| `file_read`     | 读取文件内容                  |
| `file_write`    | 写入文件                    |
| `file_edit`     | 编辑文件 (SearchReplace 模式) |
| `file_delete`   | 删除文件/目录                 |
| `file_glob`     | 按模式搜索文件                 |
| `file_grep`     | 内容搜索                    |

#### 1.3 系统工具集 (FairySysTools)

系统内置的核心工具 (`FairySysTools.ts`):

| 工具名称         | 功能描述                             |
| ------------ | -------------------------------- |
| **执行 Shell** | 执行系统命令，支持 bash/cmd/powershell    |
| **文件管理**     | 文件的读、写、编辑、删除、glob、grep 操作        |
| **任务管理**     | 创建任务清单、更新状态、委派子任务                |
| **浏览器控制**    | 基于 FairyAction 的浏览器自动化           |
| **记忆搜索**     | 搜索过往记忆，第一信息来源                    |
| **触桥**       | 与 FAP (FairyAction Package) 应用交互 |
| **角色配置**     | 读取/修改智能体角色身份配置                   |

**文件管理工具** (file\_manager):

```json
{
  "action": "read|write|edit|delete|glob|grep",
  "path": "文件路径",
  "pattern": "搜索模式",
  "offset": 1,        // 起始行号
  "limit": 100,      // 最大行数
  "content": "内容",  // write 时需要
  "oldStr": "原始",   // edit 时需要
  "newStr": "替换"    // edit 时需要
}
```

**浏览器控制工具** (browser):

支持的 action: `start`, `stop`, `navigate`, `click`, `input`, `scroll`, `send_keys`, `screenshot`, `extract`, `get_dom`, `get_state`, `go_back`, `go_forward`, `reload`, `switch_tab`, `close_tab`, `new_tab`, `search`, `evaluate`, `select_option`, `toggle_annotations`, `save_to_file`, `read_file`, `wait`, `done`

#### 1.4 技能系统 (Skills)

可扩展的技能加载与管理架构 (`src/agent/skills/`):

| 模块              | 职责                   |
| --------------- | -------------------- |
| `skillLoader`   | 技能加载器，从文件系统或远程加载技能   |
| `skillManager`  | 技能管理器，注册、启用、禁用技能     |
| `skillSelector` | 技能选择器，根据上下文选择合适的技能   |
| `skillInjector` | 技能注入器，将技能注入到智能体执行上下文 |
| `skillEvolver`  | 技能演化，动态优化和改进技能       |
| `types`         | 技能系统类型定义             |

#### 1.5 子智能体 (SubAgent)

支持多智能体协作架构 (`subAgent.ts`):

- 独立执行上下文
- 可配置的工具白名单
- 最大迭代次数限制
- 独立的消息历史

***

### 2. 文件记忆系统 (FBM)

FBM (File-based Memory) 是基于向量的文件记忆检索系统 (`src/fbm/`):

#### 2.1 核心架构

```
FBM
├── QdrantStore          # 向量存储 (Qdrant 客户端)
├── DirectoryManager     # 目录管理器
├── BlockLifecycleManager # 记忆块生命周期管理
├── MemoryConsolidator   # 记忆整合器
├── MemoryRetriever      # 记忆检索器
└── MemoryReorganizer    # 记忆重组器
```

#### 2.2 核心组件

**QdrantStore** (`qdrant-store.ts`):

- 向量存储与检索
- 支持 BM25 全文搜索
- 批量嵌入处理
- 记忆块和目录双集合管理

**MemoryConsolidator** (`memory-consolidator.ts`):

- 将对话消息转化为记忆块
- 关键词提取
- 记忆整合与去重

**MemoryRetriever** (`memory-retriever.ts`):

- 语义检索
- 结果重排序 (rerank)
- 最小分数阈值过滤
- 上下文感知的检索增强

**DirectoryManager** (`directory-manager.ts`):

- 目录结构管理
- 基于目录的检索

#### 2.3 检索配置

```typescript
interface RetrievalConfig {
  retrievalTopK: number       // 检索返回 top K 结果
  minScore: number            // 最小相关性分数
  refineResults: boolean      // 是否重排序结果
  directoryThreshold: number  // 目录检索阈值
}
```

#### 2.4 生命周期管理

- **合并检查**: 可配置的检查间隔
- **过期机制**: 可选的记忆过期删除
- **自动整合**: 对话完成后自动触发整合

***

### 3. 渠道集成系统 (Channel Integration)

多消息渠道的接入与统一处理 (`src/agent/channelBridge.ts`):

#### 3.1 渠道架构

```typescript
interface ChannelBridge {
  readonly channelId: string           // 渠道唯一标识
  readonly displayName: string          // 显示名称
  readonly badgeIcon: ChannelBadgeIcon // 徽章图标

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

| 渠道              | 说明       | 相关文件              |
| --------------- | -------- | ----------------- |
| **飞书 (Feishu)** | 企业协作平台集成 | `feishuBridge.ts` |
| **微信 (WeChat)** | 即时通讯集成   | `weixinBridge.ts` |

#### 3.3 消息调度

`MessageDispatcher` 统一处理来自不同渠道的消息，提供:

- 消息路由
- 统一格式转换
- 状态回调

***

### 4. 上下文管理 (Context Management)

#### 4.1 Token 估算

`tokenEstimator.ts` - 精确的 Token 计数:

- 支持多种模型 (GPT-4, Claude, Gemini 等)
- 特殊令牌处理
- 消息批量估算

#### 4.2 对话摘要

`conversationSummarizer.ts` - 智能对话压缩:

```typescript
// 摘要生成
generateSummary(
  messages: Array<{ role: string; content: string }>,
  existingSummary?: string,
  providerId?: string
): Promise<string | null>
```

**特性**:

- 支持增量摘要 (合并已有摘要)
- 保留关键事实、用户意图、实体信息
- 时间线清晰

#### 4.3 上下文构建

`contextManager.ts` - 智能上下文管理:

```typescript
interface ContextBuildOptions {
  systemPrompt: string           // 系统提示词
  memoryContext: string          // 记忆上下文
  messages: Array<...>          // 历史消息
  conversationSummary?: string  // 对话摘要
  conversationId: string
  providerId: string
}
```

**策略**:

- 默认上下文窗口: 128000 tokens
- 保留最近 10 轮对话
- 摘要压缩旧消息
- 保留 4096 tokens 用于输出

***

### 5. API 提供者配置

支持多种 LLM 提供者 (`settings.ts`):

| 协议        | 提供者                    |
| --------- | ---------------------- |
| OpenAI 兼容 | OpenAI, DashScope, 自定义 |
| Anthropic | Claude 系列              |
| Google    | Gemini 系列              |
| 自定义       | 支持任意兼容 API             |

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

***

### 6. 安全规则系统

`security-rules.json` - 可配置的安全规则:

**规则结构**:

```json
{
  "id": "unique-rule-id",
  "pattern": "危险模式",
  "category": "shell|file|network",
  "description": "规则描述",
  "enabled": true,
  "isBuiltIn": true
}
```

**内置规则类别**:

| 类别        | 说明           |
| --------- | ------------ |
| `shell`   | Shell 命令危险模式 |
| `file`    | 文件操作危险模式     |
| `network` | 网络请求危险模式     |

***

## 项目结构

```
AuroraFairy/
├── src/                              # 前端源代码 (Vue 3 + TypeScript)
│   ├── agent/                        # 智能体核心模块
│   │   ├── skills/                   # 技能系统
│   │   │   ├── skillLoader.ts       # 技能加载器
│   │   │   ├── skillManager.ts      # 技能管理器
│   │   │   ├── skillSelector.ts     # 技能选择器
│   │   │   ├── skillInjector.ts     # 技能注入器
│   │   │   ├── skillEvolver.ts      # 技能演化
│   │   │   └── types.ts             # 类型定义
│   │   ├── fairyDo.ts               # 工具执行引擎 ⭐
│   │   ├── fairyTask.ts             # 任务管理
│   │   ├── reactLoop.ts             # ReAct 推理循环 ⭐
│   │   ├── subAgent.ts              # 子智能体
│   │   ├── FairySysTools.ts         # 系统工具集 ⭐
│   │   ├── channelBridge.ts         # 渠道桥接接口
│   │   ├── channelRegistry.ts       # 渠道注册表
│   │   ├── contextManager.ts        # 上下文管理 ⭐
│   │   ├── conversationSummarizer.ts # 对话摘要 ⭐
│   │   ├── tokenEstimator.ts        # Token 估算
│   │   ├── messageDispatcher.ts     # 消息调度
│   │   ├── virtualHandlers.ts       # 虚拟工具处理器
│   │   ├── feishuBridge.ts          # 飞书渠道
│   │   └── weixinBridge.ts          # 微信渠道
│   │
│   ├── fbm/                         # 文件记忆系统 (FBM)
│   │   └── src/
│   │       ├── core/                # 核心功能
│   │       │   ├── fbm.ts           # FBM 主类 ⭐
│   │       │   ├── qdrant-store.ts # Qdrant 向量存储
│   │       │   ├── memory-retriever.ts    # 记忆检索
│   │       │   ├── memory-consolidator.ts # 记忆整合
│   │       │   ├── memory-reorganizer.ts  # 记忆重组
│   │       │   ├── directory-manager.ts   # 目录管理
│   │       │   ├── block-lifecycle.ts    # 生命周期
│   │       │   ├── keyword-extractor.ts  # 关键词提取
│   │       │   ├── json-utils.ts         # JSON 工具
│   │       │   └── adapters/             # LLM 适配器
│   │       └── types/                   # 类型定义
│   │
│   ├── components/                  # Vue 组件
│   │   ├── ChatWindow.vue           # 主聊天窗口
│   │   ├── ChatSidebar.vue          # 聊天侧边栏
│   │   ├── MessageItem.vue          # 消息项
│   │   ├── AgentToolsPage.vue       # 智能体工具配置
│   │   ├── AgentMemoryPage.vue      # 记忆管理
│   │   ├── SkillsPage.vue           # 技能管理
│   │   ├── SecurityRulesPage.vue    # 安全规则配置
│   │   ├── ChannelsPage.vue         # 渠道管理
│   │   ├── LocalModelsPage.vue      # 本地模型
│   │   ├── PersonalizationPage.vue  # 个性化设置
│   │   ├── RolePage.vue             # 角色配置
│   │   ├── SettingsPage.vue         # 应用设置
│   │   ├── AboutPage.vue            # 关于页面
│   │   └── ...
│   │
│   ├── stores/                       # 状态管理
│   │   ├── settings.ts              # 设置 (API提供者等) ⭐
│   │   ├── chat.ts                  # 聊天状态
│   │   ├── conversation.ts          # 对话管理
│   │   ├── tools.ts                 # 工具管理
│   │   ├── securityStore.ts         # 安全规则
│   │   ├── channelStore.ts          # 渠道状态
│   │   └── ...
│   │
│   ├── composables/                  # Vue 组合式函数
│   │   ├── useModelManager.ts        # 模型管理
│   │   ├── useTheme.ts              # 主题管理
│   │   ├── useTTS.ts                # 语音合成
│   │   └── ...
│   │
│   ├── types/                        # 类型定义
│   │   ├── tool.ts                  # 工具类型
│   │   ├── message.ts               # 消息类型
│   │   ├── channel.ts              # 渠道类型
│   │   ├── task.ts                 # 任务类型
│   │   └── security.ts             # 安全类型
│   │
│   └── App.vue                       # 根组件
│
├── src-tauri/                        # Rust 后端源代码
│   ├── src/
│   │   ├── commands/                 # Tauri 命令
│   │   │   ├── file.rs              # 文件操作命令
│   │   │   ├── shell.rs             # Shell 执行命令
│   │   │   ├── browser.rs           # 浏览器控制命令
│   │   │   ├── feishu.rs            # 飞书 API
│   │   │   ├── weixin.rs            # 微信 API
│   │   │   ├── qdrant_manager.rs    # Qdrant 管理
│   │   │   ├── security_rules.rs    # 安全规则
│   │   │   ├── tts.rs               # 语音合成
│   │   │   └── ...
│   │   ├── lib.rs                    # 库入口
│   │   └── main.rs                   # 程序入口
│   │
│   ├── binaries/                     # 内置二进制
│   │   ├── qdrant.exe               # 向量数据库服务
│   │   └── fairy-action.exe          # 动作执行引擎
│   │
│   ├── icons/                        # 应用图标
│   └── tauri.conf.json               # Tauri 配置
│
├── scripts/                           # 构建脚本
│   ├── build-fairy-action.mjs       # 构建 fairy-action
│   ├── prepare-binaries.mjs          # 准备二进制文件
│   ├── convert_model.py             # 模型转换
│   ├── download_model.py             # 模型下载
│   └── transformers_server.py       # Transformer 服务
│
├── public/                            # 静态资源
└── index.html                         # 入口 HTML
```

***

## 功能页面

| 页面    | 文件                        | 说明            |
| ----- | ------------------------- | ------------- |
| 主聊天   | `ChatWindow.vue`          | 核心对话界面        |
| 聊天侧边栏 | `ChatSidebar.vue`         | 对话列表管理        |
| 智能体工具 | `AgentToolsPage.vue`      | 配置 Agent 可用工具 |
| 记忆管理  | `AgentMemoryPage.vue`     | FBM 记忆检索与统计   |
| 技能管理  | `SkillsPage.vue`          | 技能加载与配置       |
| 安全规则  | `SecurityRulesPage.vue`   | 配置安全规则        |
| 渠道管理  | `ChannelsPage.vue`        | 飞书/微信渠道配置     |
| 本地模型  | `LocalModelsPage.vue`     | 本地 LLM 配置     |
| 个性化   | `PersonalizationPage.vue` | 主题、语言等        |
| 角色配置  | `RolePage.vue`            | Agent 角色身份配置  |
| 应用设置  | `SettingsPage.vue`        | API 提供者、全局设置  |
| 关于    | `AboutPage.vue`           | 应用信息          |

***

## 环境要求

| 依赖      | 版本要求         |
| ------- | ------------ |
| Node.js | >= 18        |
| pnpm    | >= 8         |
| Rust    | >= 1.70      |
| Windows | 10/11 (主要支持) |

***

## 安装与构建

### 安装依赖

```bash
pnpm install
```

### 开发运行

```bash
# 启动 Tauri 开发模式
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

***

## 内置二进制说明

| 二进制                | 路径                    | 用途                   |
| ------------------ | --------------------- | -------------------- |
| `qdrant.exe`       | `src-tauri/binaries/` | 轻量级向量数据库，用于 FBM 记忆存储 |
| `fairy-action.exe` | `src-tauri/binaries/` | 浏览器自动化执行引擎，支持无头模式    |

***

## 配置与数据

### 设置存储

- **API 设置**: `localStorage['aurorafairy-settings']`
- **主题设置**: `localStorage['aurorafairy-theme-settings']`
- **TTS 设置**: `localStorage['aurorafairy-tts-settings']`

### 安全规则

配置文件: `security-rules.json`

***

## 开源许可

本项目基于多项优秀开源项目构建，详见应用内的「关于」页面 → 「开源鸣谢」。

***

© 2024-2026 AuroraFairy. All rights reserved.
