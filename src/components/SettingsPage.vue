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
        <h2>API 设置</h2>
      </div>
      <button class="add-btn" @click="handleAddProvider">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
        添加提供商
      </button>
    </div>

    <div class="settings-content">
      <div class="providers-list" v-if="providers.length > 0">
        <div 
          v-for="provider in providers" 
          :key="provider.id"
          class="provider-item"
          :class="{ default: provider.id === defaultProviderId }"
        >
          <div class="provider-info">
            <div class="provider-name">
              {{ provider.displayName }}
              <span v-if="provider.id === defaultProviderId" class="default-badge">默认</span>
            </div>
            <div class="provider-id">{{ provider.id }}</div>
            <div class="provider-url">{{ provider.baseUrl }}</div>
            <div class="provider-model" v-if="provider.model">{{ provider.model }}</div>
          </div>
          <div class="provider-actions">
            <button 
              class="action-btn test"
              :class="{
                'testing': connectionStatus[provider.id] === 'testing',
                'success': connectionStatus[provider.id] === 'success',
                'error': connectionStatus[provider.id] === 'error'
              }"
              :disabled="connectionStatus[provider.id] === 'testing'"
              @click="testConnection(provider)"
            >
              <svg v-if="connectionStatus[provider.id] !== 'testing'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                <polyline points="22 4 12 14.01 9 11.01"></polyline>
              </svg>
              <svg v-else class="spin-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 12a9 9 0 1 1-6.219-8.56"></path>
              </svg>
              <span v-if="connectionStatus[provider.id] === 'testing'">测试中</span>
              <span v-else-if="connectionStatus[provider.id] === 'success'">通过</span>
              <span v-else-if="connectionStatus[provider.id] === 'error'">失败</span>
              <span v-else>测试</span>
            </button>
            <button 
              v-if="provider.id !== defaultProviderId"
              class="action-btn set-default" 
              @click="handleSetDefault(provider.id)"
            >
              设为默认
            </button>
            <button class="action-btn edit" @click="handleEditProvider(provider)">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
              </svg>
            </button>
            <button class="action-btn delete" @click="handleDeleteProvider(provider.id)">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3,6 5,6 21,6"></polyline>
                <path d="M19,6v14a2,2,0,0,1-2,2H7a2,2,0,0,1-2-2V6m3,0V4a2,2,0,0,1,2-2h4a2,2,0,0,1,2,2v2"></path>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <div class="empty-state" v-else>
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M12 2L2 7l10 5 10-5-10-5z"></path>
            <path d="M2 17l10 5 10-5"></path>
            <path d="M2 12l10 5 10-5"></path>
          </svg>
        </div>
        <p>还没有配置 API 提供商</p>
        <span>点击"添加提供商"按钮开始配置</span>
      </div>
    </div>

    <BaseDialog v-model="showForm" :title="editingProvider ? '编辑提供商' : '添加提供商'" @close="closeForm">
      <div class="form-group">
        <label>提供商 ID</label>
        <input
          v-model="formData.id"
          :disabled="!!editingProvider"
          placeholder="例如 openai, google, anthropic"
          @input="validateProviderId"
        />
        <span class="form-hint">创建后不可更改</span>
        <span v-if="idError" class="error-text">{{ idError }}</span>
      </div>

      <div class="form-group">
        <label>显示名称</label>
        <input
          v-model="formData.displayName"
          placeholder="例如 OpenAI, Google Gemini"
        />
      </div>

      <div class="form-group">
        <label>Base URL</label>
        <input
          v-model="formData.baseUrl"
          placeholder="例如 https://api.example.com"
        />
        <span class="form-hint">基础地址即可，系统会自动拼接路径。如需自定义完整路径，以 /chat/completions 结尾即可</span>
        <div class="preset-buttons">
          <button 
            v-for="(_, key) in providerPresets" 
            :key="key"
            type="button"
            class="preset-btn"
            @click="applyPreset(key)"
          >
            {{ key }}
          </button>
        </div>
      </div>

      <div class="form-group">
        <label>API 密钥</label>
        <div class="input-wrapper">
          <input
            :type="showApiKey ? 'text' : 'password'"
            v-model="formData.apiKey"
            placeholder="输入 API 密钥 (sk-...)"
          />
          <button class="toggle-btn" @click="showApiKey = !showApiKey" type="button">
            <svg v-if="showApiKey" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
              <line x1="1" y1="1" x2="23" y2="23"></line>
            </svg>
            <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
              <circle cx="12" cy="12" r="3"></circle>
            </svg>
          </button>
        </div>
      </div>

      <div class="form-group">
        <label>模型</label>
        <input
          v-model="formData.model"
          placeholder="例如 gpt-4, claude-3-sonnet"
        />
      </div>

      <div class="form-group">
        <label>协议</label>
        <BaseSelect
          :modelValue="formData.protocol"
          :options="protocolSelectOptions"
          @update:modelValue="formData.protocol = $event as any"
        />
      </div>

      <div class="form-group toggle-group">
        <label>深度思考</label>
        <div class="toggle-wrapper">
          <input
            type="checkbox"
            v-model="formData.thinkingEnabled"
            class="toggle-input"
          />
          <div class="toggle-slider" @click="formData.thinkingEnabled = !formData.thinkingEnabled"></div>
        </div>
        <span class="toggle-hint">启用后模型将输出推理过程（仅 OpenAI/custom 协议支持）</span>
      </div>

      <div class="form-group toggle-group">
        <label>支持工具调用</label>
        <div class="toggle-wrapper">
          <input
            type="checkbox"
            v-model="formData.supportsTools"
            class="toggle-input"
          />
          <div class="toggle-slider" @click="formData.supportsTools = !formData.supportsTools"></div>
        </div>
        <span class="toggle-hint">关闭后向模型发送请求时不携带 tools 参数</span>
      </div>
      <template #actions>
        <button class="cancel-btn" @click="closeForm">取消</button>
        <button class="save-btn" @click="handleSaveProvider" :disabled="!canSave">
          {{ editingProvider ? '保存' : '创建' }}
        </button>
      </template>
    </BaseDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import BaseDialog from './BaseDialog.vue'
import BaseSelect from './BaseSelect.vue'
import type { SelectOption } from './BaseSelect.vue'
import { 
  ApiProvider, 
  loadSettings, 
  saveSettings, 
  protocolOptions, 
  providerPresets,
  validateProviderId as validateId
} from '../stores/settings'

type ConnectionStatus = 'idle' | 'testing' | 'success' | 'error'

defineEmits<{
  back: []
}>()

const providers = ref<ApiProvider[]>([])
const defaultProviderId = ref<string | null>(null)
const showForm = ref(false)
const showApiKey = ref(false)
const editingProvider = ref<ApiProvider | null>(null)
const idError = ref('')
const connectionStatus = ref<Record<string, ConnectionStatus>>({})

const protocolSelectOptions = computed<SelectOption[]>(() =>
  protocolOptions.map(o => ({ value: o.value, label: o.label }))
)

const formData = reactive<ApiProvider>({
  id: '',
  displayName: '',
  baseUrl: '',
  apiKey: '',
  model: '',
  protocol: 'openai',
  supportsTools: true
})

const canSave = computed(() => {
  return formData.id && formData.displayName && formData.apiKey && formData.protocol && !idError.value
})

onMounted(() => {
  const settings = loadSettings()
  providers.value = settings.providers
  defaultProviderId.value = settings.defaultProviderId
})

const validateProviderId = () => {
  if (!formData.id) {
    idError.value = ''
    return
  }
  if (!validateId(formData.id)) {
    idError.value = '请输入提供商 ID'
  } else if (providers.value.find(p => p.id === formData.id) && formData.id !== editingProvider.value?.id) {
    idError.value = '该 ID 已存在'
  } else {
    idError.value = ''
  }
}

const handleAddProvider = () => {
  Object.assign(formData, {
    id: '',
    displayName: '',
    baseUrl: '',
    apiKey: '',
    model: '',
    protocol: 'openai',
    thinkingEnabled: false,
    supportsTools: true
  })
  editingProvider.value = null
  idError.value = ''
  showForm.value = true
}

const handleEditProvider = (provider: ApiProvider) => {
  Object.assign(formData, { ...provider })
  editingProvider.value = provider
  idError.value = ''
  showForm.value = true
}

const closeForm = () => {
  showForm.value = false
  editingProvider.value = null
}

const applyPreset = (key: string) => {
  const preset = providerPresets[key]
  formData.baseUrl = preset.baseUrl
  formData.protocol = preset.protocol
}

const handleSaveProvider = () => {
  if (!canSave.value) return

  if (editingProvider.value) {
    const index = providers.value.findIndex(p => p.id === editingProvider.value!.id)
    if (index !== -1) {
      providers.value[index] = { ...formData }
    }
  } else {
    providers.value.push({ ...formData })
    if (providers.value.length === 1) {
      defaultProviderId.value = formData.id
    }
  }

  saveSettings({
    providers: providers.value,
    defaultProviderId: defaultProviderId.value
  })

  closeForm()
}

const handleDeleteProvider = (id: string) => {
  const index = providers.value.findIndex(p => p.id === id)
  if (index === -1) return

  providers.value.splice(index, 1)

  if (id === defaultProviderId.value) {
    defaultProviderId.value = providers.value[0]?.id || null
  }

  saveSettings({
    providers: providers.value,
    defaultProviderId: defaultProviderId.value
  })
}

const handleSetDefault = (id: string) => {
  defaultProviderId.value = id
  saveSettings({
    providers: providers.value,
    defaultProviderId: id
  })
}

async function testConnection(provider: ApiProvider) {
  connectionStatus.value[provider.id] = 'testing'

  try {
    const controller = new AbortController()
    const timeout = setTimeout(() => controller.abort(), 15000)
    const headers: Record<string, string> = {
      'Content-Type': 'application/json'
    }
    let url: string
    let body: Record<string, unknown>

    if (provider.protocol === 'anthropic') {
      headers['x-api-key'] = provider.apiKey
      headers['anthropic-version'] = '2023-06-01'
      url = provider.baseUrl.endsWith('/messages')
        ? provider.baseUrl
        : `${provider.baseUrl}/messages`
      body = {
        model: provider.model || 'claude-3-haiku-20240307',
        max_tokens: 1,
        messages: [{ role: 'user', content: 'Hi' }]
      }
    } else if (provider.baseUrl.endsWith('/embeddings')) {
      headers['Authorization'] = `Bearer ${provider.apiKey}`
      url = provider.baseUrl
      body = {
        model: provider.model || 'text-embedding-3-small',
        input: 'Hi'
      }
    } else {
      headers['Authorization'] = `Bearer ${provider.apiKey}`
      url = provider.baseUrl.endsWith('/chat/completions')
        ? provider.baseUrl
        : `${provider.baseUrl}/chat/completions`
      body = {
        model: provider.model || 'gpt-3.5-turbo',
        messages: [{ role: 'user', content: 'Hi' }],
        max_tokens: 1,
        stream: false
      }
    }

    const response = await fetch(url, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
      signal: controller.signal
    })

    clearTimeout(timeout)

    if (!response.ok) {
      const errorText = await response.text()
      throw new Error(`${response.status}: ${errorText.slice(0, 200)}`)
    }

    connectionStatus.value[provider.id] = 'success'
  } catch (error: unknown) {
    connectionStatus.value[provider.id] = 'error'
    if (error instanceof Error && error.name === 'AbortError') {
      console.error(`[Connection Test] ${provider.displayName || provider.id}: 连接超时 (15s)`)
    } else {
      console.error(`[Connection Test] ${provider.displayName || provider.id}:`, error)
    }
  }

  setTimeout(() => {
    if (connectionStatus.value[provider.id] !== 'testing') {
      connectionStatus.value[provider.id] = 'idle'
    }
  }, 5000)
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

.add-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border: none;
  background: var(--color-primary-gradient);
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-inverse);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 8px var(--color-shadow-primary-hover);
}

.add-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 16px var(--color-shadow-primary-strong);
}

.providers-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.provider-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px 20px;
  border: 1px solid var(--color-border);
  border-radius: 14px;
  background: var(--color-surface);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.provider-item.default {
  border-color: var(--color-primary);
  background: var(--color-primary-alpha-04);
}

.provider-item:hover {
  border-color: var(--color-border-light);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px var(--color-shadow-alpha-08);
}

.provider-info {
  flex: 1;
}

.provider-name {
  font-size: 15px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 4px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.default-badge {
  font-size: 10px;
  padding: 3px 8px;
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
  border-radius: 6px;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.provider-id {
  font-size: 12px;
  color: var(--color-text-muted);
  margin-bottom: 2px;
}

.provider-url {
  font-size: 12px;
  color: var(--color-text-muted);
}

.provider-model {
  font-size: 12px;
  color: var(--color-primary);
  margin-top: 2px;
}

.provider-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 8px 14px;
  border: 1px solid var(--color-border);
  background: var(--color-surface-card);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  font-size: 12px;
}

.action-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.action-btn.set-default {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-alpha-08);
}

.action-btn.set-default:hover {
  background: var(--color-primary);
  color: var(--color-text-inverse);
}

.action-btn.delete:hover {
  border-color: var(--color-accent-error);
  color: var(--color-accent-error);
  background: var(--color-danger-bg);
}

.action-btn.test {
  gap: 4px;
  min-width: 64px;
}

.action-btn.test:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.action-btn.test.testing {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-alpha-08);
}

.action-btn.test.success {
  border-color: var(--color-accent-success);
  color: var(--color-accent-success);
  background: rgba(39, 174, 96, 0.08);
}

.action-btn.test.error {
  border-color: var(--color-accent-error);
  color: var(--color-accent-error);
  background: var(--color-accent-error-alpha-08);
}

.spin-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.empty-state {
  text-align: center;
  padding: 60px 24px;
  color: var(--color-text-muted);
}

.empty-icon {
  margin-bottom: 20px;
  color: var(--color-border);
}

.empty-icon svg {
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { transform: scale(1); opacity: 0.6; }
  50% { transform: scale(1.05); opacity: 1; }
}

.empty-state p {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.empty-state span {
  font-size: 13px;
}

.cancel-btn {
  flex: 1;
  padding: 14px;
  border: 1px solid var(--color-border);
  background: var(--color-surface-card);
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.cancel-btn:hover {
  border-color: var(--color-text-muted);
  background: var(--color-surface-secondary);
}

.form-group {
  margin-bottom: 22px;
}

.form-group:last-child {
  margin-bottom: 0;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.form-group input,
.form-group select {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-surface);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.form-group input:focus,
.form-group select:focus {
  outline: none;
  border-color: var(--color-primary);
  background: var(--color-surface-card);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.form-group input:disabled {
  background: var(--color-surface-secondary);
  cursor: not-allowed;
}

.form-group input::placeholder {
  color: var(--color-text-muted);
}

.form-hint {
  display: block;
  margin-top: 6px;
  font-size: 11px;
  color: var(--color-text-muted);
}

.error-text {
  display: block;
  margin-top: 6px;
  font-size: 12px;
  color: var(--color-accent-error);
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.input-wrapper input {
  padding-right: 44px;
}

.toggle-btn {
  position: absolute;
  right: 8px;
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  transition: color 0.25s;
}

.toggle-btn:hover {
  color: var(--color-text-secondary);
}

.preset-buttons {
  display: flex;
  gap: 8px;
  margin-top: 10px;
  flex-wrap: wrap;
}

.preset-btn {
  padding: 6px 12px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.preset-btn:hover {
  border-color: var(--color-primary);
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
}

.save-btn {
  flex: 2;
  padding: 14px;
  border: none;
  border-radius: 10px;
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 16px var(--color-shadow-primary-hover);
}

.save-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px var(--color-shadow-primary-strong);
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.toggle-group {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}

.toggle-group label {
  margin-bottom: 0;
}

.toggle-wrapper {
  position: relative;
  width: 48px;
  height: 26px;
  flex-shrink: 0;
}

.toggle-input {
  opacity: 0;
  width: 0;
  height: 0;
  position: absolute;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--color-border-medium);
  transition: 0.3s;
  border-radius: 26px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 20px;
  width: 20px;
  left: 3px;
  bottom: 3px;
  background-color: var(--color-text-inverse);
  transition: 0.3s;
  border-radius: 50%;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.toggle-input:checked + .toggle-slider {
  background: var(--color-primary-gradient);
}

.toggle-input:checked + .toggle-slider:before {
  transform: translateX(22px);
}

.toggle-hint {
  width: 100%;
  font-size: 11px;
  color: var(--color-text-muted);
  margin-top: 0;
}
</style>
