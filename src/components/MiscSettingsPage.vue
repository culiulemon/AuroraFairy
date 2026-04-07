<template>
  <div class="settings-page">
    <div class="settings-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="19" y1="12" x2="5" y2="12"></line>
            <polyline points="12,19 5,12 12,5"></polyline>
          </svg>
        </button>
        <h2>杂项</h2>
      </div>
    </div>

    <div class="settings-content">
      <div class="section-card">
        <div class="section-title">全局工作目录</div>
        <div class="dir-status">
          <span class="dir-status-label">当前路径：</span>
          <span class="dir-status-badge" :class="{ custom: !!settings?.globalWorkingDir }">
            {{ settings?.globalWorkingDir ? '自定义' : '默认' }}
          </span>
        </div>
        <div class="dir-path-display">{{ loading ? '加载中...' : effectiveDir }}</div>
        <div class="dir-actions">
          <button class="action-btn primary" @click="handleSelectDir" :disabled="loading">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
            </svg>
            选择目录
          </button>
          <button class="action-btn" @click="handleOpenDir" :disabled="loading">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
              <polyline points="15 3 21 3 21 9"></polyline>
              <line x1="10" y1="14" x2="21" y2="3"></line>
            </svg>
            打开目录
          </button>
          <button
            v-if="settings?.globalWorkingDir"
            class="action-btn danger"
            @click="handleResetDir"
            :disabled="loading"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="1,4 1,10 7,10"></polyline>
              <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"></path>
            </svg>
            恢复默认
          </button>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import {
  type MiscSettings,
  loadMiscSettings,
  setWorkingDir,
  getEffectiveWorkingDir,
} from '../stores/miscSettings'
import { fbmStore } from '../stores/fbmStore'

defineEmits<{
  back: []
}>()

const settings = ref<MiscSettings | null>(null)
const loading = ref(true)

const effectiveDir = computed(() => {
  if (!settings.value) return ''
  return getEffectiveWorkingDir(settings.value)
})

onMounted(async () => {
  await reloadSettings()
})

async function reloadSettings() {
  loading.value = true
  try {
    settings.value = await loadMiscSettings()
  } catch (error) {
    console.error('[MiscSettings] 加载设置失败:', error)
  } finally {
    loading.value = false
  }
}

async function handleSelectDir() {
  try {
    const selected = await open({ directory: true, multiple: false })
    if (selected && typeof selected === 'string') {
      await setWorkingDir(selected)
      await fbmStore.setWorkingDir(selected)
      await reloadSettings()
    }
  } catch (error) {
    console.error('[MiscSettings] 选择目录失败:', error)
  }
}

async function handleOpenDir() {
  try {
    await invoke('open_folder', { path: effectiveDir.value })
  } catch (error) {
    console.error('[MiscSettings] 打开目录失败:', error)
  }
}

async function handleResetDir() {
  if (!settings.value) return
  try {
    const currentDefault = settings.value.defaultWorkingDir
    await setWorkingDir(null)
    if (currentDefault) {
      await fbmStore.setWorkingDir(currentDefault)
    }
    await reloadSettings()
  } catch (error) {
    console.error('[MiscSettings] 恢复默认目录失败:', error)
  }
}
</script>

<style scoped>
.settings-page {
  width: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-surface-secondary) 50%, var(--color-surface-tertiary) 100%);
  overflow: hidden;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 28px;
  background: var(--color-surface-card);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.back-btn {
  width: 40px;
  height: 40px;
  border: none;
  background: var(--color-surface-secondary);
  cursor: pointer;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.back-btn:hover {
  background: var(--color-surface-tertiary);
  color: var(--color-text-primary);
}

.settings-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  letter-spacing: -0.02em;
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
}

.section-card {
  padding: 24px;
  border: 1px solid var(--color-border);
  border-radius: 14px;
  background: var(--color-surface);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.section-card:hover {
  border-color: var(--color-border-light);
  box-shadow: 0 4px 12px var(--color-shadow-alpha-08);
}

.section-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 16px;
}

.setting-description {
  font-size: 13px;
  color: var(--color-text-muted);
  margin-bottom: 16px;
  line-height: 1.5;
}

.dir-status {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.dir-status-label {
  font-size: 13px;
  color: var(--color-text-muted);
}

.dir-status-badge {
  font-size: 11px;
  padding: 3px 8px;
  background: var(--color-border);
  color: var(--color-text-secondary);
  border-radius: 6px;
  font-weight: 700;
}

.dir-status-badge.custom {
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
}

.dir-path-display {
  font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--color-text-primary);
  background: var(--color-surface-tertiary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 12px 16px;
  word-break: break-all;
  line-height: 1.5;
  margin-bottom: 20px;
}

.dir-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border: 1px solid var(--color-border);
  background: var(--color-surface-card);
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.action-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.primary {
  border: none;
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
  box-shadow: 0 2px 8px var(--color-shadow-primary-hover);
}

.action-btn.primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 16px var(--color-shadow-primary-strong);
  color: var(--color-text-inverse);
}

.action-btn.danger:hover {
  border-color: var(--color-accent-error);
  color: var(--color-accent-error);
  background: var(--color-danger-bg);
}
</style>
