<template>
  <div class="chat-sidebar">
    <div class="sidebar-header">
      <button class="new-chat-btn" @click="$emit('create')">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
        新对话
      </button>
    </div>
    <div class="conversation-list">
      <div
        v-for="conv in conversations"
        :key="conv.id"
        class="conversation-item"
        :class="{ active: conv.id === activeId }"
        @click="$emit('select', conv.id)"
      >
        <div class="avatar-wrapper">
          <div class="avatar">
            <span>{{ conv.title.charAt(0) }}</span>
          </div>
          <div v-if="channelRegistry.getBadge(conv.source)" class="source-badge"
            :style="{ background: channelRegistry.getBadge(conv.source)!.backgroundColor }"
            v-html="channelRegistry.getBadge(conv.source)!.svgContent">
          </div>
        </div>
        <div class="conv-info">
          <div class="conv-header">
            <span class="conv-title">{{ conv.title }}</span>
            <span class="conv-time">{{ formatTime(conv.updatedAt) }}</span>
          </div>
          <div class="conv-preview">
            {{ conv.preview || getLastMessage(conv.messages) }}
          </div>
        </div>
        <button class="delete-btn" @click.stop="$emit('delete', conv.id)" title="删除">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3,6 5,6 21,6"></polyline>
            <path d="M19,6v14a2,2,0,0,1-2,2H7a2,2,0,0,1-2-2V6m3,0V4a2,2,0,0,1,2-2h4a2,2,0,0,1,2,2v2"></path>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Conversation, Message } from '../stores/conversation'
import { channelRegistry } from '../agent/channelRegistry'

defineProps<{
  conversations: Conversation[]
  activeId: string | null
}>()

defineEmits<{
  select: [id: string]
  create: []
  delete: [id: string]
}>()

const formatTime = (dateStr: string): string => {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)
  const days = Math.floor(diff / 86400000)

  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes}分钟前`
  if (hours < 24) return `${hours}小时前`
  if (days < 7) return `${days}天前`
  return date.toLocaleDateString('zh-CN')
}

const getLastMessage = (messages: Message[]): string => {
  if (messages.length === 0) return ''
  const text = messages[messages.length - 1].content
    .filter(c => c.type === 'text' && c.text)
    .map(c => c.text || '')
    .join('')
  return text.substring(0, 28) + (text.length > 28 ? '...' : '')
}
</script>

<style scoped>
.chat-sidebar {
  width: 280px;
  min-width: 280px;
  height: 100%;
  background: var(--color-surface-card);
  display: flex;
  flex-direction: column;
  box-shadow: 2px 0 12px var(--color-shadow-alpha-06);
}

.sidebar-header {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  height: auto;
  min-height: 60px;
  border-bottom: 1px solid var(--color-border);
}

.sidebar-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: var(--color-text-primary);
  letter-spacing: -0.02em;
}

.new-chat-btn {
  width: 100%;
  padding: 10px 16px;
  border: none;
  background: var(--color-primary-alpha-10);
  cursor: pointer;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-primary);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.new-chat-btn:hover {
  background: var(--color-primary-alpha-15);
  transform: translateY(-1px);
}

.conversation-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.conversation-list::-webkit-scrollbar {
  width: 4px;
}

.conversation-list::-webkit-scrollbar-track {
  background: transparent;
}

.conversation-list::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 4px;
}

.conversation-item {
  display: flex;
  align-items: center;
  padding: 14px 16px;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: 12px;
  margin-bottom: 4px;
  position: relative;
}

.conversation-item:hover {
  background: var(--color-primary-alpha-08);
}

.conversation-item.active {
  background: var(--color-primary-alpha-08);
}

.conversation-item.active .avatar {
  background: var(--color-primary-alpha-15);
  color: var(--color-primary);
}

.avatar {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: var(--color-primary-alpha-10);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-primary);
  font-size: 16px;
  font-weight: 700;
  flex-shrink: 0;
  transition: all 0.25s ease;
}

.avatar-wrapper {
  position: relative;
  flex-shrink: 0;
}

.source-badge {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px solid var(--color-surface-card);
}

.source-badge :deep(svg) {
  width: 10px;
  height: 10px;
}

.conversation-item:hover .avatar {
  background: var(--color-primary-alpha-12);
}

.conv-info {
  flex: 1;
  margin-left: 14px;
  overflow: hidden;
}

.conv-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.conv-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.conv-time {
  font-size: 11px;
  color: var(--color-text-secondary);
  flex-shrink: 0;
  margin-left: 8px;
  font-weight: 600;
}

.conv-preview {
  font-size: 12px;
  color: var(--color-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.4;
}

.delete-btn {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%) scale(0.8);
  width: 32px;
  height: 32px;
  border: none;
  background: var(--color-surface-card);
  cursor: pointer;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  opacity: 0;
  transition: all 0.25s;
  box-shadow: 0 2px 8px var(--color-shadow-alpha-08);
}

.conversation-item:hover .delete-btn {
  opacity: 1;
  transform: translateY(-50%) scale(1);
}

.delete-btn:hover {
  background: var(--color-danger-bg);
  color: var(--color-accent-error);
}
</style>
