<template>
  <div class="skills-page">
    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 12H5M12 19l-7-7 7-7" />
          </svg>
        </button>
        <h2>技能管理</h2>
      </div>
      <div class="header-actions">
        <button class="action-btn refresh" @click="refreshSkills" :disabled="loading">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M1 4v6h6M23 20v-6h-6" />
            <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15" />
          </svg>
          <span>刷新</span>
        </button>
        <button class="action-btn open-folder" @click="openDir">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
          <span>打开目录</span>
        </button>
        <button class="action-btn add" @click="showInstallDialog = true">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19" />
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          <span>安装技能</span>
        </button>
      </div>
    </div>

    <div class="skills-grid" v-if="skills.length > 0">
      <div v-for="skill in skills" :key="skill.name" class="skill-card" :class="{ disabled: !skill.enabled }">
        <div class="skill-card-header">
          <span class="skill-emoji">{{ skill.emoji || '⚡' }}</span>
          <h3 class="skill-name">{{ skill.name }}</h3>
          <span class="source-tag" :class="skill.source">{{ skill.source === 'workspace' ? '工作区' : skill.source === 'user' ? '用户' : '内置' }}</span>
          <label class="toggle-switch">
            <input type="checkbox" :checked="skill.enabled" :disabled="!isEligible(skill)" @change="toggleSkill(skill.name, ($event.target as HTMLInputElement).checked)" />
            <span class="toggle-slider"></span>
          </label>
        </div>
        <p class="skill-description">{{ skill.description || '暂无描述' }}</p>
        <div class="skill-deps" v-if="skill.gatingStatus && !isEligible(skill)">
          <div class="dep-items">
            <span v-for="bin in (skill.gatingStatus.missingBins || [])" :key="'bin-'+bin" class="dep-item missing">
              <span class="dep-icon">✗</span> {{ bin }}
            </span>
            <span v-for="env in (skill.gatingStatus.missingEnv || [])" :key="'env-'+env" class="dep-item missing">
              <span class="dep-icon">✗</span> {{ env }}
            </span>
            <span v-if="skill.gatingStatus.osMismatch" class="dep-item os-warn">
              <span class="dep-icon">⚠</span> 当前操作系统不支持
            </span>
          </div>
        </div>
        <div class="skill-deps" v-else-if="skill.gatingStatus && isEligible(skill)">
          <div class="dep-items">
            <span class="dep-item ok">
              <span class="dep-icon">✓</span> 环境检查通过
            </span>
          </div>
        </div>
        <div class="skill-card-footer">
          <div class="readonly-control">
            <span class="readonly-label">{{ skill.readonly ? '只读' : '可写' }}</span>
            <label class="mini-toggle">
              <input type="checkbox" :checked="!skill.readonly" @change="toggleReadonly(skill.name, !($event.target as HTMLInputElement).checked)" />
              <span class="mini-toggle-slider"></span>
            </label>
          </div>
          <button v-if="skill.source === 'user'" class="delete-btn" @click="startDelete(skill.name)">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6" />
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            </svg>
            <span>卸载</span>
          </button>
        </div>
      </div>
    </div>

    <div class="empty-state" v-else>
      <div class="empty-icon">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
          <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
          <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
          <line x1="12" y1="22.08" x2="12" y2="12" />
        </svg>
      </div>
      <p>还没有安装技能</p>
      <span>点击"安装技能"按钮从 GitHub 安装技能包</span>
    </div>

    <div class="dialog-overlay" v-if="showInstallDialog" @click.self="showInstallDialog = false">
      <div class="dialog">
        <div class="dialog-header">
          <h3>安装技能</h3>
          <button class="close-btn" @click="showInstallDialog = false">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <div class="dialog-content">
          <div class="form-group">
            <label>安装方式</label>
            <div class="radio-group">
              <label class="radio-label">
                <input type="radio" value="git" v-model="installType" />
                <span>GitHub URL</span>
              </label>
              <label class="radio-label">
                <input type="radio" value="command" v-model="installType" />
                <span>命令安装</span>
              </label>
            </div>
          </div>
          <div class="form-group">
            <label>{{ installType === 'git' ? 'GitHub 仓库地址' : '安装命令' }}</label>
            <input
              v-model="installInput"
              :placeholder="installType === 'git' ? '输入 GitHub URL 或 用户名/仓库名' : '输入安装命令'"
              @keyup.enter="confirmInstall"
            />
          </div>
          <div v-if="installError" class="install-error">{{ installError }}</div>
        </div>
        <div class="dialog-actions">
          <button class="cancel-btn" @click="showInstallDialog = false">取消</button>
          <button class="save-btn" @click="confirmInstall" :disabled="!installInput.trim() || installLoading">
            {{ installLoading ? '安装中...' : '安装' }}
          </button>
        </div>
      </div>
    </div>

    <div class="dialog-overlay" v-if="showDeleteDialog" @click.self="showDeleteDialog = false">
      <div class="dialog">
        <div class="dialog-header">
          <h3>确认卸载</h3>
          <button class="close-btn" @click="showDeleteDialog = false">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <div class="dialog-content">
          <p class="confirm-text">确定要卸载技能「{{ deleteTarget }}」吗？此操作不可撤销。</p>
        </div>
        <div class="dialog-actions">
          <button class="cancel-btn" @click="showDeleteDialog = false">取消</button>
          <button class="save-btn danger" @click="confirmDelete">确认卸载</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { skillManager } from '../agent/skills/skillManager'
import type { SkillIndexEntry } from '../agent/skills/types'

defineEmits<{ back: [] }>()

const skills = ref<SkillIndexEntry[]>([])
const loading = ref(false)
const showInstallDialog = ref(false)
const showDeleteDialog = ref(false)
const deleteTarget = ref<string>('')
const installType = ref<'git' | 'command'>('git')
const installInput = ref('')
const installLoading = ref(false)
const installError = ref('')

async function loadSkills() {
  loading.value = true
  try {
    await skillManager.loadAllSkills()
    skills.value = skillManager.getAllIndexEntries()
  } catch (e) {
    console.warn('Failed to load skills:', e)
  } finally {
    loading.value = false
  }
}

async function toggleSkill(name: string, enabled: boolean) {
  if (enabled) {
    await skillManager.enableSkill(name)
  } else {
    await skillManager.disableSkill(name)
  }
  skills.value = skillManager.getAllIndexEntries()
}

async function toggleReadonly(name: string, readonly: boolean) {
  await skillManager.setReadonly(name, readonly)
  skills.value = skillManager.getAllIndexEntries()
}

async function refreshSkills() {
  await loadSkills()
}

async function openDir() {
  await skillManager.openSkillsDir()
}

async function confirmInstall() {
  if (!installInput.value.trim()) return
  installLoading.value = true
  installError.value = ''
  try {
    let result
    if (installType.value === 'git') {
      let url = installInput.value.trim()
      if (!url.startsWith('http') && !url.startsWith('git@')) {
        url = `https://github.com/${url}`
      }
      result = await skillManager.installFromGit(url)
    } else {
      result = await skillManager.installByCommand(installInput.value.trim())
    }
    if (result.success) {
      showInstallDialog.value = false
      installInput.value = ''
      await loadSkills()
    } else {
      installError.value = result.message
    }
  } catch (e: any) {
    installError.value = e?.toString() || '安装失败'
  } finally {
    installLoading.value = false
  }
}

function startDelete(name: string) {
  deleteTarget.value = name
  showDeleteDialog.value = true
}

async function confirmDelete() {
  const result = await skillManager.uninstallSkill(deleteTarget.value)
  showDeleteDialog.value = false
  if (result.success) {
    await loadSkills()
  } else {
    console.warn(result.message)
  }
}

function isEligible(entry: SkillIndexEntry): boolean {
  return entry.gatingStatus?.eligible !== false
}

onMounted(() => {
  loadSkills()
})
</script>

<style scoped>
.skills-page {
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

.action-btn.refresh:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.skills-grid {
  flex: 1;
  padding: 16px 24px;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  align-content: start;
}

.skill-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: var(--color-surface-card);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.skill-card:hover {
  border-color: var(--color-border-light);
  background: var(--color-primary-alpha-08);
  box-shadow: 0 2px 8px var(--color-shadow-alpha-06);
}

.skill-card.disabled {
  opacity: 0.55;
}

.skill-card-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.skill-emoji {
  font-size: 22px;
  line-height: 1;
  flex-shrink: 0;
}

.skill-name {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-tag {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  flex-shrink: 0;
}

.source-tag.workspace {
  background: var(--color-accent-info-alpha-10);
  color: var(--color-accent-info);
}

.source-tag.user {
  background: var(--color-primary-alpha-10);
  color: var(--color-primary);
}

.source-tag.builtin {
  background: var(--color-surface-secondary);
  color: var(--color-text-muted);
}

.toggle-switch {
  position: relative;
  width: 40px;
  height: 22px;
  flex-shrink: 0;
  cursor: pointer;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
  position: absolute;
}

.toggle-slider {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-surface-secondary);
  border-radius: 22px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid var(--color-border);
}

.toggle-slider::before {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  top: 2px;
  background: var(--color-text-secondary);
  border-radius: 50%;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.toggle-switch input:checked + .toggle-slider {
  background: var(--color-primary-gradient);
  border-color: var(--color-primary);
}

.toggle-switch input:checked + .toggle-slider::before {
  transform: translateX(18px);
  background: var(--color-text-inverse);
}

.toggle-switch input:disabled + .toggle-slider {
  opacity: 0.4;
  cursor: not-allowed;
}

.skill-description {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-secondary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.5;
}

.skill-deps {
  margin-top: 2px;
}

.dep-items {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.dep-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 6px;
}

.dep-item.ok {
  color: #22c55e;
  background: rgba(34, 197, 94, 0.1);
}

.dep-item.missing {
  color: var(--color-accent-error);
  background: var(--color-danger-bg);
}

.dep-item.os-warn {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.1);
}

.dep-icon {
  font-weight: 700;
}

.skill-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 2px;
}

.readonly-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.readonly-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-muted);
  user-select: none;
}

.mini-toggle {
  position: relative;
  width: 32px;
  height: 18px;
  flex-shrink: 0;
  cursor: pointer;
}

.mini-toggle input {
  opacity: 0;
  width: 0;
  height: 0;
  position: absolute;
}

.mini-toggle-slider {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-surface-secondary);
  border-radius: 18px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid var(--color-border);
}

.mini-toggle-slider::before {
  content: '';
  position: absolute;
  width: 12px;
  height: 12px;
  left: 2px;
  top: 2px;
  background: var(--color-text-secondary);
  border-radius: 50%;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.mini-toggle input:checked + .mini-toggle-slider {
  background: var(--color-primary-gradient);
  border-color: var(--color-primary);
}

.mini-toggle input:checked + .mini-toggle-slider::before {
  transform: translateX(14px);
  background: var(--color-text-inverse);
}

.delete-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border: none;
  background: var(--color-danger-bg);
  color: var(--color-text-secondary);
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.delete-btn:hover {
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

.empty-icon svg {
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { transform: scale(1); opacity: 0.6; }
  50% { transform: scale(1.05); opacity: 1; }
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

.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-shadow-alpha-40);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.dialog {
  width: 100%;
  max-width: 480px;
  max-height: 80vh;
  background: var(--color-surface-card);
  border-radius: 20px;
  box-shadow: 0 20px 60px var(--color-shadow-alpha-20);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  animation: slideUp 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 28px;
  border-bottom: 1px solid var(--color-border);
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.close-btn {
  width: 36px;
  height: 36px;
  border: none;
  background: var(--color-surface-secondary);
  cursor: pointer;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all 0.25s;
}

.close-btn:hover {
  background: var(--color-surface-tertiary);
  color: var(--color-text-primary);
}

.dialog-content {
  padding: 28px;
  overflow-y: auto;
  flex: 1;
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

.form-group input {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-surface);
  transition: all 0.25s;
  font-family: inherit;
}

.form-group input:focus {
  outline: none;
  border-color: var(--color-primary);
  background: var(--color-surface-card);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.form-group input::placeholder {
  color: var(--color-text-muted);
}

.radio-group {
  display: flex;
  gap: 16px;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 14px;
  color: var(--color-text-primary);
}

.radio-label input[type="radio"] {
  width: 16px;
  height: 16px;
  accent-color: var(--color-primary);
}

.install-error {
  padding: 10px 14px;
  background: var(--color-danger-bg);
  color: var(--color-accent-error);
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  margin-top: 12px;
}

.confirm-text {
  font-size: 14px;
  color: var(--color-text-primary);
  line-height: 1.6;
  margin: 0;
}

.dialog-actions {
  display: flex;
  gap: 12px;
  padding: 20px 28px;
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
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
  background: var(--color-accent-error);
  box-shadow: 0 4px 16px rgba(239, 68, 68, 0.25);
}

.save-btn.danger:hover:not(:disabled) {
  box-shadow: 0 6px 20px rgba(239, 68, 68, 0.35);
}
</style>
