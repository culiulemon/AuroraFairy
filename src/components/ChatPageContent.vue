<template>
  <div class="chat-page">
    <ChatSidebar
      v-if="!isInitializing"
      :conversations="conversations"
      :activeId="activeConversationId"
      @select="selectConversation"
      @create="handleCreateConversation"
      @delete="handleDeleteConversation"
    />
    <div v-else class="chat-sidebar-loading">
      <div class="loading-spinner"></div>
      <span>加载中...</span>
    </div>
    <div class="chat-divider"></div>
    <ChatWindow
      v-if="activeConversation"
      :conversation="activeConversation"
      :isGenerating="isGenerating"
      @send="handleSendMessage"
      @stop="handleStopGenerating"
    />
    <div v-else class="chat-empty">
      <div class="empty-content">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" width="80" height="80" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
          </svg>
        </div>
        <p>选择一个会话开始对话</p>
        <span>或点击上方按钮创建新会话</span>
      </div>
    </div>
    <ToolApprovalDialog
      :visible="showApprovalDialog"
      :tool-name="approvalToolName"
      :target-path="approvalTargetPath"
      @approve="handleApprovalApprove"
      @deny="handleApprovalDeny"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, nextTick, triggerRef, ref, watch } from 'vue'
import ChatSidebar from './ChatSidebar.vue'
import ChatWindow from './ChatWindow.vue'
import ToolApprovalDialog from './ToolApprovalDialog.vue'
import { useConversationStore } from '../stores/useConversationStore'
import { loadTtsSettings } from '../stores/settings'
import { fairyDo } from '../agent/fairyDo'
import { registerVirtualHandlers, setCurrentUserMessage, setRecentUserMessages } from '../agent/virtualHandlers'
import { generateConversationTitle } from '../stores/chat'
import { useTTS } from '../composables/useTTS'
import { fbmStore } from '../stores/fbmStore'
import { dispatchMessage } from '../agent/messageDispatcher'
import type { Tool } from '../types/tool'
import { channelRegistry } from '../agent/channelRegistry'

const {
  conversations,
  activeConversationId,
  activeConversation,
  initialize,
  createNewConversation,
  selectConversation,
  deleteConversation,
  addMessage,
  addLoadingMessage,
  appendToMessage,
  appendReasoningToMessage,
  setMessageLoading,
  setErrorMessage,
  createTextContent,
  addToolCallToMessage,
  addToolResultToMessage,
  setRecallingToMessage,
  markMessagesConsolidated,
  updateConversationTitle
} = useConversationStore()

const isGenerating = ref(false)
let abortController: AbortController | null = null
let currentLoadingMessageId: string | null = null
let currentMessageHasContent = false
let needsNewMessage = false
let currentProviderId = ''
let currentTools: Tool[] = []

function ensureMessage(_forText = false) {
  if (!activeConversation.value) return
  if (needsNewMessage && currentMessageHasContent && currentLoadingMessageId) {
    setMessageLoading(activeConversation.value.id, currentLoadingMessageId, false)
    currentLoadingMessageId = null
    currentMessageHasContent = false
  }
  if (!currentLoadingMessageId) {
    const msgId = addLoadingMessage(activeConversation.value.id)
    currentLoadingMessageId = msgId
    currentMessageHasContent = false
  }
  needsNewMessage = false
  triggerRef(conversations)
  nextTick(() => {
    const list = document.querySelector('.message-list')
    if (list) list.scrollTop = list.scrollHeight
  })
}

const showApprovalDialog = ref(false)
const approvalToolName = ref('')
const approvalTargetPath = ref('')
let approvalResolve: ((value: boolean) => void) | null = null

function requestAccessApproval(toolName: string, targetPath: string): Promise<boolean> {
  return new Promise((resolve) => {
    approvalToolName.value = toolName
    approvalTargetPath.value = targetPath
    showApprovalDialog.value = true
    approvalResolve = resolve
  })
}

function handleApprovalApprove() {
  showApprovalDialog.value = false
  if (approvalResolve) {
    approvalResolve(true)
    approvalResolve = null
  }
}

function handleApprovalDeny() {
  showApprovalDialog.value = false
  if (approvalResolve) {
    approvalResolve(false)
    approvalResolve = null
  }
}

const { speak } = useTTS()

registerVirtualHandlers(fairyDo, {
  getCurrentTools: () => currentTools,
  getCurrentProviderId: () => currentProviderId,
})

const isInitializing = ref(true)

onMounted(() => {
  initialize().finally(() => {
    isInitializing.value = false
  })
})

watch(activeConversationId, () => {
  fbmStore.resetConsolidationState()
})

const handleDeleteConversation = async (id: string) => {
  await deleteConversation(id)
}

const handleCreateConversation = async () => {
  await createNewConversation()
}

const handleSendMessage = async (content: string) => {
  if (!activeConversation.value) return

  abortController = new AbortController()
  isGenerating.value = true

  setCurrentUserMessage(content)
  addMessage(activeConversation.value.id, 'user', [createTextContent(content)])

  const recentMsgs = activeConversation.value.messages
    .filter(m => !m.isLoading && !m.isGreeting && m.role === 'user')
    .map(m => m.content.filter(c => c.type === 'text' && c.text).map(c => c.text || '').join(''))
    .filter(t => t.length > 0)
    .slice(-3)
  setRecentUserMessages(recentMsgs.length > 0 ? recentMsgs : [content])

  currentLoadingMessageId = null
  currentMessageHasContent = false
  needsNewMessage = false

  ensureMessage()

  try {
    await dispatchMessage(content, {
      getActiveConversation: () => activeConversation.value,
      addMessage: (convId, role, msgContent) => addMessage(convId, role, msgContent as any),
      getConversationMessages: (convId) => {
        const conv = conversations.value.find(c => c.id === convId)
        if (!conv) return []
        return conv.messages
          .filter(m => !m.isLoading && !m.isGreeting)
          .map(m => ({
            role: m.role as 'user' | 'assistant',
            content: m.content.filter(c => c.type === 'text' && c.text).map(c => c.text || '').join('')
          }))
      },
      generateTitleIfNeeded: (providerId, userMsg, convId, currentTitle) => {
        if (currentTitle === '新会话') {
          generateConversationTitle(providerId, userMsg).then(title => {
            if (title !== '新会话') {
              updateConversationTitle(convId, title)
            }
          }).catch(err => {
            console.warn('[ChatPage] 生成标题失败:', err)
          })
        }
      },
      fbmStore,
      fairyDo,
      setCurrentUserMessage,
      setRecentUserMessages,
      setCurrentTools: (tools) => { currentTools = tools },
      setCurrentProviderId: (id) => { currentProviderId = id },
      saveConversationSummary: (convId: string, summary: string) => {
        const conv = conversations.value.find(c => c.id === convId)
        if (conv) {
          conv.summary = summary
          conv.summaryUpdatedAt = new Date().toISOString()
          import('../stores/conversationStore').then(({ debouncedSave }) => {
            debouncedSave(conv)
          })
        }
      },
    }, {
      onChunk: (chunk) => {
        if (!activeConversation.value) return
        ensureMessage(true)
        if (!currentLoadingMessageId) return
        appendToMessage(activeConversation.value.id, currentLoadingMessageId, chunk)
        currentMessageHasContent = true
        triggerRef(conversations)
        nextTick(() => {
          const list = document.querySelector('.message-list')
          if (list) list.scrollTop = list.scrollHeight
        })
      },
      onReasoningChunk: (reasoning) => {
        if (!activeConversation.value) return
        ensureMessage(true)
        if (!currentLoadingMessageId) return
        appendReasoningToMessage(activeConversation.value.id, currentLoadingMessageId, reasoning)
        currentMessageHasContent = true
        triggerRef(conversations)
        nextTick(() => {
          const list = document.querySelector('.message-list')
          if (list) list.scrollTop = list.scrollHeight
        })
      },
      onToolExecuting: (toolCallId, tool, input) => {
        if (!activeConversation.value) return
        ensureMessage(false)
        if (!currentLoadingMessageId) return
        addToolCallToMessage(activeConversation.value.id, currentLoadingMessageId, toolCallId, tool, input)
        currentMessageHasContent = true
        if (tool === 'memory_search') {
          setRecallingToMessage(activeConversation.value.id, currentLoadingMessageId, '正在回忆...')
        }
        triggerRef(conversations)
        nextTick(() => {
          const list = document.querySelector('.message-list')
          if (list) list.scrollTop = list.scrollHeight
        })
      },
      onToolResult: (tool, result) => {
        if (!activeConversation.value || !currentLoadingMessageId) return
        addToolResultToMessage(activeConversation.value.id, currentLoadingMessageId, tool, result as any)
        if (tool === 'memory_search') {
          const data = result as { success?: boolean; data?: string } | null
          const summary = data?.success ? data.data : typeof result === 'string' ? result : JSON.stringify(result)
          const keywords = fbmStore.getLastRetrieveKeywords()
          const hasMemory = !!summary && summary !== '没有找到相关记忆'
          let recallingText = hasMemory ? `回忆完成\n${summary}` : (summary ?? '没有找到相关记忆')
          if (keywords.length > 0) {
            recallingText = `检索关键词: ${keywords.join(', ')}\n\n${recallingText}`
          }
          setRecallingToMessage(activeConversation.value.id, currentLoadingMessageId, recallingText)
        }
        triggerRef(conversations)
        nextTick(() => {
          const list = document.querySelector('.message-list')
          if (list) list.scrollTop = list.scrollHeight
        })
      },
      onTurnStart: () => {
        needsNewMessage = true
      },
      onTurnEnd: (_messageId) => {
        if (!activeConversation.value || !currentLoadingMessageId) return
        const msgId = currentLoadingMessageId
        setMessageLoading(activeConversation.value.id, msgId, false)
        currentLoadingMessageId = null

        if (activeConversation.value.source) {
          const msg = activeConversation.value.messages.find(m => m.id === msgId)
          if (msg) {
            const replyText = msg.content
              .filter(c => c.type === 'text' && c.text)
              .map(c => c.text || '')
              .join('')
            if (replyText.trim()) {
              channelRegistry.sendReply(
                activeConversation.value.source,
                activeConversation.value.accountId!,
                activeConversation.value.externalChatId!,
                replyText,
                { lastMessageId: '' },
              ).catch(err => console.error('[ChatPage] Channel reply failed:', err))
            }
          }
        }

        const ttsSettings = loadTtsSettings()
        if (ttsSettings.enabled && activeConversation.value) {
          const msg = activeConversation.value.messages.find(m => m.id === msgId)
          if (msg) {
            const text = msg.content
              .filter(c => c.type === 'text' && c.text)
              .map(c => c.text || '')
              .join('')
            if (text) {
              speak(text, msgId)
            }
          }
        }
      },
      onApproveAccess: requestAccessApproval,
      onMemoryRecallStart: () => {
        ensureMessage(false)
        if (currentLoadingMessageId && activeConversation.value) {
          setRecallingToMessage(activeConversation.value.id, currentLoadingMessageId, '正在回忆...')
          triggerRef(conversations)
        }
      },
      onMemoryRecallComplete: (summary, keywords) => {
        if (!currentLoadingMessageId || !activeConversation.value) return
        const hasMemory = summary && summary !== '没有找到相关记忆'
        let recallingText = hasMemory ? `回忆完成\n${summary}` : summary
        if (keywords.length > 0) {
          recallingText = `检索关键词: ${keywords.join(', ')}\n\n${recallingText}`
        }
        setRecallingToMessage(activeConversation.value.id, currentLoadingMessageId, recallingText)
        triggerRef(conversations)
      },
      onMemoryRecallError: () => {
        if (currentLoadingMessageId && activeConversation.value) {
          setRecallingToMessage(activeConversation.value.id, currentLoadingMessageId, '回忆失败')
          triggerRef(conversations)
        }
      },
      onConsolidationComplete: (convId) => {
        markMessagesConsolidated(convId)
      },
      onContextCompressed: (_convId, compressedCount) => {
        if (activeConversation.value) {
          activeConversation.value.compressedMessageCount = compressedCount
        }
      },
    }, abortController.signal)
  } catch (error) {
    if (activeConversation.value) {
      if (!abortController?.signal.aborted) {
        setErrorMessage(
          activeConversation.value.id,
          currentLoadingMessageId || '',
          `请求失败: ${error instanceof Error ? error.message : '未知错误'}`
        )
      }
    }
  } finally {
    isGenerating.value = false
    abortController = null
    currentLoadingMessageId = null
    currentMessageHasContent = false
    needsNewMessage = false
    showApprovalDialog.value = false
    approvalResolve = null
  }
}

const handleStopGenerating = () => {
  if (abortController) {
    abortController.abort()
  }
  if (approvalResolve) {
    approvalResolve(false)
    approvalResolve = null
    showApprovalDialog.value = false
  }
}
</script>

<style scoped>
.chat-page {
  display: flex;
  width: 100%;
  height: 100%;
  background: var(--color-surface-card);
  overflow: hidden;
}

.chat-divider {
  width: 1px;
  background: linear-gradient(180deg, transparent 0%, var(--color-border) 50%, transparent 100%);
}

.chat-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface);
}

.empty-content {
  text-align: center;
  color: var(--color-text-muted);
}

.empty-icon {
  margin-bottom: 24px;
  opacity: 0.5;
  color: var(--color-text-secondary);
}

.empty-icon svg {
  animation: float 3s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-8px); }
}

.empty-content p {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.empty-content span {
  font-size: 13px;
}

.chat-sidebar-loading {
  width: 240px;
  min-width: 240px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  background: var(--color-surface-card);
  color: var(--color-text-muted);
  font-size: 13px;
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
