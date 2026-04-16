<template>
  <div class="touchbridge-page" :class="{ 'drag-active': isDragOver }">
    <div class="drag-overlay" v-if="isDragOver">
      <div class="drag-overlay-content">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="7 10 12 15 17 10" />
          <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
        <p>拖放 .fap 文件到此处安装</p>
      </div>
    </div>

    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 12H5M12 19l-7-7 7-7" />
          </svg>
        </button>
        <h2>我的应用</h2>
      </div>
      <div class="header-actions">
        <button class="action-btn refresh" @click="loadPackages" :disabled="loading">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M1 4v6h6M23 20v-6h-6" />
            <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15" />
          </svg>
          <span>刷新</span>
        </button>
        <button class="action-btn add" @click="pickAndInstall">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19" />
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          <span>安装应用</span>
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading-state">
      <div class="loading-spinner"></div>
      <p>加载中...</p>
    </div>

    <div v-else-if="error" class="error-state">
      <p class="error-text">{{ error }}</p>
      <button class="action-btn" @click="loadPackages">重试</button>
    </div>

    <div class="packages-list" v-else-if="packages.length > 0">
      <div
        v-for="pkg in packages"
        :key="pkg.package"
        class="package-card"
      >
        <div class="package-card-header" @click="openDetail(pkg)">
          <div class="package-info">
            <div class="package-title-row">
              <span class="package-name">{{ pkg.name }}</span>
              <span class="package-version">v{{ pkg.version }}</span>
              <span v-if="pkg.mode === 'sdk'" class="mode-tag sdk">SDK</span>
              <span v-if="pkg.signature" class="sign-tag signed">已签名</span>
              <span v-else class="sign-tag unsigned">未签名</span>
            </div>
            <div class="package-id">{{ pkg.package }}</div>
            <div v-if="pkg.description" class="package-desc">{{ pkg.description }}</div>
          </div>
          <div class="package-actions">
            <button class="delete-btn" @click.stop="startUninstall(pkg)">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6" />
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
              </svg>
              <span>卸载</span>
            </button>
            <button class="detail-btn" @click.stop="openDetail(pkg)">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10" />
                <line x1="12" y1="16" x2="12" y2="12" />
                <line x1="12" y1="8" x2="12.01" y2="8" />
              </svg>
              <span>详情</span>
            </button>
          </div>
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
      <p>还没有安装应用</p>
      <span>点击"安装应用"按钮选择 .fap 文件，或直接拖放 .fap 文件到此处进行安装</span>
    </div>

    <div class="dialog-overlay" v-if="showUninstallDialog" @click.self="showUninstallDialog = false">
      <div class="dialog">
        <div class="dialog-header">
          <h3>确认卸载</h3>
          <button class="close-btn" @click="showUninstallDialog = false">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <div class="dialog-content">
          <p class="confirm-text">确定要卸载应用「{{ uninstallTarget?.name }}」({{ uninstallTarget?.package }}) 吗？此操作不可撤销。</p>
        </div>
        <div class="dialog-actions">
          <button class="cancel-btn" @click="showUninstallDialog = false">取消</button>
          <button class="save-btn danger" @click="confirmUninstall" :disabled="uninstallLoading">
            {{ uninstallLoading ? '卸载中...' : '确认卸载' }}
          </button>
        </div>
      </div>
    </div>

    <div class="dialog-overlay" v-if="installDialog.show" @click.self="installDialog.status !== 'installing' && closeInstallDialog()">
      <div class="dialog install-dialog">
        <div class="dialog-header">
          <h3>安装应用</h3>
          <button v-if="installDialog.status !== 'installing'" class="close-btn" @click="closeInstallDialog">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <div class="dialog-content">
          <div class="install-file-name">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
            </svg>
            <span>{{ installDialog.fileName }}</span>
          </div>
          <div class="install-status" v-if="installDialog.status === 'installing'">
            <div class="loading-spinner small"></div>
            <span class="status-text">正在安装...</span>
          </div>
          <div class="install-status success" v-else-if="installDialog.status === 'success'">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
              <polyline points="22 4 12 14.01 9 11.01" />
            </svg>
            <span class="status-text">安装成功</span>
          </div>
          <div class="install-status error" v-else-if="installDialog.status === 'error'">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" />
              <line x1="9" y1="9" x2="15" y2="15" />
            </svg>
            <span class="status-text">安装失败</span>
          </div>
          <p v-if="installDialog.status === 'error'" class="install-error-msg">{{ installDialog.message }}</p>
        </div>
        <div class="dialog-actions" v-if="installDialog.status !== 'installing'">
          <button class="save-btn" @click="closeInstallDialog">关闭</button>
        </div>
      </div>
    </div>

    <div class="dialog-overlay" v-if="showDetailDialog" @click.self="showDetailDialog = false">
      <div class="dialog detail-dialog">
        <div class="dialog-header">
          <h3>{{ detailTarget?.name }}</h3>
          <button class="close-btn" @click="showDetailDialog = false">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <div class="dialog-scroll-content" v-if="detailTarget">
          <div class="detail-summary">
            <div class="detail-summary-row">
              <span class="package-version">v{{ detailTarget.version }}</span>
              <span v-if="detailTarget.mode === 'sdk'" class="mode-tag sdk">SDK</span>
              <span v-if="detailTarget.signature" class="sign-tag signed">已签名</span>
              <span v-else class="sign-tag unsigned">未签名</span>
            </div>
            <div class="package-id">{{ detailTarget.package }}</div>
            <div v-if="detailTarget.description" class="package-desc">{{ detailTarget.description }}</div>
          </div>

          <div class="detail-section">
            <h4>能力域</h4>
            <div v-if="detailTarget.capabilities && Object.keys(detailTarget.capabilities).length > 0" class="capability-list">
              <div v-for="(actions, domain) in detailTarget.capabilities" :key="domain" class="capability-domain">
                <div class="domain-name">{{ domain }}</div>
                <div class="action-list">
                  <div v-for="action in actions" :key="action['名称']" class="action-item">
                    <span v-if="action['名称'] !== domain" class="action-name">{{ action['名称'] }}</span>
                    <div v-if="action['参数'] && Object.keys(action['参数']).length > 0" class="action-params">
                      <span v-for="(paramDef, paramKey) in action['参数']" :key="paramKey" class="param-tag">
                        {{ paramKey }}
                        <span class="param-type">{{ paramDef['类型'] }}</span>
                        <span v-if="paramDef['必填']" class="param-required">*</span>
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <div v-else class="no-capabilities">无能力声明</div>
          </div>

          <div class="detail-section" v-if="detailTarget.permissions && detailTarget.permissions.length > 0">
            <h4>权限</h4>
            <div class="permission-tags">
              <span v-for="perm in detailTarget.permissions" :key="perm" class="perm-tag">{{ perm }}</span>
            </div>
          </div>

          <div class="detail-section">
            <h4>信息</h4>
            <div class="info-grid">
              <div class="info-item">
                <span class="info-label">运行模式</span>
                <span class="info-value">{{ detailTarget.mode === 'sdk' ? 'SDK 模式 (fa-bridge-sdk)' : 'Manifest Mapping' }}</span>
              </div>
              <div class="info-item">
                <span class="info-label">生命周期</span>
                <span class="info-value">{{ lifecycleLabel(detailTarget.lifecycle) }}</span>
              </div>
              <div class="info-item">
                <span class="info-label">平台</span>
                <span class="info-value">{{ (detailTarget.platforms || []).join(', ') || '未指定' }}</span>
              </div>
            </div>
          </div>
        </div>
        <div class="dialog-actions">
          <button class="save-btn" @click="showDetailDialog = false">关闭</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invalidateFapCache } from '../agent/systemPromptAssembler'

defineEmits<{ back: [] }>()

interface FapPackage {
  package: string
  name: string
  version: string
  description?: string
  mode: string
  lifecycle?: string
  platforms?: string[]
  capabilities?: Record<string, Array<{ '名称': string; '参数'?: Record<string, { '类型': string; '必填'?: boolean; '描述'?: string; '默认'?: any }>; 'invoke'?: any }>>
  permissions?: string[]
  signature?: any
}

const packages = ref<FapPackage[]>([])
const loading = ref(false)
const error = ref('')
const showDetailDialog = ref(false)
const detailTarget = ref<FapPackage | null>(null)
const showUninstallDialog = ref(false)
const uninstallTarget = ref<FapPackage | null>(null)
const uninstallLoading = ref(false)
const isDragOver = ref(false)

const installDialog = ref<{
  show: boolean
  status: 'installing' | 'success' | 'error'
  message: string
  fileName: string
}>({ show: false, status: 'installing', message: '', fileName: '' })

const unlisteners: UnlistenFn[] = []

function lifecycleLabel(lifecycle?: string): string {
  switch (lifecycle) {
    case 'oneshot': return '单次执行 (Oneshot)'
    case 'persistent': return '持久连接 (Persistent)'
    case 'both': return '两种模式 (Both)'
    default: return '未指定'
  }
}

function openDetail(pkg: FapPackage) {
  detailTarget.value = pkg
  showDetailDialog.value = true
}

function extractFileName(path: string): string {
  const parts = path.replace(/\\/g, '/').split('/')
  return parts[parts.length - 1] || path
}

async function loadPackages() {
  loading.value = true
  error.value = ''
  try {
    const result = await invoke('fap_list') as any
    if (result && result.packages) {
      packages.value = result.packages
    } else if (Array.isArray(result)) {
      packages.value = result
    } else if (result && typeof result === 'object') {
      const pkgs: FapPackage[] = []
      const data = result
      if (data['能力池'] && Array.isArray(data['能力池'])) {
        for (const entry of data['能力池']) {
          pkgs.push({
            package: entry['包名'] || entry['package'] || '',
            name: entry['名称'] || entry['name'] || '',
            version: entry['版本'] || entry['version'] || '0.0.0',
            mode: entry['模式'] || entry['mode'] || 'manifest',
            capabilities: entry['能力域'] ? Object.fromEntries(
              entry['能力域'].map((d: any) => [d['名称'], d['动作'] || []])
            ) : {},
          })
        }
      }
      packages.value = pkgs
    }
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function installFromPath(fapPath: string) {
  installDialog.value = {
    show: true,
    status: 'installing',
    message: '',
    fileName: extractFileName(fapPath)
  }
  try {
    await invoke('fap_install', { fapPath })
    invalidateFapCache()
    installDialog.value.status = 'success'
    await loadPackages()
  } catch (e) {
    installDialog.value.status = 'error'
    installDialog.value.message = String(e)
  }
}

function closeInstallDialog() {
  installDialog.value.show = false
}

async function pickAndInstall() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'FAP 包', extensions: ['fap'] }]
  })
  if (!selected || typeof selected !== 'string') return
  await installFromPath(selected)
}

function startUninstall(pkg: FapPackage) {
  uninstallTarget.value = pkg
  showUninstallDialog.value = true
}

async function confirmUninstall() {
  if (!uninstallTarget.value) return
  uninstallLoading.value = true
  try {
    await invoke('fap_uninstall', { packageName: uninstallTarget.value.package })
    invalidateFapCache()
    showUninstallDialog.value = false
    uninstallTarget.value = null
    await loadPackages()
  } catch (e) {
    error.value = String(e)
    showUninstallDialog.value = false
  } finally {
    uninstallLoading.value = false
  }
}

onMounted(async () => {
  loadPackages()

  const unlistenDrop = await listen<string[]>('tauri://file-drop', (event) => {
    isDragOver.value = false
    const fapFiles = event.payload.filter(f => f.toLowerCase().endsWith('.fap'))
    if (fapFiles.length > 0) {
      installFromPath(fapFiles[0])
    }
  })
  const unlistenHover = await listen<string[]>('tauri://file-drop-hover', () => {
    isDragOver.value = true
  })
  const unlistenCancel = await listen('tauri://file-drop-cancelled', () => {
    isDragOver.value = false
  })

  unlisteners.push(unlistenDrop, unlistenHover, unlistenCancel)
})

onUnmounted(() => {
  for (const fn of unlisteners) {
    fn()
  }
})
</script>

<style scoped>
.touchbridge-page {
  width: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-surface-secondary) 50%, var(--color-surface-tertiary) 100%);
  overflow: hidden;
  position: relative;
}

.touchbridge-page.drag-active {
  outline: none;
}

.drag-overlay {
  position: absolute;
  inset: 0;
  z-index: 500;
  background: var(--color-primary-alpha-10);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.drag-overlay-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  color: var(--color-primary);
  padding: 40px 60px;
  border: 2px dashed var(--color-primary);
  border-radius: 20px;
  background: var(--color-primary-alpha-08);
}

.drag-overlay-content p {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
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

.loading-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  gap: 16px;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.loading-spinner.small {
  width: 20px;
  height: 20px;
  border-width: 2px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 40px;
}

.error-text {
  color: var(--color-accent-error);
  font-size: 14px;
  font-weight: 600;
  text-align: center;
}

.packages-list {
  flex: 1;
  padding: 16px 24px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.package-card {
  background: var(--color-surface-card);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.package-card:hover {
  border-color: var(--color-border-light);
  box-shadow: 0 2px 8px var(--color-shadow-alpha-06);
}

.package-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  cursor: pointer;
  transition: background 0.2s;
}

.package-card-header:hover {
  background: var(--color-primary-alpha-08);
}

.package-info {
  flex: 1;
  min-width: 0;
}

.package-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.package-name {
  font-size: 15px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.package-version {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  background: var(--color-surface-secondary);
  padding: 2px 8px;
  border-radius: 6px;
}

.mode-tag {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 6px;
}

.mode-tag.manifest {
  background: var(--color-accent-info-alpha-10);
  color: var(--color-accent-info);
}

.mode-tag.sdk {
  background: var(--color-primary-alpha-10);
  color: var(--color-primary);
}

.sign-tag {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 6px;
}

.sign-tag.signed {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
}

.sign-tag.unsigned {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.package-id {
  font-size: 12px;
  color: var(--color-text-muted);
  margin-top: 4px;
  font-family: monospace;
}

.package-desc {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 4px;
}

.package-actions {
  display: flex;
  align-items: center;
  gap: 12px;
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

.detail-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border: none;
  background: var(--color-surface-secondary);
  color: var(--color-text-secondary);
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.detail-btn:hover {
  background: var(--color-primary-alpha-10);
  color: var(--color-primary);
}

.package-detail {
  padding: 0 20px 20px;
  border-top: 1px solid var(--color-border);
}

.detail-section {
  margin-top: 16px;
}

.detail-section h4 {
  margin: 0 0 10px;
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.capability-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.capability-domain {
  background: var(--color-surface-secondary);
  border-radius: 10px;
  padding: 12px 16px;
}

.domain-name {
  font-size: 13px;
  font-weight: 700;
  color: var(--color-primary);
  margin-bottom: 8px;
}

.action-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.action-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.action-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.action-params {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.param-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text-secondary);
  font-weight: 600;
}

.param-type {
  color: var(--color-text-muted);
  font-weight: 400;
}

.param-required {
  color: var(--color-accent-error);
  font-weight: 700;
}

.no-capabilities {
  font-size: 13px;
  color: var(--color-text-muted);
}

.permission-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.perm-tag {
  font-size: 11px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 6px;
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.info-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-item {
  display: flex;
  gap: 12px;
}

.info-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  min-width: 80px;
}

.info-value {
  font-size: 12px;
  color: var(--color-text-secondary);
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
  background: var(--color-surface-card);
  border-radius: 20px;
  box-shadow: 0 20px 60px var(--color-shadow-alpha-20);
  overflow: hidden;
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

.save-btn.danger {
  background: var(--color-accent-error);
  box-shadow: 0 4px 16px rgba(239, 68, 68, 0.25);
}

.save-btn.danger:hover:not(:disabled) {
  box-shadow: 0 6px 20px rgba(239, 68, 68, 0.35);
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.install-dialog .dialog-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.install-file-name {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  background: var(--color-surface-secondary);
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
  word-break: break-all;
}

.install-file-name svg {
  flex-shrink: 0;
  color: var(--color-primary);
}

.install-status {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-radius: 10px;
  background: var(--color-surface-secondary);
}

.install-status.success {
  color: #22c55e;
}

.install-status.error {
  color: var(--color-accent-error);
}

.install-status .status-text {
  font-size: 14px;
  font-weight: 600;
}

.install-error-msg {
  margin: 0;
  font-size: 13px;
  color: var(--color-accent-error);
  line-height: 1.5;
  word-break: break-word;
}

.detail-dialog {
  max-width: 560px;
}

.dialog-scroll-content {
  padding: 28px;
  max-height: 60vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.detail-summary {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.detail-summary-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
</style>
