import type { Tool } from '../types/tool'
import type { ChatMessage } from '../stores/chat'
import type { ToolUseBlock } from '../types/message'
import type { TokenUsage } from '../stores/conversation'
import { sendChatMessage } from '../stores/chat'
import { fairyDo, type ToolResult, isOutOfWorkdirError, extractOutOfWorkdirPath } from './fairyDo'
import { addDebugLog } from '../stores/debugStore'
import { estimateMessagesTokens } from './tokenEstimator'

export interface ToolCallMessage {
  role: 'assistant'
  content: string
  reasoning_content?: string
  tool_calls: Array<{
    id: string
    type: 'function'
    function: {
      name: string
      arguments: string
    }
  }>
}

export interface ToolResultMessage {
  role: 'tool'
  tool_call_id: string
  name: string
  content: string
}

export interface ReActConfig {
  maxIterations?: number
  tools: Tool[]
  extraAllowedPaths?: string[]
  workingDirOverride?: string
  onTurnStart?: (turnNumber: number) => void
  onChunk?: (chunk: string) => void
  onReasoningChunk?: (reasoning: string) => void
  onToolExecuting?: (toolCallId: string, tool: string, input: Record<string, unknown>) => void
  onToolResult?: (tool: string, result: ToolResult) => void
  onUsage?: (usage: TokenUsage) => void
  onTurnEnd?: () => void
  onApproveAccess?: (toolName: string, targetPath: string) => Promise<boolean>
  signal?: AbortSignal
}

export interface ReActResult {
  content: string
  allContent: string
  toolCalls: ToolUseBlock[]
  iterations: number
  toolResults: Array<{
    toolName: string
    input: Record<string, unknown>
    result: ToolResult
  }>
}

function formatToolCallsMessage(
  toolCalls: ToolUseBlock[],
  reasoningContent?: string
): ToolCallMessage {
  return {
    role: 'assistant',
    content: '',
    ...(reasoningContent ? { reasoning_content: reasoningContent } : {}),
    tool_calls: toolCalls.map(tc => ({
      id: tc.id,
      type: 'function',
      function: {
        name: tc.name,
        arguments: JSON.stringify(tc.input)
      }
    }))
  }
}

const MAX_TOOL_RESULT_LENGTH = 5000
const MAX_TOOL_HISTORY_TOKENS = 20000
const AGGRESSIVE_TRUNCATE_LENGTH = 1000

function truncateToolResult(data: string): string {
  if (data.length <= MAX_TOOL_RESULT_LENGTH) {
    return data
  }

  if (data.startsWith('data:image/') || (data.length > 10000 && /^[A-Za-z0-9+/=]+$/.test(data.slice(0, 100)))) {
    return `[截图已成功捕获，图片大小: ${(data.length * 3 / 4 / 1024).toFixed(1)}KB，已省略 base64 数据以节省 token]`
  }

  return data.slice(0, MAX_TOOL_RESULT_LENGTH) + `\n...[结果已截断，原始长度: ${data.length} 字符]`
}

function reTruncateToolResult(content: string): string {
  if (content.length <= AGGRESSIVE_TRUNCATE_LENGTH) return content
  if (content.startsWith('[截图已成功捕获')) return content
  return content.slice(0, AGGRESSIVE_TRUNCATE_LENGTH) + `\n...[结果已二次截断]`
}

function formatToolResultMessage(
  toolCallId: string,
  toolName: string,
  result: ToolResult
): ToolResultMessage {
  let content: string

  if (result.success) {
    const rawData = result.data || '执行成功'
    content = truncateToolResult(rawData)
  } else {
    content = `错误: ${result.error?.code || 'UNKNOWN'} - ${result.error?.message || '未知错误'}`
  }

  return {
    role: 'tool',
    tool_call_id: toolCallId,
    name: toolName,
    content
  }
}

export async function executeReActLoop(
  providerId: string,
  initialMessages: ChatMessage[],
  config: ReActConfig
): Promise<ReActResult> {
  const maxIterations = config.maxIterations || Infinity
  const messages = [...initialMessages]

  let allContent = ''
  const allToolCalls: ToolUseBlock[] = []
  const toolResults: ReActResult['toolResults'] = []
  let iterationsExecuted = 0
  let accumulatedUsage: TokenUsage = { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 }
  const approvedPaths = new Set<string>()
  if (config.extraAllowedPaths) {
    for (const p of config.extraAllowedPaths) {
      approvedPaths.add(p)
    }
  }
  let emptyRetries = 0
  const MAX_EMPTY_RETRIES = 2

  while (true) {
    if (config.signal?.aborted) {
      console.log('[ReActLoop] 已中止')
      break
    }

    if (iterationsExecuted >= maxIterations) {
      console.log('[ReActLoop] 达到最大迭代次数限制')
      break
    }

    iterationsExecuted++
    console.log(`[ReActLoop] 第 ${iterationsExecuted} 轮迭代`)
    console.log(`[ReActLoop] 当前 messages 数量: ${messages.length}`)
    if (iterationsExecuted > 1) {
      const lastTwo = messages.slice(-2)
      console.log(`[ReActLoop] 最后两条消息:`, JSON.stringify(lastTwo, null, 2)?.slice(0, 500))
    }

    config.onTurnStart?.(iterationsExecuted)

    const turnToolCalls: ToolUseBlock[] = []
    let turnReasoning = ''

    const toolMessages = messages.filter(m => m.role === 'tool')
    if (toolMessages.length > 4) {
      const toolTokens = estimateMessagesTokens(
        messages.filter(m => m.role === 'tool' || (m.role === 'assistant' && ('tool_calls' in m)))
          .map(m => ({ role: m.role, content: typeof m.content === 'string' ? m.content : '' }))
      )
      if (toolTokens > MAX_TOOL_HISTORY_TOKENS) {
        console.log(`[ReActLoop] 工具历史 token ${toolTokens} 超过阈值 ${MAX_TOOL_HISTORY_TOKENS}，执行二次截断`)
        for (let i = 0; i < messages.length; i++) {
          if (messages[i].role === 'tool' && i < messages.length - 4) {
            const msg = messages[i]
            if (typeof msg.content === 'string') {
              msg.content = reTruncateToolResult(msg.content)
            }
          }
        }
      }
    }

    const response = await sendChatMessage({
      providerId,
      messages,
      tools: config.tools,
      signal: config.signal,
      onChunk: (chunk) => {
        allContent += chunk
        config.onChunk?.(chunk)
      },
      onReasoningChunk: (reasoning) => {
        turnReasoning += reasoning
        config.onReasoningChunk?.(reasoning)
      },
      onToolCall: (toolCall) => {
        allToolCalls.push(toolCall)
        turnToolCalls.push(toolCall)
        console.log('[ReActLoop] 收集到工具调用:', toolCall.name, toolCall.input)
      }
    })

    if (response.usage) {
      accumulatedUsage.prompt_tokens += response.usage.prompt_tokens
      accumulatedUsage.completion_tokens += response.usage.completion_tokens
      accumulatedUsage.total_tokens += response.usage.total_tokens
      config.onUsage?.({ ...accumulatedUsage })
    }

    console.log(`[ReActLoop] 模型响应完成, 本轮工具调用: ${turnToolCalls.length}, 累计: ${allToolCalls.length}`)
    console.log(`[ReActLoop] response.content 长度: ${response.content?.length || 0}, response.toolCalls 数量: ${response.toolCalls?.length || 0}`)
    if (response.content) {
      console.log(`[ReActLoop] response.content 预览: ${response.content.slice(0, 200)}`)
    }

    if (!response.toolCalls || response.toolCalls.length === 0) {
      if (response.content && response.content.trim().length > 0) {
        console.log('[ReActLoop] 模型不再调用工具，结束循环')
        console.log('[ReActLoop] 模型最终响应内容:', response.content.slice(0, 200))
        break
      }

      const hasToolHistory = messages.some(m => m.role === 'tool')
      if (hasToolHistory && emptyRetries < MAX_EMPTY_RETRIES) {
        emptyRetries++
        console.log(`[ReActLoop] 模型返回空内容但有工具历史，注入提示重试 (${emptyRetries}/${MAX_EMPTY_RETRIES})`)
        messages.push({
          role: 'user' as const,
          content: '[系统提示] 你刚刚收到了工具的执行结果，请根据结果继续回复用户。如果你需要调用更多工具，请使用工具调用功能。不要在思考中输出工具调用，请使用结构化的 tool_calls。'
        })
        continue
      }

      console.log('[ReActLoop] 模型不再调用工具，结束循环')
      break
    }

    console.log(`[ReActLoop] 检测到 ${response.toolCalls.length} 个工具调用`)
    console.log('[ReActLoop] 工具调用详情:', JSON.stringify(response.toolCalls, null, 2))

    messages.push(formatToolCallsMessage(response.toolCalls, turnReasoning || undefined) as unknown as ChatMessage)

    for (const toolCall of response.toolCalls) {
      if (config.signal?.aborted) break

      config.onToolExecuting?.(toolCall.id, toolCall.name, toolCall.input)

      addDebugLog('request', `Tool Call: ${toolCall.name}`, JSON.stringify(toolCall.input, null, 2), {
        toolCallId: toolCall.id,
        toolName: toolCall.name
      })

      const extraAllowedPaths = approvedPaths.size > 0 ? Array.from(approvedPaths) : undefined
      const result = await fairyDo.execute(toolCall.name, toolCall.input, config.signal, extraAllowedPaths, config.workingDirOverride)

      if (config.signal?.aborted) {
        console.log('[ReActLoop] 工具执行后被中止，跳出循环')
        break
      }

      if (!result.success && result.error?.message && isOutOfWorkdirError(result.error.message)) {
        const targetPath = extractOutOfWorkdirPath(result.error.message)
        addDebugLog('error', `路径越界: ${toolCall.name}`, `尝试访问: ${targetPath}`, {
          toolName: toolCall.name,
          targetPath
        })

        if (config.onApproveAccess && !approvedPaths.has(targetPath)) {
          const approved = await config.onApproveAccess(toolCall.name, targetPath)
          if (config.signal?.aborted) break
          if (approved) {
            approvedPaths.add(targetPath)
            addDebugLog('info', '用户授权访问', targetPath, { toolName: toolCall.name })
            const retryResult = await fairyDo.execute(toolCall.name, toolCall.input, config.signal, Array.from(approvedPaths), config.workingDirOverride)
            console.log(`[ReActLoop] 重试工具 ${toolCall.name} 结果:`, JSON.stringify(retryResult, null, 2))
            toolResults.push({
              toolName: toolCall.name,
              input: toolCall.input,
              result: retryResult
            })
            config.onToolResult?.(toolCall.name, retryResult)
            messages.push(formatToolResultMessage(toolCall.id, toolCall.name, retryResult) as unknown as ChatMessage)
            continue
          }
        }

        toolResults.push({
          toolName: toolCall.name,
          input: toolCall.input,
          result
        })
        config.onToolResult?.(toolCall.name, result)
        messages.push(formatToolResultMessage(toolCall.id, toolCall.name, result) as unknown as ChatMessage)
        continue
      }

      addDebugLog('response', `Tool Result: ${toolCall.name}`, result.success
        ? (result.data || '执行成功')
        : `错误: ${result.error?.code || 'UNKNOWN'} - ${result.error?.message || '未知错误'}`,
        {
          toolName: toolCall.name,
          success: result.success,
          resultLength: (result.data || '').length
        }
      )

      toolResults.push({
        toolName: toolCall.name,
        input: toolCall.input,
        result
      })

      config.onToolResult?.(toolCall.name, result)

      messages.push(formatToolResultMessage(toolCall.id, toolCall.name, result) as unknown as ChatMessage)
    }
  }

  config.onTurnEnd?.()

  return {
    content: allContent,
    allContent,
    toolCalls: allToolCalls,
    iterations: iterationsExecuted,
    toolResults
  }
}
