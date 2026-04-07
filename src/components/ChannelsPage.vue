<template>
  <div class="channels-page">
    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="19" y1="12" x2="5" y2="12"></line>
            <polyline points="12,19 5,12 12,5"></polyline>
          </svg>
        </button>
        <h2>渠道管理</h2>
      </div>
      <div class="header-actions">
        <button class="action-btn add" @click="handleAdd">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          <span>添加渠道</span>
        </button>
      </div>
    </div>

    <div class="channels-list" v-if="accounts.length > 0">
      <div v-for="account in accounts" :key="account.id" class="channel-card">
        <div class="channel-header">
          <div class="channel-info">
            <div class="channel-icon"
              :style="{ background: channelRegistry.getBadge(account.channelId)?.backgroundColor ?? 'var(--color-surface-secondary)' }"
              v-html="channelRegistry.getBadge(account.channelId)?.svgContent ?? '<svg viewBox=\'0 0 24 24\' width=\'22\' height=\'22\' fill=\'currentColor\'><path d=\'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z\'/></svg>'">
            </div>
            <span class="channel-name">{{ account.displayName }}</span>
            <span class="channel-tag tag-type">{{ getChannelDisplayName(account.channelId) }}</span>
            <span class="channel-status" :class="connectionStatuses[account.id] || 'disconnected'">
              {{ getStatusText(connectionStatuses[account.id]) }}
            </span>
          </div>
          <div class="channel-toggle" :class="{ active: account.enabled }" @click="handleToggle(account)">
            <div class="toggle-thumb"></div>
          </div>
        </div>
        <div class="channel-actions">
          <button
            class="rule-action-btn"
            :class="connectionStatuses[account.id] === 'connected' ? 'disconnect' : 'connect'"
            @click.stop="toggleConnection(account)"
            :title="connectionStatuses[account.id] === 'connected' ? '断开连接' : '连接'"
          >
            <svg v-if="connectionStatuses[account.id] === 'connected'" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="1" y1="1" x2="23" y2="23"></line>
              <path d="M16.72 11.06A10.94 10.94 0 0 1 19 12.55"></path>
              <path d="M5 12.55a10.94 10.94 0 0 1 5.17-2.39"></path>
              <path d="M8.53 16.11a6 6 0 0 1 6.95 0"></path>
              <line x1="12" y1="20" x2="12.01" y2="20"></line>
            </svg>
            <svg v-else viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M5 12.55a11 11 0 0 1 14.08 0"></path>
              <path d="M8.53 16.11a6 6 0 0 1 6.95 0"></path>
              <line x1="12" y1="20" x2="12.01" y2="20"></line>
            </svg>
          </button>
          <button class="rule-action-btn edit" @click="handleEdit(account)" title="编辑">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
            </svg>
          </button>
          <button class="rule-action-btn delete" @click="handleDelete(account)" title="删除">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3,6 5,6 21,6"></polyline>
              <path d="M19,6v14a2,2 0 0 1-2,2H7a2,2 0 0 1-2-2V6m3,0V4a2,2 0 0 1 2-2h4a2,2 0 0 1 2,2v2"></path>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <div class="empty-state" v-else>
      <div class="empty-icon">
        <svg viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1">
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
        </svg>
      </div>
      <p>暂无渠道连接</p>
      <span>点击"添加渠道"配置IM连接</span>
    </div>

    <BaseDialog v-model="showFormDialog" :title="editingAccount ? '编辑渠道' : '添加渠道'" @close="closeFormDialog">
      <div class="form-group">
        <label>渠道类型</label>
        <BaseSelect
          :modelValue="formChannelId"
          :options="channelTypeOptions"
          @update:modelValue="handleChannelTypeChange"
          :disabled="!!editingAccount"
        />
      </div>
      <div class="form-group">
        <label>账户名称</label>
        <input v-model="formDisplayName" placeholder="例如: 我的飞书机器人" />
      </div>
      <div v-for="field in currentConfigFields" :key="field.key" class="form-group">
        <label>{{ field.label }}</label>
        <BaseSelect
          v-if="field.type === 'select'"
          :modelValue="formConfig[field.key] as string ?? null"
          :options="field.options?.map(o => ({ value: o.value, label: o.label })) ?? []"
          @update:modelValue="(v: string | null) => formConfig[field.key] = v ?? ''"
        />
        <div v-else-if="field.type === 'qrcode'" class="qrcode-field">
          <div class="qrcode-actions">
            <button class="qrcode-btn" @click="handleRequestQrCode" :disabled="qrCodeLoading">
              {{ qrCodeLoading ? '获取中...' : '获取二维码' }}
            </button>
            <span v-if="qrCodeStatus" class="qrcode-status" :class="qrCodeStatusClass">{{ qrCodeStatus }}</span>
          </div>
          <div v-if="qrCodeImageData" class="qrcode-preview">
            <img v-if="!qrCodeIsSvg" :src="qrCodeImageData" alt="微信扫码登录" />
            <div v-else v-html="qrCodeImageData" class="qrcode-svg-container"></div>
          </div>
        </div>
        <input v-else-if="field.type === 'password'" type="password" :placeholder="field.placeholder" v-model="formConfig[field.key]" />
        <input v-else-if="field.type === 'number'" type="number" :placeholder="field.placeholder" v-model.number="formConfig[field.key]" />
        <input v-else type="text" :placeholder="field.placeholder" v-model="formConfig[field.key]" />
      </div>
      <template #actions>
        <button class="cancel-btn" @click="closeFormDialog">取消</button>
        <button class="save-btn" @click="handleSave" :disabled="!isFormValid">保存</button>
      </template>
    </BaseDialog>

    <BaseDialog v-model="showDeleteDialog" title="删除渠道" @close="showDeleteDialog = false">
      <p class="reset-confirm-text">确定要删除渠道 "{{ pendingDeleteAccount?.displayName }}" 吗？此操作不可撤销。</p>
      <template #actions>
        <button class="cancel-btn" @click="showDeleteDialog = false">取消</button>
        <button class="save-btn danger" @click="confirmDelete">确认删除</button>
      </template>
    </BaseDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import BaseDialog from './BaseDialog.vue'
import BaseSelect from './BaseSelect.vue'
import type { SelectOption } from './BaseSelect.vue'
import type { ChannelAccount, ChannelId } from '../types/channel'
import type { ConfigFieldDefinition } from '../types/channel'
import {
  loadChannelSettings,
  addChannelAccount,
  updateChannelAccount,
  deleteChannelAccount,
  generateAccountId
} from '../stores/channelStore'
import { channelRegistry } from '../agent/channelRegistry'
import { requestQrCode, onQrCodeStatus, onQrCodeUrl } from '../agent/weixinBridge'
import { listen } from '@tauri-apps/api/event'

defineEmits<{
  back: []
}>()

const accounts = ref<ChannelAccount[]>([])
const showFormDialog = ref(false)
const showDeleteDialog = ref(false)
const pendingDeleteAccount = ref<ChannelAccount | null>(null)
const editingAccount = ref<ChannelAccount | null>(null)
const connectionStatuses = ref<Record<string, string>>({})

const formChannelId = ref<ChannelId>('feishu')
const formDisplayName = ref('')
const formConfig = ref<Record<string, unknown>>({})

const qrCodeImageData = ref('')
const qrCodeIsSvg = ref(false)
const qrCodeLoading = ref(false)
const qrCodeStatus = ref('')
const qrCodeAccountId = ref('')
const qrCodeStatusClass = ref('')
let qrCleanup: (() => void) | null = null

const channelTypeOptions = computed<SelectOption[]>(() =>
  channelRegistry.getAll().map(b => ({ value: b.channelId, label: b.displayName }))
)

const currentConfigFields = computed<ConfigFieldDefinition[]>(() => {
  const bridge = channelRegistry.get(formChannelId.value)
  if (!bridge) return []
  return bridge.getConfigFields().filter(field => {
    if (!field.condition) return true
    return formConfig.value[field.condition.field] === field.condition.value
  })
})

const isFormValid = computed(() => {
  if (formDisplayName.value.trim() === '') return false
  const bridge = channelRegistry.get(formChannelId.value)
  if (!bridge) return false
  const result = bridge.validateConfig(formConfig.value)
  return result.valid
})

const refreshAccounts = () => {
  accounts.value = loadChannelSettings().accounts
}

let statusUnlisten: (() => void) | null = null

onMounted(async () => {
  refreshAccounts()
  statusUnlisten = await channelRegistry.onStatusChange((accountId, status) => {
    connectionStatuses.value[accountId] = status
  })
  for (const account of accounts.value) {
    if (account.enabled) {
      try {
        const status = await channelRegistry.getStatus(account.channelId, account.id)
        if (status) connectionStatuses.value[account.id] = status
      } catch {}
    }
  }
})

onUnmounted(() => {
  if (statusUnlisten) {
    statusUnlisten()
    statusUnlisten = null
  }
})

const initFormConfig = (channelId: ChannelId) => {
  const bridge = channelRegistry.get(channelId)
  if (!bridge) {
    formConfig.value = {}
    return
  }
  const config: Record<string, unknown> = {}
  for (const field of bridge.getConfigFields()) {
    config[field.key] = field.defaultValue ?? (field.type === 'number' ? '' : '')
  }
  formConfig.value = config
}

const handleAdd = () => {
  editingAccount.value = null
  const firstChannel = channelRegistry.getAll()[0]
  formChannelId.value = firstChannel?.channelId ?? 'feishu'
  formDisplayName.value = ''
  initFormConfig(formChannelId.value)
  showFormDialog.value = true
}

const handleEdit = (account: ChannelAccount) => {
  editingAccount.value = account
  formChannelId.value = account.channelId
  formDisplayName.value = account.displayName
  formConfig.value = { ...(account.config as unknown as Record<string, unknown>) }
  showFormDialog.value = true
}

const handleChannelTypeChange = (value: string | null) => {
  const newChannelId = (value as ChannelId) ?? 'feishu'
  if (newChannelId !== formChannelId.value) {
    formChannelId.value = newChannelId
    initFormConfig(newChannelId)
  }
}

const handleSave = () => {
  if (!isFormValid.value) return

  const config = { ...formConfig.value } as any

  if (editingAccount.value) {
    updateChannelAccount(editingAccount.value.id, {
      displayName: formDisplayName.value.trim(),
      config
    })
  } else {
    const id = qrCodeAccountId.value || generateAccountId(formChannelId.value)
    const account: ChannelAccount = {
      id,
      channelId: formChannelId.value,
      displayName: formDisplayName.value.trim(),
      enabled: true,
      config
    }
    addChannelAccount(account)
  }

  closeFormDialog()
  refreshAccounts()
}

const handleToggle = (account: ChannelAccount) => {
  updateChannelAccount(account.id, { enabled: !account.enabled })
  refreshAccounts()
}

const handleDelete = (account: ChannelAccount) => {
  pendingDeleteAccount.value = account
  showDeleteDialog.value = true
}

const confirmDelete = () => {
  if (!pendingDeleteAccount.value) return
  deleteChannelAccount(pendingDeleteAccount.value.id)
  refreshAccounts()
  showDeleteDialog.value = false
  pendingDeleteAccount.value = null
}

const closeFormDialog = () => {
  showFormDialog.value = false
  editingAccount.value = null
  qrCodeImageData.value = ''
  qrCodeIsSvg.value = false
  qrCodeStatus.value = ''
  qrCodeAccountId.value = ''
  qrCodeStatusClass.value = ''
  if (qrCleanup) {
    qrCleanup()
    qrCleanup = null
  }
}

const handleRequestQrCode = async () => {
  const accountId = editingAccount.value?.id || generateAccountId(formChannelId.value)
  qrCodeAccountId.value = accountId
  qrCodeLoading.value = true
  qrCodeImageData.value = ''
  qrCodeIsSvg.value = false
  qrCodeStatus.value = ''

  if (qrCleanup) {
    qrCleanup()
    qrCleanup = null
  }

  const handleLoginResult = (event: any) => {
    console.log('[QR] Login result:', JSON.stringify(event))
    if (event.accountId !== accountId) return
    if (event.success) {
      qrCodeStatus.value = '登录成功'
      qrCodeStatusClass.value = 'success'
      formConfig.value.botToken = event.botToken || ''
      formConfig.value.ilinkBotId = event.ilinkBotId || ''
      formConfig.value.ilinkUserId = event.ilinkUserId || ''
    } else {
      qrCodeStatus.value = event.error || '登录失败'
      qrCodeStatusClass.value = event.error?.includes('过期') ? 'error' : 'pending'
    }
  }

  const unsubStatus = onQrCodeStatus(handleLoginResult)

  const unsubDirect = await listen<any>('weixin-login-status', (event) => {
    handleLoginResult(event.payload)
  })

  const unsubUrl = onQrCodeUrl(accountId, (content) => {
    if (content.trim().startsWith('<svg') || content.trim().startsWith('<?xml')) {
      qrCodeIsSvg.value = true
      qrCodeImageData.value = content
    } else if (content.startsWith('http://') || content.startsWith('https://') || content.startsWith('data:')) {
      qrCodeIsSvg.value = false
      qrCodeImageData.value = content
    } else {
      qrCodeIsSvg.value = false
      qrCodeImageData.value = `data:image/png;base64,${content}`
    }
  })

  qrCleanup = () => {
    unsubStatus()
    unsubUrl()
    unsubDirect()
  }

  try {
    let content = await requestQrCode(accountId)
    if (content) {
      if (content.trim().startsWith('<svg') || content.trim().startsWith('<?xml')) {
        qrCodeIsSvg.value = true
        qrCodeImageData.value = content
      } else if (content.startsWith('http://') || content.startsWith('https://') || content.startsWith('data:')) {
        qrCodeIsSvg.value = false
        qrCodeImageData.value = content
      } else {
        qrCodeIsSvg.value = false
        qrCodeImageData.value = `data:image/png;base64,${content}`
      }
    }
    if (!qrCodeStatus.value) {
      qrCodeStatus.value = '等待扫码...'
      qrCodeStatusClass.value = 'pending'
    }
  } catch (err) {
    qrCodeStatus.value = `获取失败: ${err}`
    qrCodeStatusClass.value = 'error'
  } finally {
    qrCodeLoading.value = false
  }
}

const toggleConnection = async (account: ChannelAccount) => {
  if (connectionStatuses.value[account.id] === 'connected') {
    await channelRegistry.disconnectAccount(account.id, account.channelId)
    connectionStatuses.value[account.id] = 'disconnected'
  } else {
    connectionStatuses.value[account.id] = 'connecting'
    try {
      await channelRegistry.connectAccount(account)
    } catch (err) {
      console.error('[Channel] Connect failed:', err)
      connectionStatuses.value[account.id] = 'error'
      alert(`连接失败: ${err}`)
    }
  }
}

const getStatusText = (status?: string) => {
  switch (status) {
    case 'connected': return '已连接'
    case 'connecting': return '连接中...'
    case 'error': return '连接错误'
    default: return '未连接'
  }
}

const getChannelDisplayName = (channelId: string): string => {
  const bridge = channelRegistry.get(channelId)
  return bridge?.displayName ?? channelId
}
</script>

<style scoped>
.channels-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-surface-secondary) 50%, var(--color-surface-tertiary) 100%);
  overflow: hidden;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: var(--color-surface-card);
  border-bottom: 1px solid var(--color-border);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
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
  gap: 10px;
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
  background: var(--color-primary-alpha-10);
}

.action-btn.add {
  background: var(--color-primary-gradient);
  border-color: var(--color-primary);
  color: var(--color-text-inverse);
  box-shadow: 0 2px 8px var(--color-shadow-primary-hover);
}

.action-btn.add:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 16px var(--color-shadow-primary-strong);
}

.channels-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
  gap: 16px;
  padding: 16px 24px;
  overflow-y: auto;
  align-content: start;
}

.channel-card {
  background: var(--color-surface-card);
  border-radius: 12px;
  border: 1px solid var(--color-border);
  padding: 16px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.channel-card:hover {
  border-color: var(--color-border-light);
  background: var(--color-surface);
  box-shadow: 0 2px 8px var(--color-shadow-alpha-06);
}

.channel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.channel-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  flex: 1;
  min-width: 0;
}

.channel-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-primary);
  flex-shrink: 0;
}

.channel-icon :deep(svg) {
  width: 22px;
  height: 22px;
}

.channel-name {
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.channel-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 600;
  white-space: nowrap;
}

.tag-type {
  background: var(--color-accent-info-alpha-10);
  color: var(--color-accent-info);
}

.tag-mode {
  background: var(--color-accent-success-alpha-10);
  color: var(--color-accent-success);
}

.tag-domain {
  background: var(--color-surface-secondary);
  color: var(--color-primary);
}

.channel-toggle {
  width: 44px;
  height: 24px;
  border-radius: 12px;
  background: var(--color-border-medium);
  cursor: pointer;
  position: relative;
  transition: background 0.25s ease;
  flex-shrink: 0;
}

.channel-toggle.active {
  background: var(--color-primary);
}

.toggle-thumb {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--color-surface-card);
  position: absolute;
  top: 2px;
  left: 2px;
  transition: transform 0.25s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
}

.channel-toggle.active .toggle-thumb {
  transform: translateX(20px);
}

.channel-actions {
  display: flex;
  gap: 6px;
  justify-content: flex-end;
}

.rule-action-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.rule-action-btn.edit {
  background: var(--color-surface-secondary);
  color: var(--color-text-secondary);
}

.rule-action-btn.edit:hover {
  background: var(--color-primary);
  color: var(--color-text-inverse);
}

.rule-action-btn.delete {
  background: var(--color-danger-bg);
  color: var(--color-text-secondary);
}

.rule-action-btn.delete:hover {
  background: var(--color-accent-error);
  color: var(--color-text-inverse);
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  padding: 40px;
}

.empty-icon {
  margin-bottom: 20px;
  color: var(--color-border);
}

.empty-state p {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.empty-state span {
  font-size: 13px;
}

.form-group {
  margin-bottom: 20px;
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
.form-group textarea,
.form-group select {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-surface);
  transition: all 0.25s;
  font-family: inherit;
  resize: vertical;
  box-sizing: border-box;
}

.form-group input:focus,
.form-group textarea:focus,
.form-group select:focus {
  outline: none;
  border-color: var(--color-primary);
  background: var(--color-surface-card);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.form-group input::placeholder,
.form-group textarea::placeholder {
  color: var(--color-text-muted);
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
  transition: all 0.25s;
}

.cancel-btn:hover {
  border-color: var(--color-text-muted);
  background: var(--color-surface-secondary);
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
  transition: all 0.25s;
  box-shadow: 0 4px 16px var(--color-shadow-primary-hover);
}

.save-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px var(--color-shadow-primary-strong);
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.save-btn.danger {
  background: var(--color-accent-dark-red);
  box-shadow: 0 4px 16px var(--color-shadow-error);
}

.save-btn.danger:hover:not(:disabled) {
  box-shadow: 0 6px 20px var(--color-shadow-error-hover);
}

.reset-confirm-text {
  font-size: 14px;
  color: var(--color-text-secondary);
  line-height: 1.6;
  margin: 0;
}

.channel-status {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 500;
}

.channel-status.connected {
  background: rgba(76, 175, 80, 0.15);
  color: #4caf50;
}

.channel-status.connecting {
  background: rgba(255, 193, 7, 0.15);
  color: #ffc107;
}

.channel-status.disconnected {
  background: rgba(158, 158, 158, 0.15);
  color: #9e9e9e;
}

.channel-status.error {
  background: rgba(244, 67, 54, 0.15);
  color: #f44336;
}

.rule-action-btn.connect {
  color: var(--color-primary);
}

.rule-action-btn.disconnect {
  color: var(--color-accent-dark-red);
}

.qrcode-field {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.qrcode-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.qrcode-btn {
  padding: 10px 20px;
  border: 1px solid var(--color-primary);
  background: var(--color-primary-alpha-10);
  border-radius: 10px;
  color: var(--color-primary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s;
}

.qrcode-btn:hover:not(:disabled) {
  background: var(--color-primary);
  color: var(--color-text-inverse);
}

.qrcode-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.qrcode-status {
  font-size: 12px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 8px;
}

.qrcode-status.pending {
  background: rgba(255, 193, 7, 0.15);
  color: #ffc107;
}

.qrcode-status.success {
  background: rgba(76, 175, 80, 0.15);
  color: #4caf50;
}

.qrcode-status.error {
  background: rgba(244, 67, 54, 0.15);
  color: #f44336;
}

.qrcode-preview {
  display: flex;
  justify-content: center;
  padding: 16px;
  background: var(--color-surface);
  border-radius: 12px;
  border: 1px solid var(--color-border);
}

.qrcode-preview img {
  max-width: 256px;
  max-height: 256px;
  border-radius: 8px;
}

.qrcode-svg-container {
  display: flex;
  justify-content: center;
}

.qrcode-svg-container :deep(svg) {
  max-width: 256px;
  max-height: 256px;
}
</style>
