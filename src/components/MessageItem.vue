<template>
  <div class="message-item" :class="message.role" :data-message-id="message.id">
    <template v-if="message.role === 'assistant'">
      <div class="message-avatar">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2a2 2 0 0 1 2 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 0 1 7 7h1a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1v1a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-1H2a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h1a7 7 0 0 1 7-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 0 1 2-2z"></path>
          <circle cx="8" cy="14" r="1"></circle>
          <circle cx="12" cy="14" r="1"></circle>
          <circle cx="16" cy="14" r="1"></circle>
        </svg>
      </div>
      <div class="message-content">
        <div v-if="hasDisplayableContent || message.isLoading" class="message-bubble" :class="{ 'has-memory-dot': showMemoryDot }">
          <template v-if="message.isLoading && !hasContent">
            <span class="loading-dots">
              <span></span><span></span><span></span>
            </span>
          </template>
          <template v-else>
            <template v-for="(item, index) in message.content" :key="'bar-' + index">
              <div v-if="item.type === 'text' && item.reasoning && item.reasoning.length > 0" class="reasoning-section">
                <div class="reasoning-handle" @click="toggleReasoning">
                  <div class="handle-bar"></div>
                  <span class="handle-text">{{ reasoningCollapsed ? '思考过程' : '思考详情' }}</span>
                </div>
                <div v-show="!reasoningCollapsed" class="reasoning-content">
                  <pre>{{ item.reasoning }}</pre>
                </div>
              </div>
              <div v-if="item.type === 'text' && item.recalling && item.recalling.length > 0" class="recalling-section">
                <div class="recalling-handle" @click="toggleRecalling">
                  <div class="recalling-bar"></div>
                  <span class="recalling-text">{{ recallingCollapsed ? (message.isLoading ? '正在回忆...' : '回忆完毕') : '回忆详情' }}</span>
                </div>
                <div v-show="!recallingCollapsed" class="recalling-content">
                  <pre>{{ item.recalling }}</pre>
                </div>
              </div>
            </template>
            <template v-for="(item, index) in message.content" :key="'content-' + index">
              <div v-if="item.type === 'text' && item.text !== undefined" class="text-content md-content" v-html="renderMarkdown(item.text || '')" @click="handleMdClick"></div>
              <img
                v-else-if="item.type === 'image' && item.attachmentId && imageUrls[item.attachmentId]"
                :src="imageUrls[item.attachmentId]"
                class="image-content"
                alt="图片"
              />
              <div v-else-if="item.type === 'code'" class="code-content">
                <pre><code>{{ item.text }}</code></pre>
              </div>
              <div v-else-if="item.type === 'file'" class="file-content">
                <span class="file-icon">📎</span>
                <span class="file-name">{{ item.filename }}</span>
              </div>
            </template>
            <span v-if="message.isLoading" class="cursor-blink">|</span>
          </template>
          <span v-if="showMemoryDot" class="memory-dot" title="非永久记忆"></span>
        </div>
        <div class="tool-inline" :class="{ 'tool-inline-full': !hasFormalContent }">
          <template v-for="(item, index) in message.content" :key="'tool-' + index">
            <div v-if="item.type === 'tool_call'" class="tool-call-section">
              <TaskOrchestrationBlock
                v-if="isTaskOrchestrationTool(item.toolName)"
                :toolCall="item"
                :toolResult="getToolResult(item.toolCallId)"
                :expanded="!!toolCallExpanded(item.toolCallId)"
                @toggle="toggleToolCall(item.toolCallId)"
              />
              <ToolCallBlock
                v-else
                :toolCall="item"
                :toolResult="getToolResult(item.toolCallId)"
                @toggle="toggleToolCall(item.toolCallId)"
                :expanded="!!toolCallExpanded(item.toolCallId)"
              />
            </div>
          </template>
        </div>
        <div v-if="hasFormalContent" class="message-actions">
          <button class="tts-btn" @click="handleTTS" :title="isSpeakingThis ? '停止朗读' : '朗读'">
            <svg v-if="!isSpeakingThis" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
              <polygon points="5,3 19,12 5,21"></polygon>
            </svg>
            <svg v-else class="speaking-icon" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
              <rect x="6" y="4" width="4" height="16" rx="1"></rect>
              <rect x="14" y="4" width="4" height="16" rx="1"></rect>
            </svg>
          </button>
        </div>
        <div class="message-time">
          {{ formatTime(message.timestamp) }}
        </div>
      </div>
    </template>
    <template v-else>
      <div class="message-content">
        <div class="message-bubble" :class="{ 'has-memory-dot': showMemoryDot }">
          <template v-if="message.isLoading && !hasContent">
            <span class="loading-dots">
              <span></span><span></span><span></span>
            </span>
          </template>
          <template v-else>
            <template v-for="(item, index) in message.content" :key="index">
              <span v-if="item.type === 'text'" class="text-content">{{ item.text }}</span>
              <img 
                v-else-if="item.type === 'image' && item.attachmentId && imageUrls[item.attachmentId]" 
                :src="imageUrls[item.attachmentId]" 
                class="image-content"
                alt="图片"
              />
              <div v-else-if="item.type === 'code'" class="code-content">
                <pre><code>{{ item.text }}</code></pre>
              </div>
              <div v-else-if="item.type === 'file'" class="file-content">
                <span class="file-icon">📎</span>
                <span class="file-name">{{ item.filename }}</span>
              </div>
            </template>
            <span v-if="message.isLoading" class="cursor-blink">|</span>
          </template>
          <span v-if="showMemoryDot" class="memory-dot" title="非永久记忆"></span>
        </div>
        <div class="message-time">
          {{ formatTime(message.timestamp) }}
        </div>
      </div>
      <div class="message-avatar user-avatar">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
          <circle cx="12" cy="7" r="4"></circle>
        </svg>
      </div>
    </template>
    <Teleport to="body">
      <div v-if="previewUrl" class="image-preview-overlay" @click="previewUrl = ''">
        <div class="image-preview-container" @click.stop>
          <img :src="previewUrl" class="image-preview-img" alt="预览" @click="previewUrl = ''" />
          <button class="image-preview-close" @click="previewUrl = ''">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import type { Message, MessageContent } from '../stores/conversation'
import { getAttachmentUrl, revokeAttachmentUrl } from '../stores/attachment'
import { useTTS } from '../composables/useTTS'
import { renderMarkdown } from '../composables/useMarkdown'
import { readFile } from '@tauri-apps/plugin-fs'
import { fbmStore } from '../stores/fbmStore'
import ToolCallBlock from './ToolCallBlock.vue'
import TaskOrchestrationBlock from './TaskOrchestrationBlock.vue'

const props = defineProps<{
  message: Message
}>()

const showMemoryDot = computed(() => {
  return !props.message.isLoading
    && !props.message.memoryConsolidated
    && !props.message.isGreeting
    && fbmStore.isEnabled()
})

const imageUrls = ref<Record<string, string>>({})
const previewUrl = ref('')
const reasoningCollapsedManual = ref(true)
const expandedToolCalls = ref<Set<string>>(new Set())

const { speak, stop, onStop, offStop, onStart, offStart } = useTTS()
const isSpeakingThis = ref(false)

const resetSpeaking = () => { isSpeakingThis.value = false }
const handleStart = (messageId: string) => {
  isSpeakingThis.value = messageId === props.message.id
}
onStop(resetSpeaking)
onStart(handleStart)
onUnmounted(() => {
  offStop(resetSpeaking)
  offStart(handleStart)
})

const hasContent = computed(() => {
  return props.message.content.some(c =>
    c.text || c.attachmentId ||
    (c.type === 'text' && c.reasoning !== undefined && c.reasoning.length > 0)
  )
})

const hasReasoning = computed(() => {
  return props.message.content.some(c => c.type === 'text' && c.reasoning && c.reasoning.length > 0)
})

const hasFormalContent = computed(() => {
  return props.message.content.some(c => c.type === 'text' && c.text && c.text.length > 0)
})

const hasDisplayableContent = computed(() => {
  return hasFormalContent.value || hasReasoning.value || hasRecalling.value
})

const reasoningCollapsed = computed(() => {
  if (!hasReasoning.value) return false
  if (!props.message.isLoading) return reasoningCollapsedManual.value
  if (!hasFormalContent.value) return false
  return true
})

const toggleReasoning = () => {
  if (!props.message.isLoading && hasReasoning.value) {
    reasoningCollapsedManual.value = !reasoningCollapsedManual.value
  }
}

const recallingCollapsedManual = ref(true)

const hasRecalling = computed(() => {
  return props.message.content.some(c => c.type === 'text' && c.recalling && c.recalling.length > 0)
})

const recallingCollapsed = computed(() => {
  if (!hasRecalling.value) return false
  if (!props.message.isLoading) return recallingCollapsedManual.value
  if (!hasFormalContent.value) return false
  return true
})

const toggleRecalling = () => {
  if (!props.message.isLoading && hasRecalling.value) {
    recallingCollapsedManual.value = !recallingCollapsedManual.value
  }
}

const toggleToolCall = (toolCallId?: string) => {
  if (!toolCallId) return
  if (expandedToolCalls.value.has(toolCallId)) {
    expandedToolCalls.value.delete(toolCallId)
  } else {
    expandedToolCalls.value.add(toolCallId)
  }
}

const toolCallExpanded = (toolCallId?: string) => {
  return toolCallId && expandedToolCalls.value.has(toolCallId)
}

const getToolResult = (toolCallId?: string): MessageContent | undefined => {
  const toolCallIndex = props.message.content.findIndex(
    item => item.type === 'tool_call' && (item as MessageContent).toolCallId === toolCallId
  )
  if (toolCallIndex === -1) return undefined
  return props.message.content.find(
    (item, index) => index > toolCallIndex && item.type === 'tool_result'
  )
}

const isTaskOrchestrationTool = (toolName?: string): boolean => {
  if (!toolName) return false
  const orchestrationTools = ['task_manager', 'todo_write']
  return orchestrationTools.includes(toolName)
}

const formatTime = (timestamp: string): string => {
  return new Date(timestamp).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit'
  })
}

const handleTTS = () => {
  if (isSpeakingThis.value) {
    stop()
    isSpeakingThis.value = false
    return
  }
  const text = props.message.content
    .filter(c => c.type === 'text' && c.text)
    .map(c => c.text || '')
    .join('')
  if (text) {
    speak(text, props.message.id)
  }
}

watch(() => props.message.content, async (content) => {
  for (const item of content) {
    if (item.type === 'image' && item.attachmentId && !imageUrls.value[item.attachmentId]) {
      const url = await getAttachmentUrl(item.attachmentId)
      if (url) {
        imageUrls.value[item.attachmentId] = url
      }
    }
    if (item.type === 'text' && item.reasoning) {
      void item.reasoning
    }
  }
}, { immediate: true, deep: true })

const localBlobUrls = new Map<string, string>()

async function loadLocalImages() {
  await nextTick()
  const container = document.querySelector(`[data-message-id="${props.message.id}"]`)
  if (!container) return
  const localImages = container.querySelectorAll<HTMLImageElement>('img.md-image-local')
  for (const img of localImages) {
    const filePath = img.getAttribute('data-file-path')
    if (!filePath || img.src) continue
    if (localBlobUrls.has(filePath)) {
      img.src = localBlobUrls.get(filePath)!
      continue
    }
    try {
      const data = await readFile(filePath)
      const ext = filePath.split('.').pop()?.toLowerCase() || 'png'
      const mimeMap: Record<string, string> = {
        png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg',
        gif: 'image/gif', webp: 'image/webp', svg: 'image/svg+xml',
        bmp: 'image/bmp', ico: 'image/x-icon', avif: 'image/avif'
      }
      const mime = mimeMap[ext] || 'image/png'
      const blob = new Blob([data], { type: mime })
      const url = URL.createObjectURL(blob)
      localBlobUrls.set(filePath, url)
      img.src = url
    } catch {
      img.alt = `[无法加载图片: ${filePath}]`
    }
  }
}

watch(() => props.message.content, () => {
  void loadLocalImages()
}, { immediate: true, deep: true })

onUnmounted(() => {
  Object.keys(imageUrls.value).forEach(id => {
    revokeAttachmentUrl(id)
  })
  localBlobUrls.forEach(url => URL.revokeObjectURL(url))
  localBlobUrls.clear()
})

function handleMdClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (target.tagName === 'IMG' && target.classList.contains('md-image')) {
    const src = target.getAttribute('src')
    if (src) {
      previewUrl.value = src
    }
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && previewUrl.value) {
    previewUrl.value = ''
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.message-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  max-width: 85%;
  min-width: 0;
  animation: messageIn 0.3s ease-out;
}

@keyframes messageIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.message-item.assistant {
  align-self: flex-start;
}

.message-item.user {
  align-self: flex-end;
}

.message-avatar {
  width: 40px;
  height: 40px;
  border-radius: 14px;
  background: var(--color-primary-gradient);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-inverse);
  flex-shrink: 0;
  box-shadow: 0 2px 8px var(--color-shadow-primary-hover);
}

.message-avatar.user-avatar {
  background: var(--color-primary-gradient);
  box-shadow: 0 2px 8px var(--color-shadow-primary-hover);
}

.message-content {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
  max-width: 100%;
}

.message-item.user .message-content {
  align-items: flex-end;
}

.tool-inline {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 4px;
}

.tool-inline-full {
  margin-top: 0;
}

.message-bubble {
  padding: 14px 18px;
  border-radius: 18px;
  font-size: 14px;
  line-height: 1.6;
  max-width: 100%;
  overflow-wrap: break-word;
  word-break: break-word;
  overflow: hidden;
}

.message-item.assistant .message-bubble {
  background: var(--color-surface-card);
  color: var(--color-text-primary);
  box-shadow: 0 2px 12px var(--color-shadow-alpha-08);
  border-top-left-radius: 4px;
}

.message-item.user .message-bubble {
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
  box-shadow: 0 2px 12px var(--color-shadow-primary);
  border-top-right-radius: 4px;
  white-space: pre-wrap;
}

.message-bubble.has-memory-dot {
  position: relative;
}

.memory-dot {
  position: absolute;
  top: 5px;
  width: 6px;
  height: 6px;
  background: var(--color-accent-orange, #f59e0b);
  border-radius: 50%;
  cursor: help;
  z-index: 1;
}

.message-item.user .memory-dot {
  right: 5px;
}

.message-item.assistant .memory-dot {
  left: 5px;
}

.message-time {
  font-size: 11px;
  color: var(--color-text-muted);
  padding: 0 4px;
}

.cursor-blink {
  animation: blink 0.7s infinite;
  color: var(--color-primary);
  font-weight: 300;
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

.loading-dots {
  display: inline-flex;
  gap: 4px;
}

.loading-dots span {
  width: 6px;
  height: 6px;
  background: var(--color-primary);
  border-radius: 50%;
  animation: dotPulse 1.4s infinite ease-in-out both;
}

.loading-dots span:nth-child(1) {
  animation-delay: -0.32s;
}

.loading-dots span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes dotPulse {
  0%, 80%, 100% {
    transform: scale(0.6);
    opacity: 0.5;
  }
  40% {
    transform: scale(1);
    opacity: 1;
  }
}

.text-content {
  word-break: break-word;
  overflow-wrap: break-word;
}

.md-content {
  line-height: 1.7;
  color: var(--color-text-primary);
}

.md-content :deep(> *:first-child) {
  margin-top: 0 !important;
}

.md-content :deep(> *:last-child) {
  margin-bottom: 0 !important;
}

.md-content :deep(p) {
  margin: 0.6em 0;
}

.md-content :deep(h1), .md-content :deep(h2), .md-content :deep(h3),
.md-content :deep(h4), .md-content :deep(h5), .md-content :deep(h6) {
  margin: 1.2em 0 0.6em;
  font-weight: 700;
  line-height: 1.35;
  padding-left: 0.8em;
  border-left: 3px solid transparent;
}

.md-content :deep(h1) {
  font-size: 1.5em;
  color: var(--color-accent-orange);
  border-left-color: var(--color-primary);
  padding-bottom: 0.4em;
  border-bottom: 2px solid var(--color-primary-alpha-15);
}

.md-content :deep(h2) {
  font-size: 1.3em;
  color: var(--color-primary-hover);
  border-left-color: var(--color-primary-light);
}

.md-content :deep(h3) {
  font-size: 1.15em;
  color: var(--color-text-secondary);
  border-left-color: var(--color-shadow-primary-strong);
}

.md-content :deep(h4) {
  font-size: 1.05em;
  color: var(--color-text-secondary);
  font-weight: 600;
  border-left-color: var(--color-shadow-primary-hover);
}

.md-content :deep(h5) {
  font-size: 1em;
  color: var(--color-text-secondary);
  font-weight: 600;
  border-left-color: var(--color-primary-alpha-20);
}

.md-content :deep(h6) {
  font-size: 0.95em;
  color: var(--color-text-secondary);
  font-weight: 600;
  border-left-color: var(--color-primary-alpha-10);
}

.md-content :deep(strong) { font-weight: 700; }
.md-content :deep(em) { font-style: italic; }
.md-content :deep(del) { text-decoration: line-through; opacity: 0.6; }

.md-content :deep(a) {
  color: var(--color-primary);
  text-decoration: underline;
  text-underline-offset: 2px;
}

.md-content :deep(a:hover) {
  color: var(--color-primary-hover);
}

.md-content :deep(ul), .md-content :deep(ol) {
  margin: 0.6em 0;
  padding-left: 1.8em;
}

.md-content :deep(ul) { list-style-type: disc; }
.md-content :deep(ol) { list-style-type: decimal; }

.md-content :deep(li) {
  margin: 0.25em 0;
  line-height: 1.7;
}

.md-content :deep(li > p) {
  margin: 0;
}

.md-content :deep(li > ul),
.md-content :deep(li > ol) {
  margin: 0.2em 0;
}

.md-content :deep(ul ul) { list-style-type: circle; }
.md-content :deep(ul ul ul) { list-style-type: square; }

.md-content :deep(code) {
  background: var(--color-primary-alpha-08);
  color: var(--color-accent-dark-red);
  padding: 0.15em 0.4em;
  border-radius: 3px;
  font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
  font-size: 0.88em;
}

.md-content :deep(pre) {
  margin: 0.8em 0;
}

.md-content :deep(.hljs-pre) {
  background: var(--color-code-bg);
  color: var(--color-code-text);
  border-radius: 8px;
  padding: 14px 18px;
  overflow-x: auto;
  font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  margin: 0.8em 0;
  scrollbar-width: thin;
  scrollbar-color: var(--color-scrollbar) transparent;
}

.md-content :deep(.hljs-pre code) {
  background: none;
  color: inherit;
  padding: 0;
  font-size: inherit;
}

.md-content :deep(.hljs) {
  background: transparent;
  display: block;
  overflow-x: auto;
  padding: 0;
}

.md-content :deep(.hljs-keyword),
.md-content :deep(.hljs-selector-tag),
.md-content :deep(.hljs-built_in) { color: var(--color-accent-purple); }

.md-content :deep(.hljs-string),
.md-content :deep(.hljs-title),
.md-content :deep(.hljs-literal),
.md-content :deep(.hljs-type) { color: var(--color-accent-success); }

.md-content :deep(.hljs-comment),
.md-content :deep(.hljs-quote) { color: var(--color-text-muted); font-style: italic; }

.md-content :deep(.hljs-number),
.md-content :deep(.hljs-symbol) { color: var(--color-accent-orange); }

.md-content :deep(.hljs-variable),
.md-content :deep(.hljs-params) { color: var(--color-accent-orange); }

.md-content :deep(.hljs-function) { color: var(--color-accent-info); }

.md-content :deep(.hljs-attr) { color: var(--color-accent-info); }

.md-content :deep(.hljs-tag) { color: var(--color-accent-error); }

.md-content :deep(.hljs-name) { color: var(--color-accent-info); }

.md-content :deep(.hljs-attribute) { color: var(--color-accent-success); }

.md-content :deep(.hljs-addition) { color: var(--color-accent-success); }
.md-content :deep(.hljs-deletion) { color: var(--color-accent-error); }

.md-content :deep(blockquote) {
  border-left: 3px solid var(--color-primary);
  margin: 0.8em 0;
  padding: 0.5em 1em;
  background: var(--color-primary-alpha-04);
  color: var(--color-text-secondary);
}

.md-content :deep(blockquote p) {
  margin: 0.3em 0;
}

.md-content :deep(hr) {
  border: none;
  border-top: 1px solid var(--color-border-alpha-12);
  margin: 1em 0;
}

.md-content :deep(.table-scroll) {
  overflow-x: auto;
  margin: 0.8em 0;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: thin;
  scrollbar-color: var(--color-border-alpha-15) transparent;
}

.md-content :deep(.table-scroll::-webkit-scrollbar) {
  height: 6px;
}

.md-content :deep(.table-scroll::-webkit-scrollbar-thumb) {
  background: var(--color-border-alpha-15);
  border-radius: 3px;
}

.md-content :deep(.table-scroll) table {
  border-collapse: collapse;
  width: 100%;
  min-width: 400px;
  font-size: 13px;
  margin: 0;
}

.md-content :deep(.table-scroll) th,
.md-content :deep(.table-scroll) td {
  border: 1px solid var(--color-border-alpha-10);
  padding: 8px 12px;
  text-align: left;
  white-space: nowrap;
}

.md-content :deep(.table-scroll) th {
  background: var(--color-primary-alpha-06);
  font-weight: 600;
  color: var(--color-text-primary);
  position: sticky;
  top: 0;
  z-index: 1;
}

.md-content :deep(.table-scroll) tr:nth-child(even) {
  background: var(--color-bg-alpha-02);
}

.md-content :deep(.table-scroll) tr:hover {
  background: var(--color-primary-alpha-04);
}

.md-content :deep(img) {
  max-width: 100%;
  border-radius: 8px;
  margin: 0.5em 0;
}

.md-content :deep(.md-image) {
  max-width: 320px;
  max-height: 320px;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
  border-radius: 8px;
  margin: 0.5em 0;
}

.md-content :deep(.md-image:hover) {
  transform: scale(1.02);
  box-shadow: 0 4px 16px var(--color-shadow-alpha-15);
}

.image-content {
  max-width: 300px;
  max-height: 300px;
  border-radius: 8px;
  margin-top: 8px;
  display: block;
}

.code-content {
  background: var(--color-bg-alpha-05);
  border-radius: 6px;
  padding: 8px 12px;
  margin-top: 8px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  overflow-x: auto;
}

.code-content pre {
  margin: 0;
}

.code-content code {
  font-family: inherit;
}

.file-content {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--color-bg-alpha-05);
  border-radius: 6px;
  margin-top: 8px;
}

.file-icon {
  font-size: 16px;
}

.file-name {
  font-size: 13px;
  color: inherit;
}

.reasoning-section {
  margin-bottom: 12px;
  border-radius: 10px;
  overflow: hidden;
  background: var(--color-bg-alpha-03);
  border: 1px solid var(--color-border-alpha-08);
}

.reasoning-handle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  user-select: none;
  transition: background 0.2s;
}

.reasoning-handle:hover {
  background: var(--color-bg-alpha-05);
}

.handle-bar {
  width: 32px;
  height: 4px;
  background: var(--color-border-medium);
  border-radius: 2px;
  flex-shrink: 0;
}

.handle-text {
  font-size: 12px;
  color: var(--color-text-muted);
  font-weight: 500;
}

.reasoning-content {
  padding: 8px 12px 12px;
  border-top: 1px solid var(--color-border-alpha-05);
}

.reasoning-content pre {
  margin: 0;
  font-size: 12px;
  line-height: 1.5;
  color: var(--color-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
  font-family: inherit;
}

.recalling-section {
  margin-bottom: 12px;
  border-radius: 10px;
  overflow: hidden;
  background: rgba(44, 24, 80, 0.03);
  border: 1px solid var(--color-border-alpha-12);
}

.recalling-handle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  user-select: none;
  transition: background 0.2s;
}

.recalling-handle:hover {
  background: var(--color-primary-alpha-06);
}

.recalling-bar {
  width: 32px;
  height: 4px;
  background: var(--color-accent-info);
  border-radius: 2px;
  flex-shrink: 0;
}

.recalling-text {
  font-size: 12px;
  color: var(--color-accent-info);
  font-weight: 500;
}

.recalling-content {
  padding: 8px 12px 12px;
  border-top: 1px solid var(--color-border-alpha-08);
}

.recalling-content pre {
  margin: 0;
  font-size: 12px;
  line-height: 1.5;
  color: var(--color-accent-info);
  white-space: pre-wrap;
  word-break: break-word;
  font-family: inherit;
}

.message-actions {
  display: flex;
  gap: 4px;
  padding: 0 4px;
}

.tts-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: var(--color-primary-alpha-08);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  transition: all 0.2s;
}

.tts-btn:hover {
  background: var(--color-primary-alpha-15);
  color: var(--color-primary);
}

.speaking-icon {
  animation: pulse 0.8s ease-in-out infinite alternate;
}

@keyframes pulse {
  from { opacity: 0.5; }
  to { opacity: 1; }
}

.image-preview-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  animation: fadeIn 0.2s ease-out;
}

.image-preview-container {
  position: relative;
  max-width: 90vw;
  max-height: 90vh;
}

.image-preview-img {
  max-width: 90vw;
  max-height: 85vh;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  object-fit: contain;
}

.image-preview-close {
  position: absolute;
  top: -40px;
  right: 0;
  width: 32px;
  height: 32px;
  border: none;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-inverse);
  transition: background 0.2s;
}

.image-preview-close:hover {
  background: rgba(255, 255, 255, 0.35);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
