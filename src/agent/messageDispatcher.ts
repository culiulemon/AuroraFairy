import { executeReActLoop } from './reactLoop'
import { assembleSystemPrompt } from './systemPromptAssembler'
import { sysTools, memorySearchTool, roleConfigTool } from './FairySysTools'
import { loadAllTools } from '../stores/toolsStore'
import { loadSettings } from '../stores/settings'
import type { Tool } from '../types/tool'
import { buildContextMessages, triggerSummaryGeneration } from './contextManager'
import { skillManager } from './skills/skillManager'
import { buildSkillsPrompt } from './skills/skillInjector'
import { evolveSkillMetadata } from './skills/skillEvolver'
import { getLLMAdapter } from '../stores/fbmStore'
import { memoryReorganizer } from '../stores/memoryReorganizer'

let lastMatchedSkills: import('./skills/types').SkillIndexEntry[] = []
let lastMatchedConvId: string | null = null

const SHORT_RESPONSE_MAX_LEN = 15
const SHORT_RESPONSE_PATTERN = /^(好的|好|嗯|是|对|谢谢|感谢|收到|明白|了解|ok|yes|no|不是|没问题|继续|可以|行|这样|那)/i

export function clearSkillCache(): void {
  lastMatchedSkills = []
  lastMatchedConvId = null
}

async function runBackgroundTasks(
  deps: DispatcherDeps,
  callbacks: DispatcherCallbacks,
  params: {
    convId: string
    providerId: string
    content: string
    activatedSkills: import('./skills/types').SkillIndexEntry[]
    contextResult: { needsSummaryUpdate: boolean; oldMessagesForSummary: Array<{ role: string; content: string }> }
    settings: ReturnType<typeof loadSettings>
    recentMessages: Array<{ role: string; content: string }>
  }
): Promise<void> {
  const { convId, providerId, content, activatedSkills, contextResult, settings: bgSettings, recentMessages } = params

  if (deps.fbmStore.isEnabled() && bgSettings.fbmConsolidationEnabled !== false) {
    const currentConv = deps.getActiveConversation()
    const convMessages = currentConv?.messages
      .filter(m => !m.isLoading)
      .map(m => ({
        role: m.role as 'user' | 'assistant',
        content: m.content.filter(c => c.type === 'text' && c.text).map(c => c.text || '').join(''),
        timestamp: m.timestamp ? new Date(m.timestamp).getTime() : Date.now(),
      })) || []
    if (convMessages.length > 0) {
      const convIdBg = currentConv?.id
      deps.fbmStore.consolidate(convMessages, convIdBg).then((result: unknown) => {
        console.log('[Dispatcher] Consolidation result:', result)
        if (result && convIdBg) {
          callbacks.onConsolidationComplete(convIdBg)
        }
      }).catch((err: unknown) => {
        console.warn('[Dispatcher] Background consolidation failed:', err)
      })
    }
  }

  if (contextResult.needsSummaryUpdate && contextResult.oldMessagesForSummary.length > 0) {
    const currentConv = deps.getActiveConversation()
    triggerSummaryGeneration(
      convId,
      contextResult.oldMessagesForSummary,
      currentConv?.summary,
      providerId,
      deps.saveConversationSummary,
    ).catch((err) => {
      console.warn('[Dispatcher] Background summary generation failed:', err)
    })
  }

  if (activatedSkills.length > 0) {
    try {
      const skillLlm = await getLLMAdapter()
      if (skillLlm) {
        const currentConv = deps.getActiveConversation()
        const lastAssistantMsg = currentConv?.messages
          .filter(m => !m.isLoading && m.role === 'assistant')
          .map(m => m.content.filter(c => c.type === 'text' && c.text).map(c => c.text || '').join(''))
          .pop() || ''
        for (const skill of activatedSkills) {
          if (!skillManager.shouldEvolve(skill.name)) continue
          const evolveResult = await evolveSkillMetadata(
            skill.description,
            skill.tags,
            skill.argumentHint,
            content,
            lastAssistantMsg,
            skillLlm,
            recentMessages
          )
          if (evolveResult.needsUpdate) {
            await skillManager.updateSkillMetadata(skill.name, {
              description: evolveResult.newDescription || undefined,
              tags: evolveResult.newTags || undefined,
              argumentHint: evolveResult.newArgumentHint || undefined,
            })
          }
          skillManager.markEvolved(skill.name)
        }
      }
    } catch (err) {
      console.warn('[Dispatcher] Skill description evolution failed:', err)
    }
  }
}

export interface DispatcherCallbacks {
  onChunk: (chunk: string) => void
  onReasoningChunk: (reasoning: string) => void
  onToolExecuting: (toolCallId: string, tool: string, input: Record<string, unknown>) => void
  onToolResult: (tool: string, result: unknown) => void
  onTurnStart: () => void
  onTurnEnd: (messageId: string) => void
  onApproveAccess: (toolName: string, targetPath: string) => Promise<boolean>
  onMemoryRecallStart: () => void
  onMemoryRecallComplete: (summary: string, keywords: string[]) => void
  onMemoryRecallError: () => void
  onConsolidationComplete: (convId: string) => void
  onContextCompressed: (convId: string, compressedCount: number) => void
}

export interface DispatcherDeps {
  getActiveConversation: () => {
    id: string
    title: string
    messages: Array<{
      id: string
      role: string
      content: Array<{ type: string; text?: string }>
      isLoading?: boolean
      isGreeting?: boolean
      timestamp?: string | number
    }>
    summary?: string
    summaryUpdatedAt?: string
  } | null
  addMessage: (convId: string, role: 'user' | 'assistant', content: Array<{ type: string; text: string }>) => void
  getConversationMessages: (convId: string) => Array<{ role: 'user' | 'assistant'; content: string }>
  generateTitleIfNeeded: (providerId: string, userMessage: string, convId: string, currentTitle: string) => void
  fbmStore: {
    isEnabled(): boolean
    ensureInit(): Promise<void>
    retrieve(query: string | string[], context?: string): Promise<any>
    getLastRetrieveKeywords(): string[]
    consolidate(
      messages: Array<{ role: string; content: string; timestamp?: number }>,
      conversationId?: string,
    ): Promise<{ created: number; updated: number; deleted: number; skipped: number } | null>
  }
  fairyDo: {
    registerAll(tools: (import('../types/tool').Tool | { name: string; description: string; parameters: import('../types/tool').ToolParameter[]; executor: import('./fairyDo').ExecutorName })[]): void
  }
  setCurrentUserMessage: (message: string) => void
  setRecentUserMessages: (messages: string[]) => void
  setConversationContext: (context: string) => void
  setCurrentTools: (tools: Tool[]) => void
  setCurrentProviderId: (id: string) => void
  saveConversationSummary: (convId: string, summary: string) => void
}

export async function dispatchMessage(
  content: string,
  deps: DispatcherDeps,
  callbacks: DispatcherCallbacks,
  signal?: AbortSignal
): Promise<{ content: string; iterations: number }> {
  memoryReorganizer.onConversationSent()

  const conv = deps.getActiveConversation()
  if (!conv) throw new Error('No active conversation')

  const settings = loadSettings()
  const provider = settings.providers.find(p => p.id === settings.defaultProviderId)
    || settings.providers[0]

  if (!provider) throw new Error('No provider configured')

  deps.generateTitleIfNeeded(provider.id, content, conv.id, conv.title)

  const messages = conv.messages
    .filter(m => !m.isLoading && !m.isGreeting)
    .map(m => ({
      role: m.role as 'user' | 'assistant',
      content: m.content.filter(c => c.type === 'text' && c.text).map(c => c.text || '').join('')
    }))

  const recentMessages = messages.slice(-6)
  const conversationContext = recentMessages
    .map(m => `${m.role === 'user' ? '用户' : '助手'}: ${m.content}`)
    .join('\n')
  deps.setConversationContext(conversationContext)

  const skillTask = (async (): Promise<{
    skillsPrompt: string
    activatedSkills: import('./skills/types').SkillIndexEntry[]
    writableSkillDirs: string[]
  }> => {
    let skillsPrompt = ''
    let activatedSkills: import('./skills/types').SkillIndexEntry[] = []
    let writableSkillDirs: string[] = []
    try {
      const skillLlm = await getLLMAdapter()
      if (skillLlm) {
        const isShortResponse = content.length <= SHORT_RESPONSE_MAX_LEN
          && SHORT_RESPONSE_PATTERN.test(content.trim())
        if (isShortResponse && lastMatchedConvId === conv.id && lastMatchedSkills.length > 0) {
          activatedSkills = lastMatchedSkills
        } else {
          const recentForSkill = messages.length > 4 ? messages.slice(-4) : messages
          activatedSkills = await skillManager.matchSkills(content, skillLlm, recentForSkill)
          lastMatchedSkills = activatedSkills
          lastMatchedConvId = conv.id
        }
        if (activatedSkills.length > 0) {
          const activatedNames = activatedSkills.map(s => s.name)
          const contentMap = await skillManager.getSkillContentMap(activatedNames)
          skillsPrompt = buildSkillsPrompt(activatedSkills, contentMap)
          writableSkillDirs = skillManager.getWritableSkillDirs(activatedNames)
        }
      }
    } catch (e) {
      console.warn('[Dispatcher] Skill matching failed:', e)
    }
    return { skillsPrompt, activatedSkills, writableSkillDirs }
  })()

  const memoryTask = (async (): Promise<string> => {
    if (settings.fbmSmartRecall !== false || !deps.fbmStore.isEnabled()) return ''
    try { await deps.fbmStore.ensureInit() } catch (e) { console.warn('[Dispatcher] FBM ensureInit failed:', e) }
    try {
      callbacks.onMemoryRecallStart()
      const userMessages = conv.messages
        .filter(m => !m.isLoading && !m.isGreeting && m.role === 'user')
        .map(m => m.content.filter(c => c.type === 'text' && c.text).map(c => c.text || '').join(''))
        .filter(t => t.length > 0)
      const recentUserMessages = userMessages.slice(-3)
      const result = await deps.fbmStore.retrieve(recentUserMessages, conversationContext)
      if (result && result.summary && result.summary !== '没有找到相关记忆') {
        const keywords = deps.fbmStore.getLastRetrieveKeywords()
        let memoryText = result.summary
        if (keywords.length > 0) {
          memoryText = `检索关键词: ${keywords.join(', ')}\n\n${memoryText}`
        }
        callbacks.onMemoryRecallComplete(result.summary, keywords)
        return [
          '',
          '---',
          '',
          '## 当前对话相关记忆',
          '',
          memoryText,
          '',
        ].join('\n')
      } else {
        const keywords = deps.fbmStore.getLastRetrieveKeywords()
        const summary = result?.summary ?? '没有找到相关记忆'
        callbacks.onMemoryRecallComplete(summary, keywords)
      }
    } catch (err) {
      console.warn('[Dispatcher] Auto retrieve failed:', err)
      callbacks.onMemoryRecallError()
    }
    return ''
  })()

  const [{ skillsPrompt, activatedSkills, writableSkillDirs }, memoryContext] = await Promise.all([skillTask, memoryTask])

  const systemPrompt = await assembleSystemPrompt(skillsPrompt)

  const userTools = await loadAllTools()
  const filtered = userTools.filter(t => !t.id.startsWith('sys-') && !t.id.startsWith('builtin-'))
  const allTools: Tool[] = [...sysTools, ...filtered]
  if (deps.fbmStore.isEnabled()) {
    allTools.push(memorySearchTool)
  }
  allTools.push(roleConfigTool)

  deps.fairyDo.registerAll(allTools)
  deps.setCurrentTools(allTools)
  deps.setCurrentProviderId(provider.id)

  const contextResult = await buildContextMessages({
    systemPrompt,
    memoryContext,
    messages,
    conversationSummary: conv.summary,
    conversationSummaryUpdatedAt: conv.summaryUpdatedAt,
    conversationId: conv.id,
    providerId: provider.id,
  })

  const llmMessages = contextResult.messages

  if (contextResult.compressedCount > 0) {
    callbacks.onContextCompressed(conv.id, contextResult.compressedCount)
  }

  try {
    const result = await executeReActLoop(provider.id, llmMessages, {
      tools: allTools,
      extraAllowedPaths: writableSkillDirs.length > 0 ? writableSkillDirs : undefined,
      signal,
      onApproveAccess: callbacks.onApproveAccess,
      onTurnStart: () => {
        callbacks.onTurnStart()
      },
      onChunk: (chunk) => {
        callbacks.onChunk(chunk)
      },
      onReasoningChunk: (reasoning) => {
        callbacks.onReasoningChunk(reasoning)
      },
      onToolExecuting: (toolCallId, tool, input) => {
        callbacks.onToolExecuting(toolCallId, tool, input)
      },
      onToolResult: (tool, result) => {
        callbacks.onToolResult(tool, result)
      },
      onTurnEnd: () => {
        callbacks.onTurnEnd('')
      }
    })

    return { content: result.content, iterations: result.iterations }
  } catch (error) {
    throw error
  } finally {
    runBackgroundTasks(deps, callbacks, {
      convId: conv.id,
      providerId: provider.id,
      content,
      activatedSkills,
      contextResult,
      settings,
      recentMessages: messages.length > 4 ? messages.slice(-4) : messages,
    })
  }
}
