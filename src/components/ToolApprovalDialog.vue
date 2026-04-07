<template>
  <BaseDialog
    v-model="dialogVisible"
    :close-on-overlay="false"
    :show-close="false"
    dialog-class="approval-dialog"
    @close="handleDeny"
  >
    <template #header>
      <div class="approval-header">
        <div class="warning-icon">
          <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 9v4m0 4h.01M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
          </svg>
        </div>
        <h3>路径访问授权</h3>
      </div>
    </template>

    <div class="approval-content">
      <p class="approval-desc">
        AI 工具 <span class="tool-name">{{ toolName }}</span> 尝试访问工作目录之外的路径：
      </p>
      <div class="path-box">
        <svg class="folder-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
        </svg>
        <code class="path-text">{{ targetPath }}</code>
      </div>
      <p class="approval-hint">本次授权仅在当前对话中生效，关闭应用后将失效</p>
    </div>

    <template #actions>
      <button class="action-btn cancel-btn" @click="handleDeny">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
        拒绝访问
      </button>
      <button class="action-btn approve-btn" @click="handleApprove">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="20,6 9,17 4,12"/>
        </svg>
        允许本次访问
      </button>
    </template>
  </BaseDialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import BaseDialog from './BaseDialog.vue'

const props = defineProps<{
  visible: boolean
  toolName: string
  targetPath: string
}>()

const emit = defineEmits<{
  approve: []
  deny: []
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: () => {}
})

function handleApprove() {
  emit('approve')
}

function handleDeny() {
  emit('deny')
}
</script>

<style scoped>
.approval-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.warning-icon {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  background: linear-gradient(135deg, var(--color-warning-bg) 0%, var(--color-accent-warning-alpha-20) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-accent-warning);
  flex-shrink: 0;
}

.approval-header h3 {
  margin: 0;
  font-size: 17px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.approval-content {
  padding: 0;
}

.approval-desc {
  font-size: 14px;
  color: var(--color-text-secondary);
  line-height: 1.6;
  margin: 0 0 16px 0;
}

.tool-name {
  font-weight: 600;
  color: var(--color-text-primary);
  background: var(--color-surface-secondary);
  padding: 2px 8px;
  border-radius: 6px;
}

.path-box {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 14px 16px;
  margin-bottom: 12px;
}

.folder-icon {
  color: var(--color-accent-warning);
  flex-shrink: 0;
  margin-top: 2px;
}

.path-text {
  font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--color-text-secondary);
  word-break: break-all;
  line-height: 1.5;
}

.approval-hint {
  font-size: 12px;
  color: var(--color-text-muted);
  margin: 0;
}

.action-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 14px 20px;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.cancel-btn {
  border: 1px solid var(--color-border);
  background: var(--color-surface-card);
  color: var(--color-text-secondary);
}

.cancel-btn:hover {
  border-color: var(--color-text-muted);
  background: var(--color-surface-secondary);
  transform: translateY(-1px);
}

.approve-btn {
  border: none;
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
  box-shadow: 0 4px 14px var(--color-shadow-primary-hover);
}

.approve-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px var(--color-shadow-primary-strong);
}

.approve-btn:active {
  transform: translateY(0);
}
</style>
