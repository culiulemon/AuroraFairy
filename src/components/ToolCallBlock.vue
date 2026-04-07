<template>
  <div class="tool-block">
    <div class="tool-handle" @click="$emit('toggle')">
      <div class="tool-icon">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"></path>
        </svg>
      </div>
      <span class="tool-name">{{ toolCall.toolName }}</span>
      <div class="tool-bar"></div>
      <span class="tool-status" :class="statusClass">{{ statusText }}</span>
      <span class="tool-expand-icon">{{ expanded ? '▼' : '▶' }}</span>
    </div>
    <div v-show="expanded" class="tool-content">
      <div class="tool-section">
        <div class="tool-section-title">参数</div>
        <pre class="tool-data">{{ formatToolInput(toolCall.toolInput) }}</pre>
      </div>
      <div v-if="toolResult" class="tool-section">
        <div class="tool-section-title">结果</div>
        <pre v-if="toolResult.toolResult?.data" class="tool-data">{{ toolResult.toolResult.data }}</pre>
        <pre v-if="toolResult.toolResult?.error" class="tool-error">{{ toolResult.toolResult.error.message }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { MessageContent } from '../stores/conversation'

const props = defineProps<{
  toolCall: MessageContent
  toolResult?: MessageContent
  expanded: boolean
}>()

defineEmits<{
  toggle: []
}>()

const statusClass = computed(() => {
  if (!props.toolResult) return ''
  return props.toolResult.toolResult?.success ? 'success' : 'error'
})

const statusText = computed(() => {
  if (!props.toolResult) return '执行中...'
  return props.toolResult.toolResult?.success ? '成功' : '失败'
})

const formatToolInput = (input?: Record<string, unknown>) => {
  if (!input) return '{}'
  try {
    return JSON.stringify(input, null, 2)
  } catch {
    return String(input)
  }
}
</script>

<style scoped>
.tool-block {
  background: var(--color-bg-alpha-06);
  border-radius: 6px;
  overflow: hidden;
}

.tool-handle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  cursor: pointer;
  user-select: none;
  transition: background 0.2s;
}

.tool-handle:hover {
  background: var(--color-bg-alpha-05);
}

.tool-icon {
  width: 20px;
  height: 20px;
  border-radius: 6px;
  background: var(--color-primary-gradient);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-inverse);
  flex-shrink: 0;
}

.tool-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.tool-bar {
  flex: 1;
  height: 4px;
  background: var(--color-border-medium);
  border-radius: 2px;
}

.tool-status {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 4px;
  flex-shrink: 0;
}

.tool-status.success {
  background: var(--color-accent-success-alpha-10);
  color: var(--color-accent-success);
}

.tool-status.error {
  background: var(--color-accent-error-alpha-10);
  color: var(--color-accent-error);
}

.tool-status:not(.success):not(.error) {
  background: var(--color-bg-alpha-08);
  color: var(--color-text-muted);
}

.tool-expand-icon {
  font-size: 10px;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.tool-content {
  padding: 8px 12px 12px;
  border-top: 1px solid var(--color-border-alpha-05);
}

.tool-section {
  margin-bottom: 12px;
}

.tool-section:last-child {
  margin-bottom: 0;
}

.tool-section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-muted);
  margin-bottom: 6px;
  text-transform: uppercase;
}

.tool-data {
  margin: 0;
  font-size: 11px;
  line-height: 1.5;
  color: var(--color-text-secondary);
  white-space: pre-wrap;
  word-break: break-word;
  font-family: 'Consolas', 'Monaco', monospace;
  background: var(--color-bg-alpha-03);
  padding: 8px;
  border-radius: 4px;
}

.tool-error {
  margin: 0;
  font-size: 11px;
  line-height: 1.5;
  color: var(--color-accent-error);
  white-space: pre-wrap;
  word-break: break-word;
  font-family: 'Consolas', 'Monaco', monospace;
  background: var(--color-accent-error-alpha-05);
  padding: 8px;
  border-radius: 4px;
}
</style>
