import { invoke } from '@tauri-apps/api/core'
import type { ToolParameter, Tool } from '../types/tool'
import type { SecurityRule } from '../types/security'

export type VirtualToolName = 'todo_write' | 'task_dispatch' | 'browser'

export type AggregateToolName = 'file_manager' | 'task_manager' | 'browser'

export type FileAction = 'read' | 'write' | 'edit' | 'delete' | 'glob' | 'grep'
export type TaskAction = 'create_todo' | 'update_todo' | 'dispatch'

export const FILE_ACTION_MAP: Record<FileAction, ExecutorName> = {
  read: 'file_read',
  write: 'file_write',
  edit: 'file_edit',
  delete: 'file_delete',
  glob: 'file_glob',
  grep: 'file_grep',
}

export const TASK_ACTION_MAP: Record<TaskAction, string> = {
  create_todo: 'todo_write',
  update_todo: 'todo_write',
  dispatch: 'task_dispatch',
}

const TASK_ACTION_TO_INTERNAL: Record<TaskAction, string> = {
  create_todo: 'create',
  update_todo: 'update',
  dispatch: 'dispatch',
}

export const AGGREGATE_ACTION_MAP: Record<AggregateToolName, Record<string, ExecutorName | string>> = {
  file_manager: FILE_ACTION_MAP,
  task_manager: TASK_ACTION_MAP,
  browser: { _default: 'browser' },
}

export function isAggregateTool(toolName: string): toolName is AggregateToolName {
  return toolName in AGGREGATE_ACTION_MAP
}

export function resolveAction(toolName: AggregateToolName, input: Record<string, unknown>): { executor: string; remainingInput: Record<string, unknown> } | null {
  if (toolName === 'browser') {
    const action = input.action as string
    if (!action) return null
    return { executor: 'browser', remainingInput: input }
  }
  const actionMap = AGGREGATE_ACTION_MAP[toolName]
  const action = input.action as string
  if (!action || !(action in actionMap)) return null
  const executor = actionMap[action]
  const { action: _removed, ...remainingInput } = input
  if (toolName === 'task_manager') {
    remainingInput.action = TASK_ACTION_TO_INTERNAL[action as TaskAction]
  }
  return { executor, remainingInput }
}

export type ExecutorName =
  | 'shell_execute'
  | 'file_read'
  | 'file_write'
  | 'file_edit'
  | 'file_delete'
  | 'file_glob'
  | 'file_grep'

export interface ToolResult {
  success: boolean
  data?: string
  error?: {
    code: string
    message: string
  }
}

export interface ToolDefinition {
  name: string
  description: string
  parameters: ToolParameter[]
  executor: ExecutorName
}

export const OUT_OF_WORKDIR_PREFIX = 'OUT_OF_WORKDIR:'

const COREFILE_PATTERN = /corefile[/\\](SOUL|HABIT|SYSPROMPT|ABOUTUSER|REBIRTH)\.md$/i

function isCorefileWrite(executor: ExecutorName, input: Record<string, unknown>): boolean {
  if (executor !== 'file_write' && executor !== 'file_edit' && executor !== 'file_delete') return false
  const path = input.path as string
  if (!path) return false
  return COREFILE_PATTERN.test(path)
}

export function isOutOfWorkdirError(errorMsg: string): boolean {
  return errorMsg.startsWith(OUT_OF_WORKDIR_PREFIX)
}

export function extractOutOfWorkdirPath(errorMsg: string): string {
  return errorMsg.slice(OUT_OF_WORKDIR_PREFIX.length)
}

export function defineTool(config: ToolDefinition): ToolDefinition {
  return config
}

export interface ExecutorParamDef {
  name: string
  type: 'string' | 'number' | 'integer' | 'boolean'
  required: boolean
  description: string
}

export type ExecutorParamDefs = Record<ExecutorName, ExecutorParamDef[]>

export const EXECUTOR_PARAM_DEFS: ExecutorParamDefs = {
  shell_execute: [
    { name: 'command', type: 'string', required: true, description: '要执行的命令' },
    { name: 'timeout', type: 'number', required: true, description: '超时时间(秒)' },
  ],
  file_read: [
    { name: 'path', type: 'string', required: true, description: '文件路径' },
    { name: 'offset', type: 'integer', required: false, description: '起始行号（从1开始）' },
    { name: 'limit', type: 'integer', required: false, description: '最大行数' },
  ],
  file_write: [
    { name: 'path', type: 'string', required: true, description: '文件路径' },
    { name: 'content', type: 'string', required: true, description: '文件内容' },
  ],
  file_edit: [
    { name: 'path', type: 'string', required: true, description: '文件路径' },
    { name: 'oldStr', type: 'string', required: true, description: '要搜索的原始内容' },
    { name: 'newStr', type: 'string', required: true, description: '替换后的新内容' },
  ],
  file_delete: [
    { name: 'path', type: 'string', required: true, description: '文件路径' },
  ],
  file_glob: [
    { name: 'pattern', type: 'string', required: true, description: '搜索模式' },
  ],
  file_grep: [
    { name: 'path', type: 'string', required: true, description: '搜索路径' },
    { name: 'pattern', type: 'string', required: true, description: '搜索模式' },
  ],
}

export const BUILTIN_SECURITY_RULES: SecurityRule[] = [
  { id: 'builtin-rm-rf', pattern: 'rm -rf /', category: 'shell', description: '删除整个根目录', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-mkfs', pattern: 'mkfs', category: 'shell', description: '格式化文件系统', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-fork-bomb', pattern: ':(){ :|:& };:', category: 'shell', description: 'Fork 炸弹', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-chmod-777', pattern: 'chmod -R 777 /', category: 'shell', description: '全局提权', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-dd-zero', pattern: 'dd if=/dev/zero', category: 'shell', description: '磁盘覆写', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-dev-redirect', pattern: '> /dev/', category: 'shell', description: '重定向到设备', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-pipe-sh', pattern: '| sh', category: 'shell', description: '管道到 sh', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-pipe-bash', pattern: '| bash', category: 'shell', description: '管道到 bash', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-pipe-zsh', pattern: '| zsh', category: 'shell', description: '管道到 zsh', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-pipe-powershell', pattern: '| powershell', category: 'shell', description: '管道到 PowerShell', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-semi-rm', pattern: '; rm ', category: 'shell', description: '分号拼接删除', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-and-rm', pattern: '&& rm ', category: 'shell', description: '逻辑与拼接删除', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
  { id: 'builtin-or-rm', pattern: '|| rm ', category: 'shell', description: '逻辑或拼接删除', enabled: true, isBuiltIn: true, createdAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z' },
]

export interface SecurityConfig {
  workingDir: string
  timeout: number
  shellTimeout?: number
  executorTimeouts?: Partial<Record<ExecutorName, number>>
  rules: SecurityRule[]
}

const defaultSecurityConfig: SecurityConfig = {
  // 工作目录由后端统一管理，此字段仅作前端展示/备用
  workingDir: '',
  timeout: 30000,
  shellTimeout: 60000,
  rules: [...BUILTIN_SECURITY_RULES],
}

function toolToDefinition(tool: Tool): ToolDefinition {
  return {
    name: tool.invokeName,
    description: tool.description,
    parameters: tool.parameters,
    executor: tool.executor
  }
}

class ToolTimeoutError extends Error {
  readonly code = 'TIMEOUT'
  constructor(timeoutMs: number) {
    super(`工具执行超时 (${Math.round(timeoutMs / 1000)}秒)`)
    this.name = 'ToolTimeoutError'
  }
}

class ToolAbortError extends Error {
  readonly code = 'ABORTED'
  constructor() {
    super('工具执行被用户取消')
    this.name = 'ToolAbortError'
  }
}

function normalizePathSegments(path: string): string[] {
  const sep = path.includes('/') ? '/' : '\\'
  const segments = path.split(sep)
  const normalized: string[] = []
  for (const seg of segments) {
    if (seg === '.' || seg === '') continue
    if (seg === '..') {
      if (normalized.length > 0 && normalized[normalized.length - 1] !== '..') {
        normalized.pop()
      } else {
        normalized.push('..')
      }
    } else {
      normalized.push(seg)
    }
  }
  return normalized
}

export class FairyDo {
  private tools: Map<string, ToolDefinition> = new Map()
  private securityConfig: SecurityConfig = { ...defaultSecurityConfig }
  private virtualHandlers: Map<string, (input: Record<string, unknown>, signal?: AbortSignal) => Promise<string>> = new Map()

  constructor() {
  }

  registerVirtualHandler(name: string, handler: (input: Record<string, unknown>, signal?: AbortSignal) => Promise<string>): void {
    this.virtualHandlers.set(name, handler)
  }

  isVirtualTool(toolName: string): boolean {
    return this.virtualHandlers.has(toolName)
  }

  register(tool: ToolDefinition): void {
    this.tools.set(tool.name, tool)
  }

  registerAll(tools: (ToolDefinition | Tool)[]): void {
    tools.forEach(tool => {
      const def = 'invokeName' in tool ? toolToDefinition(tool as Tool) : tool as ToolDefinition
      this.register(def)
    })
  }

  unregister(toolName: string): boolean {
    return this.tools.delete(toolName)
  }

  hasTool(toolName: string): boolean {
    return this.tools.has(toolName)
  }

  getTool(toolName: string): ToolDefinition | undefined {
    return this.tools.get(toolName)
  }

  listTools(): ToolDefinition[] {
    return Array.from(this.tools.values())
  }

  setSecurityRules(rules: SecurityRule[]): void {
    this.securityConfig.rules = rules
  }

  private validateInput(invokeName: string, input: Record<string, unknown>): { valid: boolean; error?: string } {
    if (invokeName.startsWith('shell_')) return this.validateShellInput(input)
    if (invokeName.startsWith('file_')) return this.validateFileInput(input)
    return { valid: true }
  }

  private validateShellInput(input: Record<string, unknown>): { valid: boolean; error?: string } {
    const command = input.command as string
    if (!command) return { valid: false, error: '缺少 command 参数' }
    if (command.length > 4096) return { valid: false, error: '命令长度超过限制 (最大 4096 字符)' }
    const shellRules = this.securityConfig.rules.filter(r => r.category === 'shell' && r.enabled)
    for (const rule of shellRules) {
      if (command.toLowerCase().includes(rule.pattern.toLowerCase())) {
        return { valid: false, error: `命令包含危险模式: ${rule.pattern}` }
      }
    }
    return { valid: true }
  }

  private validateFileInput(input: Record<string, unknown>): { valid: boolean; error?: string } {
    const path = input.path as string
    if (!path) return { valid: true }
    const segments = normalizePathSegments(path)
    if (segments.includes('..')) return { valid: false, error: '禁止路径穿越' }
    return { valid: true }
  }

  private fillDefaults(parameters: ToolParameter[], input: Record<string, unknown>): Record<string, unknown> {
    const invokeInput: Record<string, unknown> = { ...input }
    for (const param of parameters) {
      if (param.default !== undefined && !(param.name in invokeInput)) {
        invokeInput[param.name] = param.default
      }
    }
    return invokeInput
  }

  private serializeResult(result: unknown): string {
    if (result === null || result === undefined) return '操作完成'
    if (typeof result === 'string') return result
    return JSON.stringify(result, null, 2)
  }

  private getTimeout(executor: ExecutorName): number {
    if (this.securityConfig.executorTimeouts?.[executor] !== undefined) {
      return this.securityConfig.executorTimeouts[executor]!
    }
    if (executor.startsWith('shell_') && this.securityConfig.shellTimeout !== undefined) {
      return this.securityConfig.shellTimeout
    }
    return this.securityConfig.timeout
  }

  private async invokeBackend(
    executor: ExecutorName,
    input: Record<string, unknown>,
    signal?: AbortSignal,
    extraAllowedPaths?: string[]
  ): Promise<ToolResult> {
    const validation = this.validateInput(executor, input)
    if (!validation.valid) {
      return { success: false, error: { code: 'VALIDATION_FAILED', message: validation.error! } }
    }

    if (isCorefileWrite(executor, input)) {
      return { success: false, error: { code: 'COREFILE_PROTECTED', message: '核心文件受保护，禁止通过工具修改。请使用角色设置页面管理核心文件。' } }
    }

    const paramDefs = EXECUTOR_PARAM_DEFS[executor]
    const invokeInput = paramDefs ? this.fillDefaults(paramDefs, input) : { ...input }

    if (executor.startsWith('file_') && extraAllowedPaths && extraAllowedPaths.length > 0) {
      invokeInput['extraAllowedPaths'] = extraAllowedPaths
    }

    const timeoutMs = this.getTimeout(executor)

    if (executor === 'shell_execute' && !('timeout' in invokeInput)) {
      invokeInput['timeout'] = Math.round(timeoutMs / 1000)
    }

    const timeoutPromise = new Promise<never>((_, reject) =>
      setTimeout(() => reject(new ToolTimeoutError(timeoutMs)), timeoutMs)
    )

    const racers: Promise<unknown>[] = [
      invoke<unknown>(executor, invokeInput),
      timeoutPromise,
    ]

    if (signal) {
      racers.push(new Promise<never>((_, reject) => {
        if (signal.aborted) {
          reject(new ToolAbortError())
          return
        }
        const onAbort = () => reject(new ToolAbortError())
        signal.addEventListener('abort', onAbort, { once: true })
      }))
    }

    try {
      const result = await Promise.race(racers)
      return { success: true, data: this.serializeResult(result) }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      if (error instanceof ToolTimeoutError) {
        return { success: false, error: { code: 'TIMEOUT', message } }
      }
      if (error instanceof ToolAbortError) {
        return { success: false, error: { code: 'ABORTED', message } }
      }
      return { success: false, error: { code: 'EXECUTION_ERROR', message } }
    }
  }

  async execute(toolName: string, input: Record<string, unknown>, signal?: AbortSignal, extraAllowedPaths?: string[]): Promise<ToolResult> {
    if (isAggregateTool(toolName)) {
      const resolved = resolveAction(toolName, input)
      if (!resolved) {
        return { success: false, error: { code: 'INVALID_ACTION', message: `未知的 action: ${input.action}` } }
      }
      const { executor, remainingInput } = resolved

      const virtualHandler = this.virtualHandlers.get(executor)
      if (virtualHandler) {
        try {
          const data = await virtualHandler(remainingInput, signal)
          return { success: true, data }
        } catch (error) {
          const message = error instanceof Error ? error.message : String(error)
          return { success: false, error: { code: 'VIRTUAL_TOOL_ERROR', message } }
        }
      }

      return this.invokeBackend(executor as ExecutorName, remainingInput, signal, extraAllowedPaths)
    }

    const virtualHandler = this.virtualHandlers.get(toolName)
    if (virtualHandler) {
      try {
        const data = await virtualHandler(input, signal)
        return { success: true, data }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        return { success: false, error: { code: 'VIRTUAL_TOOL_ERROR', message } }
      }
    }

    const tool = this.tools.get(toolName)
    if (!tool) {
      return { success: false, error: { code: 'TOOL_NOT_FOUND', message: `工具 "${toolName}" 不存在` } }
    }

    return this.invokeBackend(tool.executor, input, signal, extraAllowedPaths)
  }
}

export const fairyDo = new FairyDo()
