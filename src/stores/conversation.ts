export interface MessageContent {
  type: 'text' | 'image' | 'audio' | 'code' | 'file' | 'tool_call' | 'tool_result'
  text?: string
  reasoning?: string
  recalling?: string
  attachmentId?: string
  language?: string
  filename?: string
  mimeType?: string
  size?: number
  toolName?: string
  toolInput?: Record<string, unknown>
  toolResult?: {
    success: boolean
    data?: string
    error?: {
      code: string
      message: string
    }
  }
  toolCallId?: string
}

export interface Message {
  id: string
  role: 'user' | 'assistant'
  content: MessageContent[]
  timestamp: string
  isLoading?: boolean
  isGreeting?: boolean
  memoryConsolidated?: boolean
}

export interface Conversation {
  id: string
  title: string
  messages: Message[]
  createdAt: string
  updatedAt: string
  messageCount: number
  preview?: string
  source?: 'local' | string
  externalChatId?: string
  accountId?: string
  summary?: string
  summaryUpdatedAt?: string
  compressedMessageCount?: number
}

export interface Attachment {
  id: string
  filename: string
  mimeType: string
  size: number
  data: Blob
  createdAt: string
}

export function createTextContent(text: string): MessageContent {
  return {
    type: 'text',
    text
  }
}

export function getTextFromMessage(message: Message): string {
  return message.content
    .filter(c => c.type === 'text' && c.text)
    .map(c => c.text || '')
    .join('')
}

export function updateConversationMeta(conversation: Conversation): void {
  conversation.messageCount = conversation.messages.length
  conversation.updatedAt = new Date().toISOString()
  
  if (conversation.messages.length > 0) {
    const lastMsg = conversation.messages[conversation.messages.length - 1]
    const text = getTextFromMessage(lastMsg)
    conversation.preview = text.substring(0, 50) + (text.length > 50 ? '...' : '')
  } else {
    conversation.preview = ''
  }
}
