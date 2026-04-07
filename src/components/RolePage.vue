<template>
  <div class="role-page">
    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="19" y1="12" x2="5" y2="12"></line>
            <polyline points="12,19 5,12 12,5"></polyline>
          </svg>
        </button>
        <h2>角色</h2>
      </div>
      <div class="header-actions">
        <button class="danger-btn" @click="handleResetRole" :disabled="resetting">
          <svg v-if="resetting" class="spin-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 11-6.219-8.56"></path>
          </svg>
          {{ resetting ? '重置中' : '重置角色' }}
        </button>
        <button class="save-btn" @click="saveAll" :disabled="!hasAnyChanges || saving">
          <svg v-if="saving" class="spin-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 11-6.219-8.56"></path>
          </svg>
          {{ saving ? '保存中' : '保存' }}
        </button>
      </div>
    </div>

    <div class="role-content">
      <div class="form-group">
        <label class="form-label">AI 名称</label>
        <input type="text" class="text-input" v-model="formData.fairyName" placeholder="Fairy" />
      </div>
      <div class="form-group">
        <label class="form-label">用户称呼</label>
        <input type="text" class="text-input" v-model="formData.userName" placeholder="主人" />
      </div>
      <div class="form-group">
        <label class="form-label">定位</label>
        <input type="text" class="text-input" v-model="formData.fairyPositioning" placeholder="智能助手" />
      </div>
      <div class="form-group">
        <label class="form-label">风格</label>
        <input type="text" class="text-input" v-model="formData.fairyStyle" placeholder="温柔体贴" />
      </div>
      <div class="form-group">
        <label class="form-label">AI补充设定</label>
        <input type="text" class="text-input" v-model="formData.fairySupplement" placeholder="可选的补充描述" />
      </div>
      <div class="form-group">
        <label class="form-label">AI习惯</label>
        <textarea class="textarea-input" v-model="formData.habitSupplement" placeholder="在此添加额外的行为习惯描述..." rows="3"></textarea>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { loadSettings, saveSettings } from '../stores/settings'
import { fbmStore } from '../stores/fbmStore'

defineEmits<{
  back: []
}>()

const defaults: Record<string, string> = {
  fairyName: '',
  fairyPositioning: '',
  fairyStyle: '',
  fairySupplement: '',
  userName: '',
  habitSupplement: '',
}

const formData = reactive<Record<string, string>>({ ...defaults })
const originalData = reactive<Record<string, string>>({ ...defaults })
const saving = ref(false)
const resetting = ref(false)

const hasAnyChanges = computed(() => {
  return Object.keys(defaults).some(key => formData[key] !== originalData[key])
})

onMounted(() => {
  const s = loadSettings() as unknown as Record<string, unknown>
  for (const key of Object.keys(defaults)) {
    formData[key] = (s[key] as string) ?? defaults[key]
    originalData[key] = formData[key]
  }
})

async function saveAll() {
  if (!hasAnyChanges.value) return
  saving.value = true
  try {
    const s = loadSettings() as unknown as Record<string, unknown>
    for (const key of Object.keys(defaults)) {
      s[key] = formData[key]
    }
    saveSettings(s as unknown as Partial<import('../stores/settings').ApiSettings>)
    await fbmStore.refreshCoreFiles()
    for (const key of Object.keys(defaults)) {
      originalData[key] = formData[key]
    }
  } finally {
    saving.value = false
  }
}

async function handleResetRole() {
  resetting.value = true
  try {
    await fbmStore.initCoreFiles(true)
    for (const key of Object.keys(defaults)) {
      formData[key] = defaults[key]
      originalData[key] = defaults[key]
    }
    const s = loadSettings() as unknown as Record<string, unknown>
    for (const key of Object.keys(defaults)) {
      s[key] = defaults[key]
    }
    saveSettings(s as unknown as Partial<import('../stores/settings').ApiSettings>)
  } catch (error) {
    console.error('[Role] 重置角色失败:', error)
  } finally {
    resetting.value = false
  }
}
</script>

<style scoped>
.role-page {
  width: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-surface-secondary) 50%, var(--color-surface-tertiary) 100%);
  overflow: hidden;
}

.page-header {
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

.page-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  letter-spacing: -0.02em;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.danger-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: 1px solid #e74c3c;
  background: transparent;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  color: #e74c3c;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.danger-btn:hover:not(:disabled) {
  background: #e74c3c;
  color: white;
}

.danger-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.save-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 18px;
  border: none;
  background: var(--color-primary-gradient);
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-inverse);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 8px var(--color-shadow-primary-hover);
}

.save-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px var(--color-shadow-primary-strong);
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.role-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.text-input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  background: var(--color-surface-secondary);
  color: var(--color-text-primary);
  font-size: 14px;
  outline: none;
  transition: border-color 0.25s, box-shadow 0.25s;
  box-sizing: border-box;
}

.text-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.text-input::placeholder {
  color: var(--color-text-muted);
}

.textarea-input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  background: var(--color-surface-secondary);
  color: var(--color-text-primary);
  font-size: 14px;
  line-height: 1.6;
  outline: none;
  resize: vertical;
  transition: border-color 0.25s, box-shadow 0.25s;
  box-sizing: border-box;
  font-family: inherit;
}

.textarea-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.textarea-input::placeholder {
  color: var(--color-text-muted);
}

.spin-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
