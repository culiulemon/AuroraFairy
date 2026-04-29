import { getDB } from './db'
import type { Conversation, Message, MessageContent } from './conversation'
import { updateConversationMeta, getTextFromMessage } from './conversation'
import { deleteAttachmentsForConversation } from './attachment'

let saveTimeout: ReturnType<typeof setTimeout> | null = null
let messageCounter = 0

export function getNextMessageId(conversationId: string): string {
  return `${conversationId}-${Date.now()}-${++messageCounter}`
}

export function saveConversation(conversation: Conversation): Promise<void> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['conversations'], 'readwrite')
    const store = transaction.objectStore('conversations')
    
    console.log('[ConversationStore] 保存对话:', conversation.id, '消息数:', conversation.messages.length)
    
    const plainConversation = JSON.parse(JSON.stringify(conversation))
    const request = store.put(plainConversation)
    
    request.onsuccess = () => {
      console.log('[ConversationStore] 对话保存成功:', conversation.id)
      resolve()
    }
    request.onerror = () => {
      console.error('[ConversationStore] 对话保存失败:', conversation.id)
      reject(new Error('保存对话失败'))
    }
  })
}

export function debouncedSave(conversation: Conversation): void {
  if (saveTimeout) {
    clearTimeout(saveTimeout)
  }
  saveTimeout = setTimeout(() => {
    saveConversation(conversation)
    saveTimeout = null
  }, 500)
}

export function loadAllConversations(): Promise<Conversation[]> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['conversations'], 'readonly')
    const store = transaction.objectStore('conversations')
    const index = store.index('updatedAt')
    
    console.log('[ConversationStore] 加载所有对话...')
    const request = index.openCursor(null, 'prev')
    const conversations: Conversation[] = []
    
    request.onsuccess = (event) => {
      const cursor = (event.target as IDBRequest<IDBCursorWithValue>).result
      if (cursor) {
        conversations.push(cursor.value)
        cursor.continue()
      } else {
        console.log('[ConversationStore] 加载完成，对话数:', conversations.length)
        resolve(conversations)
      }
    }
    
    request.onerror = () => {
      console.error('[ConversationStore] 加载对话列表失败:', request.error)
      reject(new Error('加载对话列表失败'))
    }
  })
}

export function loadConversation(id: string): Promise<Conversation | null> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['conversations'], 'readonly')
    const store = transaction.objectStore('conversations')
    
    const request = store.get(id)
    
    request.onsuccess = () => resolve(request.result || null)
    request.onerror = () => reject(new Error('加载对话失败'))
  })
}

export function deleteConversation(id: string): Promise<void> {
  return new Promise(async (resolve, reject) => {
    try {
      await deleteAttachmentsForConversation(id)
      
      const db = getDB()
      const transaction = db.transaction(['conversations'], 'readwrite')
      const store = transaction.objectStore('conversations')
      
      const request = store.delete(id)
      
      request.onsuccess = () => resolve()
      request.onerror = () => reject(new Error('删除对话失败'))
    } catch (error) {
      reject(error)
    }
  })
}

export function createNewConversation(): Conversation {
  const now = new Date().toISOString()
  const id = Date.now().toString()
  
  const conversation: Conversation = {
    id,
    title: '新会话',
    messages: [
      {
        id: getNextMessageId(id),
        role: 'assistant',
        content: [{ type: 'text', text: '你好！我是Fairy，有什么可以帮助你的吗？' }],
        timestamp: now,
        isGreeting: true
      }
    ],
    createdAt: now,
    updatedAt: now,
    messageCount: 1,
    preview: '你好！我是Fairy，有什么可以帮助你的吗？'
  }
  
  return conversation
}

export function addMessage(
  conversation: Conversation,
  role: 'user' | 'assistant',
  content: MessageContent[]
): Message {
  const message: Message = {
    id: getNextMessageId(conversation.id),
    role,
    content,
    timestamp: new Date().toISOString()
  }
  
  conversation.messages.push(message)
  updateConversationMeta(conversation)
  
  return message
}

export function updateMessage(
  conversation: Conversation,
  messageId: string,
  content: MessageContent[]
): void {
  const message = conversation.messages.find(m => m.id === messageId)
  if (message) {
    message.content = content
    updateConversationMeta(conversation)
  }
}

export function appendToMessage(
  conversation: Conversation,
  messageId: string,
  text: string
): void {
  if (!text) return

  const message = conversation.messages.find(m => m.id === messageId)
  if (message) {
    const textContent = message.content.find(c => c.type === 'text')
    if (textContent) {
      textContent.text = (textContent.text || '') + text
    } else {
      message.content.push({ type: 'text', text })
    }
    updateConversationMeta(conversation)
  }
}

export function appendReasoningToMessage(
  conversation: Conversation,
  messageId: string,
  reasoning: string
): void {
  if (!reasoning) return

  const message = conversation.messages.find(m => m.id === messageId)
  if (message) {
    const reasoningContent = message.content.find(c => c.type === 'text' && c.reasoning !== undefined)
    if (reasoningContent) {
      reasoningContent.reasoning = (reasoningContent.reasoning || '') + reasoning
    } else {
      message.content.push({ type: 'text', reasoning })
    }
    updateConversationMeta(conversation)
  }
}

export function setRecallingToMessage(
  conversation: Conversation,
  messageId: string,
  recalling: string
): void {
  const trimmed = recalling.trim()
  if (!trimmed) return

  const message = conversation.messages.find(m => m.id === messageId)
  if (message) {
    const existing = message.content.find(c => c.type === 'text' && c.recalling !== undefined)
    if (existing) {
      existing.recalling = trimmed
    } else {
      message.content.push({ type: 'text', recalling: trimmed })
    }
    updateConversationMeta(conversation)
  }
}

export function addToolCallToMessage(
  conversation: Conversation,
  messageId: string,
  toolCallId: string,
  toolName: string,
  toolInput: Record<string, unknown>
): void {
  const message = conversation.messages.find(m => m.id === messageId)
  if (message) {
    message.content.push({
      type: 'tool_call',
      toolCallId,
      toolName,
      toolInput
    })
    updateConversationMeta(conversation)
  }
}

export function addToolResultToMessage(
  conversation: Conversation,
  messageId: string,
  toolName: string,
  toolResult: {
    success: boolean
    data?: string
    error?: { code: string; message: string }
  }
): void {
  const message = conversation.messages.find(m => m.id === messageId)
  if (message) {
    message.content.push({
      type: 'tool_result',
      toolName,
      toolResult
    })
    updateConversationMeta(conversation)
  }
}

export function setLoadingState(
  conversation: Conversation,
  messageId: string,
  isLoading: boolean
): void {
  const message = conversation.messages.find(m => m.id === messageId)
  if (message) {
    message.isLoading = isLoading
  }
}

export function setMessageError(
  conversation: Conversation,
  messageId: string,
  error: string
): void {
  const message = conversation.messages.find(m => m.id === messageId)
  if (message) {
    message.content = [{ type: 'text', text: error }]
    message.isLoading = false
    updateConversationMeta(conversation)
  }
}

export function toSimpleMessages(messages: Message[]): Array<{ role: string; content: string; reasoning_content?: string }> {
  return messages
    .filter(m => !m.isLoading)
    .map(m => {
      const text = getTextFromMessage(m)
      const reasoning = m.content
        .filter(c => c.type === 'text' && c.reasoning)
        .map(c => c.reasoning || '')
        .join('')
      return {
        role: m.role,
        content: text,
        ...(reasoning && m.role === 'assistant' ? { reasoning_content: reasoning } : {})
      }
    })
}
