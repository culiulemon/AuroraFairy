<template>
  <div class="task-panel">
    <div class="task-header">
      <span class="task-icon">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 11l3 3L22 4"></path>
          <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path>
        </svg>
      </span>
      <span class="task-title">{{ taskName }}</span>
      <span class="task-folder">{{ folderPath }}</span>
    </div>
    <div class="task-list">
      <div
        v-for="task in tasks"
        :key="task.id"
        class="task-item"
        :class="task.status"
      >
        <span class="task-checkbox">
          <template v-if="task.status === 'completed'">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="#27AE60" stroke-width="3">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
          </template>
          <template v-else-if="task.status === 'in_progress'">
            <span class="spin-icon">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="#E67E22" stroke-width="2.5">
                <path d="M21 12a9 9 0 1 1-6.219-8.56"></path>
              </svg>
            </span>
          </template>
          <template v-else>
            <span class="empty-checkbox"></span>
          </template>
        </span>
        <span class="task-text">{{ task.id }}. {{ task.content }}</span>
        <span class="task-priority" :class="task.priority">{{ task.priority }}</span>
      </div>
    </div>
    <div v-if="summary" class="task-summary">
      <span class="summary-label">摘要</span>
      <span class="summary-text">{{ summary }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  taskMdContent: string
  folderPath: string
}>()

const parsed = computed(() => {
  const raw = props.taskMdContent
  const nameMatch = raw.match(/^#\s*任务:\s*(.+)$/m)
  const taskName = nameMatch ? nameMatch[1].trim() : '未命名任务'

  const tasks: Array<{ id: string; content: string; status: string; priority: string }> = []
  const regex = /^-\s*\[([ x~])\]\s*(\d+)\.\s*(.+)\s+\[(high|medium|low)\]$/gm
  let match: RegExpExecArray | null
  while ((match = regex.exec(raw)) !== null) {
    const status = match[1] === 'x' ? 'completed' : match[1] === '~' ? 'in_progress' : 'pending'
    tasks.push({
      id: match[2],
      content: match[3].trim(),
      status,
      priority: match[4]
    })
  }

  const summaryLines: string[] = []
  const summaryRegex = /^>\s*(.+)$/gm
  let sMatch: RegExpExecArray | null
  while ((sMatch = summaryRegex.exec(raw)) !== null) {
    summaryLines.push(sMatch[1].trim())
  }
  const summary = summaryLines.length > 0 ? summaryLines.join('\n') : undefined

  return { taskName, tasks, summary }
})

const taskName = computed(() => parsed.value.taskName)
const tasks = computed(() => parsed.value.tasks)
const summary = computed(() => parsed.value.summary)
</script>

<style scoped>
.task-panel {
  background: var(--color-bg-alpha-03);
  border: 1px solid var(--color-border-alpha-08);
  border-radius: 10px;
  overflow: hidden;
  margin-top: 8px;
}

.task-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: var(--color-bg-alpha-03);
  border-bottom: 1px solid var(--color-border-alpha-06);
}

.task-icon {
  color: var(--color-primary);
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.task-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-folder {
  font-size: 10px;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.task-list {
  padding: 6px 0;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 14px;
  transition: background 0.15s;
}

.task-item:hover {
  background: var(--color-bg-alpha-03);
}

.task-item.completed .task-text {
  text-decoration: line-through;
  color: var(--color-text-muted);
}

.task-checkbox {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.empty-checkbox {
  width: 12px;
  height: 12px;
  border: 2px solid var(--color-border-medium);
  border-radius: 50%;
  display: block;
}

.spin-icon svg {
  animation: spin 1.2s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.task-text {
  font-size: 12px;
  color: var(--color-text-primary);
  flex: 1;
  line-height: 1.4;
}

.task-priority {
  font-size: 9px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  text-transform: uppercase;
  flex-shrink: 0;
  letter-spacing: 0.3px;
}

.task-priority.high {
  background: var(--color-accent-error-alpha-10);
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

.task-summary {
  padding: 8px 14px;
  border-top: 1px solid var(--color-border-alpha-06);
  background: var(--color-bg-alpha-02);
}

.summary-label {
  font-size: 10px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  margin-right: 8px;
}

.summary-text {
  font-size: 11px;
  color: var(--color-text-secondary);
  line-height: 1.4;
}
</style>
