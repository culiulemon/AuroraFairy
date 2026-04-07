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
</style>
