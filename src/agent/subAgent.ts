import type { Tool } from '../types/tool'
import type { ChatMessage } from '../stores/chat'
import type { ToolUseBlock } from '../types/message'
import { executeReActLoop, type ReActResult } from './reactLoop'

export interface SubAgentConfig {
  providerId: string
  description: string
  tools: Tool[]
  maxIterations?: number
  timeout?: number
  signal?: AbortSignal
  onToolExecuting?: (toolCallId: string, tool: string, input: Record<string, unknown>) => void
  onToolResult?: (tool: string, result: { success: boolean; data?: string; error?: { code: string; message: string } }) => void
}

export interface SubAgentResult {
  success: boolean
  summary: string
  toolCalls: ToolUseBlock[]
  iterations: number
  error?: string
}

export async function executeSubAgent(config: SubAgentConfig): Promise<SubAgentResult> {
  const {
    providerId,
    description,
    tools,
    maxIterations = 20,
    timeout = 300000,
    signal,
    onToolExecuting,
    onToolResult
  } = config

  let timeoutHandle: ReturnType<typeof setTimeout> | null = null
  let abortedByTimeout = false

  const effectiveSignal = signal
    ? combineWithTimeout(signal, timeout)
    : createTimeoutSignal(timeout)

  if (effectiveSignal) {
    effectiveSignal.addEventListener('abort', () => {
      if (!signal?.aborted) {
        abortedByTimeout = true
      }
    }, { once: true })
  }

  const systemMessage: ChatMessage = {
    role: 'system',
    content: '你是一个子任务执行助手。你负责完成主 Agent 分配给你的具体任务。请专注于当前任务，使用可用的工具完成它，然后返回结果摘要。不要尝试创建新的任务列表或委派子任务。'
  }

  const userMessage: ChatMessage = {
    role: 'user',
    content: description
  }

  const initialMessages: ChatMessage[] = [systemMessage, userMessage]

  try {
    const result: ReActResult = await executeReActLoop(providerId, initialMessages, {
      tools,
      maxIterations,
      signal: effectiveSignal,
      onToolExecuting: onToolExecuting,
      onToolResult: onToolResult
    })

    return {
      success: true,
      summary: result.allContent || result.content || '子任务完成（无输出）',
      toolCalls: result.toolCalls,
      iterations: result.iterations
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    if (abortedByTimeout) {
      return {
        success: false,
        summary: '',
        toolCalls: [],
        iterations: 0,
        error: `子任务执行超时 (${Math.round(timeout / 1000)}秒)`
      }
    }
    if (signal?.aborted) {
      return {
        success: false,
        summary: '',
        toolCalls: [],
        iterations: 0,
        error: '子任务被用户取消'
      }
    }
    return {
      success: false,
      summary: '',
      toolCalls: [],
      iterations: 0,
      error: `子任务执行失败: ${message}`
    }
  } finally {
    if (timeoutHandle) {
      clearTimeout(timeoutHandle)
    }
  }
}

function combineWithTimeout(signal: AbortSignal, timeout: number): AbortSignal {
  const controller = new AbortController()
  const handle = setTimeout(() => controller.abort(), timeout)
  signal.addEventListener('abort', () => {
    clearTimeout(handle)
    controller.abort()
  }, { once: true })
  return controller.signal
}

function createTimeoutSignal(timeout: number): AbortSignal {
  const controller = new AbortController()
  setTimeout(() => controller.abort(), timeout)
  return controller.signal
}
