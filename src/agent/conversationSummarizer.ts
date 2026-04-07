import { sendChatMessage, type ChatMessage } from '../stores/chat'
import { loadSettings } from '../stores/settings'

export async function generateSummary(
  messages: Array<{ role: string; content: string }>,
  existingSummary?: string,
  providerId?: string
): Promise<string | null> {
  try {
    const settings = loadSettings()
    const provider = providerId
      ? settings.providers.find(p => p.id === providerId)
      : settings.providers.find(p => p.id === settings.defaultProviderId)
        || settings.providers[0]

    if (!provider) {
      return null
    }

    const systemPrompt = existingSummary
      ? '你是一个对话摘要助手。以下是一段对话的已有摘要和一些新的对话内容。请将它们合并为一个新的、更完整的摘要。要求：1) 保留所有关键事实、用户意图、实体信息、决策和结论；2) 保持时间线清晰；3) 不要添加新信息；4) 摘要要简洁但完整；5) 直接输出摘要文本，不要任何解释。'
      : '你是一个对话摘要助手。请对以下对话内容生成一个简洁但完整的摘要。要求：1) 保留所有关键事实、用户意图、实体信息、决策和结论；2) 保持时间线清晰；3) 不要添加新信息；4) 摘要要简洁但完整；5) 直接输出摘要文本，不要任何解释。'

    const formattedMessages = messages.map(m => `[${m.role}]: ${m.content}`).join('\n')

    const userContent = existingSummary
      ? `已有摘要:\n${existingSummary}\n\n新的对话内容:\n${formattedMessages}`
      : formattedMessages

    const chatMessages: ChatMessage[] = [
      { role: 'system', content: systemPrompt },
      { role: 'user', content: userContent }
    ]

    const response = await sendChatMessage({
      providerId: provider.id,
      messages: chatMessages
    })

    return response.content || null
  } catch {
    return null
  }
}
