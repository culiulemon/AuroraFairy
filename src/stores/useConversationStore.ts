import { ref, computed } from 'vue'
import { openDB } from './db'
import {
  loadAllConversations,
  saveConversation,
  debouncedSave,
  deleteConversation as deleteConversationFromDB,
  createNewConversation as createNewConv,
  addMessage as addMsg,
  updateMessage as updateMsg,
  appendToMessage as appendMsg,
  appendReasoningToMessage as appendReasoningMsg,
  setRecallingToMessage as setRecallingMsg,
  setLoadingState,
  setMessageError,
  toSimpleMessages,
  getNextMessageId,
  addToolCallToMessage as addToolCall,
  addToolResultToMessage as addToolResult
} from './conversationStore'
import type { Conversation, Message, MessageContent } from './conversation'
import { createTextContent, getTextFromMessage } from './conversation'

const conversations = ref<Conversation[]>([])
const activeConversationId = ref<string | null>(null)
const isInitialized = ref(false)

const activeConversation = computed(() => {
  return conversations.value.find(c => c.id === activeConversationId.value) || null
})

async function initialize(): Promise<void> {
  if (isInitialized.value) {
    return
  }
  
  try {
    console.log('[ConversationStore] 开始初始化...')
    await openDB()
    console.log('[ConversationStore] IndexedDB 打开成功')
    
    const loadedConversations = await loadAllConversations()
    console.log('[ConversationStore] 加载对话数量:', loadedConversations.length)
    
    conversations.value = loadedConversations
    
    if (loadedConversations.length > 0) {
      activeConversationId.value = loadedConversations[0].id
      console.log('[ConversationStore] 激活对话:', activeConversationId.value)
    }
    
    isInitialized.value = true
    console.log('[ConversationStore] 初始化完成')
  } catch (error) {
    console.error('[ConversationStore] 初始化对话存储失败:', error)
    throw error
  }
}

async function createNewConversation(): Promise<Conversation> {
  if (!isInitialized.value) {
    console.log('[ConversationStore] 数据库未初始化，先初始化...')
    await initialize()
  }
  
  const conversation = createNewConv()
  conversations.value.unshift(conversation)
  activeConversationId.value = conversation.id
  console.log('[ConversationStore] 创建新对话:', conversation.id)
  saveConversation(conversation).catch(err => {
    console.error('[ConversationStore] 保存新对话失败:', err)
  })
  return conversation
}

function selectConversation(id: string): void {
  activeConversationId.value = id
}

async function deleteConversation(id: string): Promise<void> {
  if (!isInitialized.value) {
    console.log('[ConversationStore] 数据库未初始化，先初始化...')
    await initialize()
  }
  
  const index = conversations.value.findIndex(c => c.id === id)
  if (index === -1) return
  
  conversations.value.splice(index, 1)
  
  if (activeConversationId.value === id) {
    activeConversationId.value = conversations.value[0]?.id || null
  }
  
  await deleteConversationFromDB(id)
}

function addMessage(
  conversationId: string,
  role: 'user' | 'assistant',
  content: MessageContent[]
): Message | null {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return null
  
  const message = addMsg(conversation, role, content)
  
  if (isInitialized.value) {
    debouncedSave(conversation)
  } else {
    console.warn('[ConversationStore] 数据库未初始化，消息未保存')
  }
  
  return message
}

function addLoadingMessage(conversationId: string): string | null {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return null
  
  const messageId = getNextMessageId(conversation.id)
  const message: Message = {
    id: messageId,
    role: 'assistant',
    content: [],
    timestamp: new Date().toISOString(),
    isLoading: true
  }
  
  conversation.messages.push(message)
  
  if (isInitialized.value) {
    debouncedSave(conversation)
  }
  
  return messageId
}

function updateMessage(conversationId: string, messageId: string, content: MessageContent[]): void {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return
  
  updateMsg(conversation, messageId, content)
  debouncedSave(conversation)
}

function appendToMessage(conversationId: string, messageId: string, text: string): void {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return
  
  appendMsg(conversation, messageId, text)
  debouncedSave(conversation)
}

function setMessageLoading(conversationId: string, messageId: string, isLoading: boolean): void {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return
  
  setLoadingState(conversation, messageId, isLoading)
  if (!isLoading) {
    debouncedSave(conversation)
  }
}

function setErrorMessage(conversationId: string, messageId: string, error: string): void {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return

  setMessageError(conversation, messageId, error)
  debouncedSave(conversation)
}

function appendReasoningToMessage(conversationId: string, messageId: string, reasoning: string): void {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return

  appendReasoningMsg(conversation, messageId, reasoning)
  debouncedSave(conversation)
}

function setRecallingToMessage(conversationId: string, messageId: string, recalling: string): void {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return

  setRecallingMsg(conversation, messageId, recalling)
  debouncedSave(conversation)
}

function addToolCallToMessage(conversationId: string, messageId: string, toolCallId: string, toolName: string, toolInput: Record<string, unknown>): void {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return

  addToolCall(conversation, messageId, toolCallId, toolName, toolInput)
  debouncedSave(conversation)
}

function addToolResultToMessage(conversationId: string, messageId: string, toolName: string, toolResult: { success: boolean; data?: string; error?: { code: string; message: string } }): void {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return

  addToolResult(conversation, messageId, toolName, toolResult)
  debouncedSave(conversation)
}

function getSimpleMessages(conversationId: string): Array<{ role: string; content: string }> {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return []
  
  return toSimpleMessages(conversation.messages)
}

function updateConversationTitle(conversationId: string, title: string): void {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return

  conversation.title = title
  debouncedSave(conversation)
}

function markMessagesConsolidated(conversationId: string): void {
  const conversation = conversations.value.find(c => c.id === conversationId)
  if (!conversation) return

  for (const msg of conversation.messages) {
    if (!msg.isLoading) {
      msg.memoryConsolidated = true
    }
  }

  debouncedSave(conversation)
}

function findOrCreateByExternalChatId(
  source: string,
  externalChatId: string,
  accountId: string,
  title: string,
): Conversation {
  const matches = conversations.value.filter(
    c => c.source === source && c.externalChatId === externalChatId
  )
  if (matches.length > 0) {
    matches.sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
    return matches[0]
  }

  const conversation = createNewConv()
  conversation.source = source
  conversation.externalChatId = externalChatId
  conversation.accountId = accountId
  conversation.title = title
  conversations.value.unshift(conversation)
  saveConversation(conversation).catch(err => {
    console.error('[ConversationStore] 保存外部渠道对话失败:', err)
  })
  return conversation
}

function createNewExternalConversation(
  source: string,
  externalChatId: string,
  accountId: string,
  title: string,
): Conversation {
  const conversation = createNewConv()
  conversation.source = source
  conversation.externalChatId = externalChatId
  conversation.accountId = accountId
  conversation.title = title
  conversations.value.unshift(conversation)
  saveConversation(conversation).catch(err => {
    console.error('[ConversationStore] 保存新外部渠道对话失败:', err)
  })
  return conversation
}

export function useConversationStore() {
  return {
    conversations,
    activeConversationId,
    activeConversation,
    isInitialized,
    initialize,
    createNewConversation,
    selectConversation,
    deleteConversation,
    addMessage,
    addLoadingMessage,
    updateMessage,
    appendToMessage,
    appendReasoningToMessage,
    setRecallingToMessage,
    setMessageLoading,
    setErrorMessage,
    getSimpleMessages,
    createTextContent,
    getTextFromMessage,
    addToolCallToMessage,
    addToolResultToMessage,
    markMessagesConsolidated,
    updateConversationTitle,
    findOrCreateByExternalChatId,
    createNewExternalConversation
  }
}
