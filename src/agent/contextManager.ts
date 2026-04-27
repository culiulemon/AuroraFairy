import { estimateTokens, estimateMessagesTokens } from './tokenEstimator'
import { generateSummary } from './conversationSummarizer'
import { loadSettings } from '../stores/settings'

export interface ContextMessage {
  role: 'user' | 'assistant'
  content: string
  reasoning_content?: string
}

export interface ContextBuildOptions {
  systemPrompt: string
  memoryContext: string
  messages: ContextMessage[]
  conversationSummary?: string
  conversationSummaryUpdatedAt?: string
  conversationId: string
  providerId: string
}

export interface ContextBuildResult {
  messages: Array<{ role: 'system' | 'user' | 'assistant'; content: string; reasoning_content?: string }>
  needsSummaryUpdate: boolean
  oldMessagesForSummary: Array<{ role: string; content: string }>
  compressedCount: number
}

const DEFAULT_CONTEXT_WINDOW = 128000
const DEFAULT_RECENT_ROUNDS = 10
const OUTPUT_TOKEN_RESERVE = 4096

function truncateSummaryToFit(summary: string, maxTokens: number): string {
  if (estimateTokens(summary) <= maxTokens) {
    return summary
  }
  const charBudget = maxTokens * 2
  return summary.slice(0, charBudget) + '\n...(摘要已截断)'
}

export async function triggerSummaryGeneration(
  conversationId: string,
  oldMessages: Array<{ role: string; content: string }>,
  existingSummary: string | undefined,
  providerId: string,
  onSaveSummary: (convId: string, summary: string) => void,
): Promise<void> {
  try {
    const result = await generateSummary(oldMessages, existingSummary, providerId)
    if (result) {
      onSaveSummary(conversationId, result)
    }
  } catch (e) {
    console.error('[ContextManager] 摘要生成失败:', e)
  }
}

export async function buildContextMessages(opts: ContextBuildOptions): Promise<ContextBuildResult> {
  const settings = loadSettings()
  const provider = settings.providers.find(p => p.id === opts.providerId)
    || settings.providers.find(p => p.id === settings.defaultProviderId)
    || settings.providers[0]

  const contextWindowSize = provider?.contextWindowSize || DEFAULT_CONTEXT_WINDOW
  const recentRounds = settings.contextRecentRounds || DEFAULT_RECENT_ROUNDS

  const budget = contextWindowSize - OUTPUT_TOKEN_RESERVE

  const systemContent = opts.systemPrompt + (opts.memoryContext ? '\n\n' + opts.memoryContext : '')
  const systemMsg: { role: 'system'; content: string } = {
    role: 'system',
    content: systemContent,
  }
  const systemTokens = estimateTokens(systemContent)
  const historyTokens = estimateMessagesTokens(opts.messages)
  const total = systemTokens + historyTokens

  if (total <= budget) {
    return {
      messages: [systemMsg, ...opts.messages],
      needsSummaryUpdate: false,
      oldMessagesForSummary: [],
      compressedCount: 0,
    }
  }

  const recentMessages: ContextMessage[] = []
  let roundCount = 0
  for (let i = opts.messages.length - 1; i >= 0; i--) {
    if (opts.messages[i].role === 'user') roundCount++
    recentMessages.unshift(opts.messages[i])
    if (roundCount >= recentRounds) break
  }
  const oldMessages = opts.messages.slice(0, opts.messages.length - recentMessages.length)

  const recentTokens = estimateMessagesTokens(recentMessages)
  const availableForSummary = budget - systemTokens - recentTokens

  if (opts.conversationSummary && availableForSummary > 0) {
    const summaryContent = truncateSummaryToFit(
      '## 之前的对话摘要\n\n' + opts.conversationSummary,
      availableForSummary,
    )
    const summaryMsg: { role: 'system'; content: string } = {
      role: 'system',
      content: summaryContent,
    }

    return {
      messages: [systemMsg, summaryMsg, ...recentMessages],
      needsSummaryUpdate: oldMessages.length > 0,
      oldMessagesForSummary: oldMessages,
      compressedCount: oldMessages.length,
    }
  }

  if (oldMessages.length > 0) {
    triggerSummaryGeneration(
      opts.conversationId,
      oldMessages,
      opts.conversationSummary,
      opts.providerId,
      () => {},
    ).catch(() => {})
  }

  return {
    messages: [systemMsg, ...recentMessages],
    needsSummaryUpdate: true,
    oldMessagesForSummary: oldMessages,
    compressedCount: oldMessages.length,
  }
}
