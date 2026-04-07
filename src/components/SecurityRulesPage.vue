<template>
  <div class="security-rules-page">
    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="19" y1="12" x2="5" y2="12"></line>
            <polyline points="12,19 5,12 12,5"></polyline>
          </svg>
        </button>
        <h2>安全规则</h2>
      </div>
      <div class="header-actions">
        <button class="action-btn add" @click="handleAdd">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          <span>添加规则</span>
        </button>
        <button class="action-btn secondary" @click="showResetDialog = true">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="1,4 1,10 7,10"></polyline>
            <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"></path>
          </svg>
          <span>恢复默认</span>
        </button>
      </div>
    </div>

    <div class="category-tabs">
      <button
        v-for="tab in categoryTabs"
        :key="tab.value"
        :class="['tab-btn', { active: activeCategory === tab.value }]"
        @click="activeCategory = tab.value"
      >
        {{ tab.label }}
        <span class="tab-count">{{ getTabCount(tab.value) }}</span>
      </button>
    </div>

    <div class="rules-list" v-if="filteredRules.length > 0">
      <div v-for="rule in filteredRules" :key="rule.id" class="rule-card">
        <div class="rule-header">
          <div class="rule-info">
            <span class="rule-pattern">{{ rule.pattern }}</span>
            <span class="rule-category-tag" :class="'tag-' + rule.category">{{ categoryLabelMap[rule.category] }}</span>
            <span v-if="rule.isBuiltIn" class="rule-built-in-tag">内置</span>
          </div>
          <div class="rule-toggle" :class="{ active: rule.enabled }" @click="handleToggle(rule)">
            <div class="toggle-thumb"></div>
          </div>
        </div>
        <div class="rule-description">{{ rule.description }}</div>
        <div v-if="!rule.isBuiltIn" class="rule-actions">
          <button class="rule-action-btn edit" @click="handleEdit(rule)" title="编辑">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
            </svg>
          </button>
          <button class="rule-action-btn delete" @click="handleDelete(rule)" title="删除">
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
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
        </svg>
      </div>
      <p>暂无安全规则</p>
      <span>点击"添加规则"创建新的安全规则</span>
    </div>

    <BaseDialog v-model="showFormDialog" :title="editingRule ? '编辑规则' : '添加规则'" @close="closeFormDialog">
      <div class="form-group">
        <label>匹配模式</label>
        <input v-model="formPattern" placeholder="例如: rm -rf *" />
      </div>
      <div class="form-group">
        <label>分类</label>
        <BaseSelect
          :modelValue="formCategory"
          :options="categoryOptions"
          @update:modelValue="formCategory = $event as any"
        />
      </div>
      <div class="form-group">
        <label>描述</label>
        <textarea v-model="formDescription" placeholder="描述该规则的用途" rows="3"></textarea>
      </div>
      <template #actions>
        <button class="cancel-btn" @click="closeFormDialog">取消</button>
        <button class="save-btn" @click="handleSave" :disabled="!formPattern.trim()">保存</button>
      </template>
    </BaseDialog>

    <BaseDialog v-model="showResetDialog" title="恢复默认" @close="showResetDialog = false">
      <p class="reset-confirm-text">确定要恢复所有安全规则为默认设置吗？自定义规则将被删除，内置规则将恢复为默认状态。</p>
      <template #actions>
        <button class="cancel-btn" @click="showResetDialog = false">取消</button>
        <button class="save-btn danger" @click="handleReset">确认恢复</button>
      </template>
    </BaseDialog>

    <BaseDialog v-model="showDeleteDialog" title="删除规则" @close="showDeleteDialog = false">
      <p class="reset-confirm-text">确定要删除规则 "{{ pendingDeleteRule?.pattern }}" 吗？此操作不可撤销。</p>
      <template #actions>
        <button class="cancel-btn" @click="showDeleteDialog = false">取消</button>
        <button class="save-btn danger" @click="confirmDelete">确认删除</button>
      </template>
    </BaseDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import BaseDialog from './BaseDialog.vue'
import BaseSelect from './BaseSelect.vue'
import type { SelectOption } from './BaseSelect.vue'
import { fairyDo } from '../agent/fairyDo'
import type { SecurityRule, SecurityRuleCategory } from '../types/security'
import {
  loadRules,
  addRule,
  updateRule,
  deleteRule,
  toggleRule,
  resetToDefaults,
  getRules
} from '../stores/securityStore'

defineEmits<{
  back: []
}>()

type CategoryFilter = 'all' | 'shell' | 'file' | 'browser'

const activeCategory = ref<CategoryFilter>('all')
const showFormDialog = ref(false)
const showResetDialog = ref(false)
const showDeleteDialog = ref(false)
const pendingDeleteRule = ref<SecurityRule | null>(null)
const editingRule = ref<SecurityRule | null>(null)
const formPattern = ref('')
const categoryOptions: SelectOption[] = [
  { value: 'shell', label: 'Shell' },
  { value: 'file', label: '文件' },
  { value: 'browser', label: '浏览器' },
]

const formCategory = ref<SecurityRuleCategory>('shell')
const formDescription = ref('')

const categoryTabs: { label: string; value: CategoryFilter }[] = [
  { label: '全部', value: 'all' },
  { label: 'Shell', value: 'shell' },
  { label: '文件', value: 'file' },
  { label: '浏览器', value: 'browser' }
]

const categoryLabelMap: Record<string, string> = {
  shell: 'Shell',
  file: '文件',
  cdp: 'CDP'
}

const rulesList = ref<SecurityRule[]>([])

const filteredRules = computed(() => {
  if (activeCategory.value === 'all') return rulesList.value
  return rulesList.value.filter(r => r.category === activeCategory.value)
})

const getTabCount = (category: CategoryFilter): number => {
  if (category === 'all') return rulesList.value.length
  return rulesList.value.filter(r => r.category === category).length
}

onMounted(async () => {
  rulesList.value = await loadRules()
})

const refreshRules = async () => {
  rulesList.value = await loadRules()
}

const syncToFairyDo = () => {
  fairyDo.setSecurityRules(getRules())
}

const handleAdd = () => {
  editingRule.value = null
  formPattern.value = ''
  formCategory.value = 'shell'
  formDescription.value = ''
  showFormDialog.value = true
}

const handleEdit = (rule: SecurityRule) => {
  editingRule.value = rule
  formPattern.value = rule.pattern
  formCategory.value = rule.category
  formDescription.value = rule.description
  showFormDialog.value = true
}

const handleSave = async () => {
  if (!formPattern.value.trim()) return
  if (editingRule.value) {
    await updateRule(editingRule.value.id, {
      pattern: formPattern.value.trim(),
      category: formCategory.value,
      description: formDescription.value.trim()
    })
  } else {
    await addRule({
      pattern: formPattern.value.trim(),
      category: formCategory.value,
      description: formDescription.value.trim(),
      enabled: true
    })
  }
  closeFormDialog()
  await refreshRules()
  syncToFairyDo()
}

const handleDelete = (rule: SecurityRule) => {
  if (rule.isBuiltIn) return
  pendingDeleteRule.value = rule
  showDeleteDialog.value = true
}

const confirmDelete = async () => {
  if (!pendingDeleteRule.value) return
  await deleteRule(pendingDeleteRule.value.id)
  await refreshRules()
  syncToFairyDo()
  showDeleteDialog.value = false
  pendingDeleteRule.value = null
}

const handleToggle = async (rule: SecurityRule) => {
  await toggleRule(rule.id)
  await refreshRules()
  syncToFairyDo()
}

const handleReset = async () => {
  await resetToDefaults()
  await refreshRules()
  syncToFairyDo()
  showResetDialog.value = false
}

const closeFormDialog = () => {
  showFormDialog.value = false
  editingRule.value = null
  formPattern.value = ''
  formCategory.value = 'shell'
  formDescription.value = ''
}
</script>

<style scoped>
.security-rules-page {
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

.category-tabs {
  display: flex;
  gap: 4px;
  padding: 12px 24px;
  background: var(--color-surface-card);
  border-bottom: 1px solid var(--color-border);
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  background: transparent;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab-btn:hover {
  background: var(--color-primary-alpha-10);
  color: var(--color-primary);
}

.tab-btn.active {
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
}

.tab-count {
  font-size: 11px;
  padding: 2px 6px;
  background: rgba(0, 0, 0, 0.08);
  border-radius: 10px;
}

.tab-btn.active .tab-count {
  background: rgba(0, 0, 0, 0.15);
}

.rules-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
  gap: 16px;
  padding: 16px 24px;
  overflow-y: auto;
  align-content: start;
}

.rule-card {
  background: var(--color-surface-card);
  border-radius: 12px;
  border: 1px solid var(--color-border);
  padding: 16px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.rule-card:hover {
  border-color: var(--color-border-light);
  background: var(--color-surface);
  box-shadow: 0 2px 8px var(--color-shadow-alpha-06);
}

.rule-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.rule-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  flex: 1;
  min-width: 0;
}

.rule-pattern {
  font-family: 'Courier New', Consolas, monospace;
  font-size: 13px;
  background: var(--color-surface-secondary);
  padding: 4px 8px;
  border-radius: 4px;
  display: inline-block;
  color: var(--color-text-primary);
  word-break: break-all;
}

.rule-category-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 600;
  white-space: nowrap;
}

.tag-shell {
  background: var(--color-accent-error-alpha-10);
  color: var(--color-accent-error);
}

.tag-file {
  background: var(--color-accent-info-alpha-10);
  color: var(--color-accent-info);
}

.tag-browser {
  background: var(--color-accent-success-alpha-10);
  color: var(--color-accent-success);
}

.tag-cdp {
  background: var(--color-surface);
  color: var(--color-accent-warning);
}

.rule-built-in-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  background: var(--color-surface-secondary);
  color: var(--color-primary);
  font-weight: 600;
  white-space: nowrap;
}

.rule-toggle {
  width: 44px;
  height: 24px;
  border-radius: 12px;
  background: var(--color-border-medium);
  cursor: pointer;
  position: relative;
  transition: background 0.25s ease;
  flex-shrink: 0;
}

.rule-toggle.active {
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

.rule-toggle.active .toggle-thumb {
  transform: translateX(20px);
}

.rule-description {
  font-size: 12px;
  color: var(--color-text-muted);
  line-height: 1.5;
  margin-bottom: 8px;
}

.rule-actions {
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
</style>
