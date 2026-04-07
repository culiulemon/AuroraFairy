<template>
  <div class="task-orchestration-block">
    <div class="orchestration-header" @click="toggleExpand">
      <div class="orchestration-icon">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 11l3 3L22 4"></path>
          <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path>
        </svg>
      </div>
      <div class="orchestration-title-section">
        <span class="orchestration-title">{{ orchestrationTitle }}</span>
        <span class="orchestration-badge" :class="actionType">{{ actionLabel }}</span>
      </div>
      <div class="orchestration-progress">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
        </div>
        <span class="progress-text">{{ completedCount }}/{{ totalCount }}</span>
      </div>
      <div class="expand-indicator">
        <svg :class="{ rotated: expanded }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </div>
    </div>

    <div v-show="expanded" class="orchestration-body">
      <div class="task-list">
        <div
          v-for="task in parsedTasks"
          :key="task.id"
          class="task-item"
          :class="{ 'executing': task.isExecuting, 'completed': task.status === 'completed', 'pending': task.status === 'pending' }"
        >
          <div class="task-status-icon">
            <template v-if="task.status === 'completed'">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="#27AE60" stroke-width="3">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
            </template>
            <template v-else-if="task.status === 'in_progress' || task.isExecuting">
              <svg class="spin-animation" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="#E67E22" stroke-width="2.5">
                <path d="M21 12a9 9 0 1 1-6.219-8.56"></path>
              </svg>
            </template>
            <template v-else>
              <div class="pending-dot"></div>
            </template>
          </div>
          <div class="task-content">
            <span class="task-id">{{ task.id }}.</span>
            <span class="task-text">{{ task.content }}</span>
          </div>
          <span v-if="task.priority" class="task-priority" :class="task.priority">{{ task.priority }}</span>
        </div>
      </div>

      <div v-if="dispatchInfo" class="dispatch-info">
        <div class="dispatch-header">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
            <circle cx="9" cy="7" r="4"></circle>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
          </svg>
          <span>子任务委派</span>
        </div>
        <div class="dispatch-content">{{ dispatchInfo }}</div>
      </div>

      <div v-if="summary" class="task-summary">
        <div class="summary-icon">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="16" x2="12" y2="12"></line>
            <line x1="12" y1="8" x2="12.01" y2="8"></line>
          </svg>
        </div>
        <span class="summary-text">{{ summary }}</span>
      </div>

      <div v-if="hasRawDetails" class="raw-details">
        <div class="detail-tabs">
          <button
            v-for="tab in ['参数', '结果']"
            :key="tab"
            :class="['detail-tab', { active: activeDetailTab === tab }]"
            @click="activeDetailTab = tab"
          >
            {{ tab }}
          </button>
        </div>
        <div class="detail-content">
          <pre v-if="activeDetailTab === '参数'">{{ formatToolInput(toolCall.toolInput) }}</pre>
          <pre v-else-if="toolResult">{{ formatToolResult(toolResult) }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { MessageContent } from '../stores/conversation'

interface ParsedTask {
  id: string
  content: string
  status: 'pending' | 'in_progress' | 'completed'
  priority?: 'high' | 'medium' | 'low'
  isExecuting?: boolean
}

const props = defineProps<{
  toolCall: MessageContent
  toolResult?: MessageContent
  expanded?: boolean
}>()

const emit = defineEmits<{
  toggle: []
}>()

const expanded = ref(props.expanded ?? false)
const activeDetailTab = ref<string>('参数')

const toggleExpand = () => {
  expanded.value = !expanded.value
  if (expanded.value) {
    emit('toggle')
  }
}

const actionType = computed(() => {
  const action = props.toolCall.toolInput?.action as string
  return action || 'unknown'
})

const actionLabel = computed(() => {
  const action = actionType.value
  const labels: Record<string, string> = {
    'create_todo': '创建任务',
    'update_todo': '更新任务',
    'dispatch': '委派子任务',
    'unknown': '未知操作'
  }
  return labels[action] || action
})

const orchestrationTitle = computed(() => {
  const action = actionType.value
  if (action === 'create_todo') {
    return (props.toolCall.toolInput?.task_name as string) || '新建任务'
  }
  if (action === 'dispatch') {
    const desc = props.toolCall.toolInput?.description as string
    return desc ? desc.slice(0, 40) + (desc.length > 40 ? '...' : '') : '子任务'
  }
  if (action === 'update_todo') {
    const taskId = props.toolCall.toolInput?.task_id as string
    return `更新任务 #${taskId}`
  }
  return '任务编排'
})

const parsedTasks = computed((): ParsedTask[] => {
  const action = actionType.value
  if (action === 'create_todo') {
    const todos = props.toolCall.toolInput?.todos as Array<{
      id: string
      content: string
      status: string
      priority?: string
    }>
    if (Array.isArray(todos)) {
      return todos.map(t => ({
        id: t.id,
        content: t.content,
        status: t.status as 'pending' | 'in_progress' | 'completed',
        priority: t.priority as 'high' | 'medium' | 'low'
      }))
    }
  }

  if (action === 'update_todo' && props.toolResult) {
    const resultData = extractResultData(props.toolResult)
    if (resultData?.rawContent) {
      return parseTasksFromMd(resultData.rawContent)
    }
  }

  if (props.toolResult && !props.toolResult.toolResult?.success) {
    return [{
      id: '1',
      content: '任务执行失败',
      status: 'pending' as const,
      priority: 'high'
    }]
  }

  return []
})

const parseTasksFromMd = (md: string): ParsedTask[] => {
  const tasks: ParsedTask[] = []
  const regex = /^-\s*\[([ x~])\]\s*(\d+)\.\s*(.+?)\s+\[(high|medium|low)\]$/gm
  let match: RegExpExecArray | null
  while ((match = regex.exec(md)) !== null) {
    const status = match[1] === 'x' ? 'completed' : match[1] === '~' ? 'in_progress' : 'pending'
    tasks.push({
      id: match[2],
      content: match[3].trim(),
      status,
      priority: match[4] as 'high' | 'medium' | 'low'
    })
  }
  return tasks
}

const completedCount = computed(() => {
  return parsedTasks.value.filter(t => t.status === 'completed').length
})

const totalCount = computed(() => {
  return parsedTasks.value.length || (actionType.value === 'dispatch' ? 1 : 0)
})

const progressPercent = computed(() => {
  if (totalCount.value === 0) return 0
  return Math.round((completedCount.value / totalCount.value) * 100)
})

const dispatchInfo = computed(() => {
  if (actionType.value !== 'dispatch') return null
  const desc = props.toolCall.toolInput?.description as string
  return desc || null
})

const summary = computed(() => {
  if (!props.toolResult) return null
  const resultData = extractResultData(props.toolResult)
  if (resultData?.summary) {
    return resultData.summary
  }
  if (!props.toolResult.toolResult?.success && props.toolResult.toolResult?.error?.message) {
    return props.toolResult.toolResult.error.message
  }
  return null
})

const hasRawDetails = computed(() => {
  return props.toolResult || Object.keys(props.toolCall.toolInput || {}).length > 0
})

const extractResultData = (item: MessageContent): any => {
  try {
    const data = item.toolResult?.data
    if (typeof data === 'string') {
      return JSON.parse(data)
    }
    return data
  } catch {
    return null
  }
}

const formatToolInput = (input?: Record<string, unknown>) => {
  if (!input) return '{}'
  try {
    return JSON.stringify(input, null, 2)
  } catch {
    return String(input)
  }
}

const formatToolResult = (result: MessageContent) => {
  if (!result.toolResult) return ''
  if (result.toolResult.error) {
    return `错误: ${result.toolResult.error.message}`
  }
  if (result.toolResult.data) {
    const data = typeof result.toolResult.data === 'string'
      ? result.toolResult.data
      : JSON.stringify(result.toolResult.data, null, 2)
    return data
  }
  return ''
}
</script>

<style scoped>
.task-orchestration-block {
  background: linear-gradient(135deg, var(--color-primary-alpha-08) 0%, var(--color-accent-warning-alpha-05) 100%);
  border: 1px solid var(--color-primary-alpha-20);
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.25s ease;
}

.task-orchestration-block:hover {
  border-color: var(--color-primary-alpha-35);
  box-shadow: 0 2px 12px var(--color-primary-alpha-10);
}

.orchestration-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  cursor: pointer;
  transition: background 0.2s;
}

.orchestration-header:hover {
  background: var(--color-primary-alpha-05);
}

.orchestration-icon {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  background: var(--color-primary-gradient);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-inverse);
  flex-shrink: 0;
  box-shadow: 0 2px 8px var(--color-shadow-primary);
}

.orchestration-title-section {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.orchestration-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.orchestration-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 4px;
  width: fit-content;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.orchestration-badge.create_todo {
  background: var(--color-accent-success-alpha-15);
  color: var(--color-accent-success);
}

.orchestration-badge.update_todo {
  background: var(--color-accent-info-alpha-15);
  color: var(--color-accent-info);
}

.orchestration-badge.dispatch {
  background: var(--color-accent-purple-alpha-15);
  color: var(--color-accent-purple);
}

.orchestration-badge.unknown {
  background: var(--color-bg-alpha-08);
  color: var(--color-text-muted);
}

.orchestration-progress {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.progress-bar {
  width: 60px;
  height: 6px;
  background: var(--color-border-alpha-10);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary-gradient-horizontal);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  min-width: 32px;
}

.expand-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  transition: transform 0.2s;
}

.expand-indicator svg.rotated {
  transform: rotate(180deg);
}

.orchestration-body {
  border-top: 1px solid var(--color-primary-alpha-15);
  padding: 8px 0;
}

.task-list {
  padding: 4px 0;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  transition: all 0.2s;
  border-left: 3px solid transparent;
}

.task-item:hover {
  background: var(--color-primary-alpha-05);
}

.task-item.executing {
  background: var(--color-primary-alpha-08);
  border-left-color: var(--color-primary);
}

.task-item.completed .task-text {
  text-decoration: line-through;
  color: var(--color-text-muted);
}

.task-status-icon {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.spin-animation {
  animation: spin 1.2s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.pending-dot {
  width: 8px;
  height: 8px;
  border: 2px solid var(--color-border-medium);
  border-radius: 50%;
}

.task-content {
  flex: 1;
  display: flex;
  gap: 4px;
  min-width: 0;
}

.task-id {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.task-text {
  font-size: 12px;
  color: var(--color-text-primary);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-priority {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
  text-transform: uppercase;
  flex-shrink: 0;
  letter-spacing: 0.3px;
}

.task-priority.high {
  background: var(--color-accent-error-alpha-15);
  color: var(--color-accent-error);
}

.task-priority.medium {
  background: var(--color-accent-warning-alpha-10);
  color: var(--color-accent-warning);
}

.task-priority.low {
  background: var(--color-bg-alpha-08);
  color: var(--color-text-muted);
}

.dispatch-info {
  margin: 8px 14px;
  padding: 10px 12px;
  background: var(--color-accent-purple-alpha-08);
  border: 1px solid var(--color-accent-purple-alpha-15);
  border-radius: 8px;
}

.dispatch-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
  color: var(--color-accent-purple);
  font-size: 11px;
  font-weight: 600;
}

.dispatch-content {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.task-summary {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin: 8px 14px;
  padding: 10px 12px;
  background: var(--color-accent-success-alpha-08);
  border: 1px solid var(--color-accent-success-alpha-15);
  border-radius: 8px;
}

.summary-icon {
  color: var(--color-accent-success);
  flex-shrink: 0;
  margin-top: 1px;
}

.summary-text {
  font-size: 12px;
  color: var(--color-text-primary);
  line-height: 1.4;
}

.raw-details {
  margin-top: 8px;
  border-top: 1px dashed var(--color-primary-alpha-15);
  padding-top: 8px;
}

.detail-tabs {
  display: flex;
  gap: 4px;
  padding: 0 14px;
  margin-bottom: 8px;
}

.detail-tab {
  padding: 4px 12px;
  border: none;
  background: transparent;
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.detail-tab:hover {
  background: var(--color-primary-alpha-08);
  color: var(--color-text-secondary);
}

.detail-tab.active {
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
}

.detail-content {
  padding: 0 14px 12px;
}

.detail-content pre {
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
</style>
