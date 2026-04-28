<template>
  <div class="chat-window">
    <div class="message-list" ref="messageListRef">
      <template v-for="(msg, index) in conversation.messages" :key="msg.id">
        <div v-if="index === compressedCount && compressedCount > 0" class="context-divider">
          <span class="context-divider-line" />
          <span class="context-divider-text">上下文已压缩</span>
          <span class="context-divider-line" />
        </div>
        <MessageItem
          :message="msg"
        />
      </template>
    </div>
    <div class="info-bar">
      <div class="info-bar-inner">
        <div class="info-capsule" :title="`输入: ${promptTokens}  输出: ${completionTokens}`">
          <svg class="capsule-icon" viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="8" cy="8" r="6.5"></circle>
            <path d="M8 4v4.5l3 1.5"></path>
          </svg>
          <span class="capsule-label">Token</span>
          <span class="capsule-value">{{ totalTokens.toLocaleString() }}</span>
        </div>
        <div class="info-capsule" :title="`输入 token 数`">
          <span class="capsule-label">↑</span>
          <span class="capsule-value">{{ promptTokens.toLocaleString() }}</span>
        </div>
        <div class="info-capsule" :title="`输出 token 数`">
          <span class="capsule-label">↓</span>
          <span class="capsule-value">{{ completionTokens.toLocaleString() }}</span>
        </div>
        <div class="info-spacer"></div>
        <div class="info-capsule workdir-capsule" :class="{ 'has-custom': !!conversation.workdir }" :title="conversation.workdir || '点击设置工作目录'" @click="handleSelectWorkdir">
          <svg class="capsule-icon" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
          <span class="capsule-value workdir-text" v-if="conversation.workdir">{{ shortWorkdir }}</span>
          <span class="capsule-label" v-else>默认目录</span>
        </div>
        <button class="info-capsule open-dir-capsule" v-if="conversation.workdir" @click.stop="handleOpenWorkdir" title="在文件管理器中打开">
          <svg class="capsule-icon" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
            <polyline points="15 3 21 3 21 9"></polyline>
            <line x1="10" y1="14" x2="21" y2="3"></line>
          </svg>
          <span class="capsule-label">打开</span>
        </button>
      </div>
    </div>
    <div class="input-area">
      <textarea
        v-model="inputText"
        placeholder="输入消息..."
        @keydown="handleKeydown"
        rows="1"
        :disabled="isGenerating"
      ></textarea>
      <button
        v-if="isGenerating"
        class="stop-btn"
        @click="handleStop"
      >
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
          <rect x="6" y="6" width="12" height="12" rx="2"></rect>
        </svg>
      </button>
      <button
        v-else
        class="send-btn"
        :class="{ active: inputText.trim() }"
        @click="handleSend"
        :disabled="!inputText.trim()"
      >
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="22" y1="2" x2="11" y2="13"></line>
          <polygon points="22,2 15,22 11,13 2,9 22,2"></polygon>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import MessageItem from './MessageItem.vue'
import type { Conversation } from '../stores/conversation'
import { getTextFromMessage } from '../stores/conversation'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useConversationStore } from '../stores/useConversationStore'

const props = defineProps<{
  conversation: Conversation
  isGenerating?: boolean
}>()

const emit = defineEmits<{
  send: [content: string]
  stop: []
}>()

const inputText = ref('')
const messageListRef = ref<HTMLElement | null>(null)

const compressedCount = computed(() => props.conversation.compressedMessageCount ?? 0)

const promptTokens = computed(() => props.conversation.tokenUsage?.prompt_tokens ?? 0)
const completionTokens = computed(() => props.conversation.tokenUsage?.completion_tokens ?? 0)
const totalTokens = computed(() => props.conversation.tokenUsage?.total_tokens ?? 0)

const shortWorkdir = computed(() => {
  const dir = props.conversation.workdir
  if (!dir) return ''
  const sep = dir.includes('/') ? '/' : '\\'
  const parts = dir.split(sep).filter(Boolean)
  if (parts.length <= 2) return dir
  return '...' + sep + parts.slice(-2).join(sep)
})

const handleSelectWorkdir = async () => {
  try {
    const selected = await open({ directory: true, multiple: false })
    if (selected && typeof selected === 'string') {
      const { updateConversationWorkdir } = useConversationStore()
      updateConversationWorkdir(props.conversation.id, selected)
    }
  } catch (error) {
    console.error('[ChatWindow] 选择工作目录失败:', error)
  }
}

const handleOpenWorkdir = async () => {
  if (!props.conversation.workdir) return
  try {
    await invoke('open_folder', { path: props.conversation.workdir })
  } catch (error) {
    console.error('[ChatWindow] 打开目录失败:', error)
  }
}

const scrollToBottom = () => {
  nextTick(() => {
    if (messageListRef.value) {
      messageListRef.value.scrollTop = messageListRef.value.scrollHeight
    }
  })
}

watch(() => props.conversation.messages.length, scrollToBottom)
watch(() => props.conversation.messages.map(m => getTextFromMessage(m)).join(''), scrollToBottom)

const handleSend = () => {
  if (!inputText.value.trim()) return
  emit('send', inputText.value.trim())
  inputText.value = ''
}

const handleStop = () => {
  emit('stop')
}

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}
</script>

<style scoped>
.chat-window {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--color-surface);
  min-width: 0;
}

.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.message-list::-webkit-scrollbar {
  width: 6px;
}

.message-list::-webkit-scrollbar-track {
  background: transparent;
}

.message-list::-webkit-scrollbar-thumb {
  background: var(--color-border-medium);
  border-radius: 6px;
}

.context-divider {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
  user-select: none;
}

.context-divider-line {
  flex: 1;
  height: 1px;
  background: var(--color-border);
}

.context-divider-text {
  font-size: 12px;
  color: var(--color-text-muted);
  white-space: nowrap;
  opacity: 0.7;
}

.input-area {
  padding: 20px 28px;
  background: var(--color-surface-card);
  border-top: 1px solid var(--color-border);
  display: flex;
  align-items: flex-end;
  gap: 16px;
}

.input-area textarea {
  flex: 1;
  padding: 14px 18px;
  border: 1px solid var(--color-border);
  border-radius: 16px;
  resize: none;
  font-size: 14px;
  font-family: inherit;
  line-height: 1.5;
  max-height: 120px;
  outline: none;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  background: var(--color-surface);
  color: var(--color-text-primary);
}

.input-area textarea:focus {
  border-color: var(--color-primary);
  background: var(--color-surface-card);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.input-area textarea::placeholder {
  color: var(--color-text-muted);
}

.send-btn {
  width: 48px;
  height: 48px;
  border: none;
  background: var(--color-border);
  border-radius: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
}

.send-btn.active {
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
  box-shadow: 0 4px 16px var(--color-shadow-primary-hover);
}

.send-btn.active:hover {
  transform: scale(1.05);
  box-shadow: 0 6px 20px var(--color-shadow-primary-strong);
}

.send-btn:disabled {
  cursor: not-allowed;
}

.stop-btn {
  width: 48px;
  height: 48px;
  border: none;
  background: var(--color-primary-gradient);
  border-radius: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-inverse);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
  box-shadow: 0 4px 16px var(--color-shadow-primary-hover);
}

.stop-btn:hover {
  transform: scale(1.05);
  box-shadow: 0 6px 20px var(--color-shadow-primary-strong);
}

.info-bar {
  padding: 8px 28px;
  background: linear-gradient(to bottom, transparent, var(--color-surface) 40%);
  position: relative;
  z-index: 1;
}

.info-bar-inner {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.info-capsule {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  border-radius: 10px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  font-size: 11px;
  line-height: 1;
  user-select: none;
  transition: all 0.2s ease;
}

.info-capsule:hover {
  border-color: var(--color-primary-alpha-20);
  background: var(--color-primary-alpha-08);
}

.capsule-icon {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.capsule-label {
  color: var(--color-text-muted);
  font-weight: 500;
}

.capsule-value {
  color: var(--color-text-secondary);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.info-spacer {
  flex: 1;
}

.workdir-capsule {
  cursor: pointer;
}

.workdir-capsule.has-custom {
  border-color: var(--color-primary-alpha-20);
  background: var(--color-primary-alpha-06);
}

.workdir-text {
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.open-dir-capsule {
  cursor: pointer;
  border: none;
  background: transparent;
  padding: 5px 8px;
}

.open-dir-capsule:hover {
  background: var(--color-primary-alpha-08);
}
</style>
