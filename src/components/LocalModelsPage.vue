<template>
  <div class="local-models-page">
    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="19" y1="12" x2="5" y2="12"></line>
            <polyline points="12,19 5,12 12,5"></polyline>
          </svg>
        </button>
        <h2>本地模型</h2>
      </div>
    </div>

    <div class="settings-content">
      <div class="card-section">
        <div class="settings-inner-card">
          <div class="settings-inner-header">
            <h3>环境检测</h3>
          </div>
            <div class="settings-inner-content">
              <div class="env-status-row" v-if="environmentStatus">
                <div class="env-item" :class="{ ok: environmentStatus.python, fail: !environmentStatus.python }">
                  <span class="env-icon">{{ environmentStatus.python ? '✅' : '❌' }}</span>
                  <span class="env-label">Python</span>
                  <span class="env-version" v-if="environmentStatus.python_version">{{ environmentStatus.python_version }}</span>
                </div>
                <div class="env-item" :class="{ ok: environmentStatus.modelscope, fail: !environmentStatus.modelscope }">
                  <span class="env-icon">{{ environmentStatus.modelscope ? '✅' : '❌' }}</span>
                  <span class="env-label">ModelScope</span>
                </div>
                <div class="env-item" :class="{ ok: environmentStatus.ollama, fail: !environmentStatus.ollama }">
                  <span class="env-icon">{{ environmentStatus.ollama ? '✅' : '❌' }}</span>
                  <span class="env-label">Ollama</span>
                  <span class="env-version" v-if="environmentStatus.ollama_version">{{ environmentStatus.ollama_version }}</span>
                  <button
                    v-if="!environmentStatus.ollama && !isInstallingOllama"
                    class="install-btn"
                    @click="installOllama"
                  >
                    下载
                  </button>
                </div>
              </div>
              <div class="env-loading" v-else>
                <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                  <line x1="12" y1="2" x2="12" y2="6"></line>
                  <line x1="12" y1="18" x2="12" y2="22"></line>
                  <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
                  <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                  <line x1="2" y1="12" x2="6" y2="12"></line>
                  <line x1="18" y1="12" x2="22" y2="12"></line>
                  <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
                  <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
                </svg>
                <span>正在检测环境...</span>
              </div>
              <div class="env-hint" v-if="environmentStatus && (!environmentStatus.python || !environmentStatus.modelscope || !environmentStatus.ollama) && !isInstallingOllama">
                <button class="hint-toggle" @click="showInstallGuide = !showInstallGuide">
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" :style="{ transform: showInstallGuide ? 'rotate(90deg)' : '' }">
                    <polyline points="9,18 15,12 9,6"></polyline>
                  </svg>
                  <span>安装指引</span>
                </button>
                <div class="install-guide" v-if="showInstallGuide">
                  <div class="guide-item" v-if="!environmentStatus.modelscope">
                    <span class="guide-label">ModelScope:</span>
                    <code class="guide-code">pip install modelscope</code>
                  </div>
                  <div class="guide-item" v-if="!environmentStatus.ollama">
                    <span class="guide-label">Ollama:</span>
                    <span>点击上方的"下载"按钮，或前往</span>
                    <code class="guide-code">ollama.com</code>
                    <span>下载安装</span>
                  </div>
                </div>
              </div>
              <div class="ollama-install-progress" v-if="isInstallingOllama">
                <div class="install-progress-header">
                  <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                    <line x1="12" y1="2" x2="12" y2="6"></line>
                    <line x1="12" y1="18" x2="12" y2="22"></line>
                    <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
                    <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                    <line x1="2" y1="12" x2="6" y2="12"></line>
                    <line x1="18" y1="12" x2="22" y2="12"></line>
                    <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
                    <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
                  </svg>
                  <span>正在下载 Ollama 安装包...</span>
                </div>
                <div class="install-progress-log" v-if="ollamaInstallProgress">
                  <span class="progress-message">{{ ollamaInstallProgress.message }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="card-section">
          <div class="settings-inner-card">
            <div class="settings-inner-header">
              <h3>安装模型</h3>
            </div>
            <div class="settings-inner-content">
              <div class="install-row">
                <input
                  class="search-input"
                  v-model="modelIdInput"
                  placeholder="输入模型 ID，例如 Qwen/Qwen3.5-0.8B"
                  @keyup.enter="handleInstallModel"
                />
                <button class="search-btn" @click="handleInstallModel" :disabled="isDownloading || !isValidModelId">
                  <svg v-if="!isDownloading" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                    <polyline points="7,10 12,15 17,10"></polyline>
                    <line x1="12" y1="15" x2="12" y2="3"></line>
                  </svg>
                  <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                    <line x1="12" y1="2" x2="12" y2="6"></line>
                    <line x1="12" y1="18" x2="12" y2="22"></line>
                    <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
                    <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                    <line x1="2" y1="12" x2="6" y2="12"></line>
                    <line x1="18" y1="12" x2="22" y2="12"></line>
                    <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
                    <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
                  </svg>
                  <span>{{ isDownloading ? '下载中...' : '下载' }}</span>
                </button>
              </div>

              <div class="download-progress-area" v-if="isDownloading || downloadProgress">
                <div class="download-info">
                  <span class="download-file" :class="{ 'download-error': downloadProgress?.status === 'error' }">
                    {{ downloadProgress?.status === 'error' ? '下载失败' : (downloadProgress?.status === 'cancelled' ? '已取消' : (downloadProgress?.current_file || '准备下载...')) }}
                  </span>
                  <span v-if="downloadProgress?.status !== 'error' && downloadProgress?.status !== 'cancelled'" class="download-percent">{{ downloadProgress?.progress_percent || 0 }}%</span>
                </div>
                <div class="progress-bar" v-if="downloadProgress?.status !== 'error' && downloadProgress?.status !== 'cancelled'">
                  <div class="progress-fill" :style="{ width: (downloadProgress?.progress_percent || 0) + '%' }"></div>
                </div>
                <div class="download-log" v-if="downloadProgress?.message && downloadProgress.message !== downloadProgress.current_file">
                  <span>{{ downloadProgress.message }}</span>
                </div>
                <div class="download-actions-row">
                  <button v-if="isDownloading" class="cancel-download-btn" @click="handleCancelDownload">取消</button>
                  <button v-if="!isDownloading && downloadProgress" class="cancel-download-btn" @click="downloadProgress = null">关闭</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="card-section">
          <div class="settings-inner-card">
            <div class="settings-inner-header">
              <h3>本地模型</h3>
            </div>
            <div class="settings-inner-content">
              <div class="deploy-error-banner" v-if="deployError">
                <span class="deploy-error-text">{{ deployError }}</span>
                <button class="deploy-error-close" @click="deployError = null">关闭</button>
              </div>
              <div class="models-list" v-if="models.length > 0">
                <div v-for="model in models" :key="model.id" class="model-card">
                  <div class="model-left">
                    <div class="model-icon">{{ getTypeIcon(model.modelType) }}</div>
                  </div>
                  <div class="model-center">
                    <div class="model-display-name">{{ model.displayName }}</div>
                    <div class="model-id-text">{{ model.modelId }}</div>
                    <div class="model-meta-row">
                      <span class="type-badge" :style="{ background: getTypeColor(model.modelType) }">
                        {{ getTypeLabel(model.modelType) }}
                      </span>
                      <span class="model-size" v-if="model.sizeBytes">{{ formatSize(model.sizeBytes) }}</span>
                    </div>
                    <div class="model-path" v-if="model.localPath">{{ model.localPath }}</div>
                  </div>
                  <div class="model-right">
                    <div class="model-status" :class="model.status">
                      <span class="status-dot"></span>
                      <span class="status-text">{{ getStatusText(model.status) }}</span>
                    </div>
                    <div class="model-actions">
                      <button
                        v-if="model.status !== 'running'"
                        class="model-action-btn deploy"
                        @click="handleDeploy(model)"
                        :disabled="model.status === 'downloading' || model.status === 'stopping'"
                      >
                        部署
                      </button>
                      <button
                        v-if="model.status === 'running'"
                        class="model-action-btn stop"
                        @click="handleStop(model.id)"
                      >
                        停止
                      </button>
                      <button class="model-action-btn config" @click="openDeployConfig(model)">
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                          <circle cx="12" cy="12" r="3"></circle>
                          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
                        </svg>
                      </button>
                      <button
                        v-if="model.status === 'running'"
                        class="model-action-btn add-api"
                        @click="handleAddAsProvider(model)"
                      >
                        添加为API
                      </button>
                      <button class="model-action-btn delete" @click="openDeleteConfirm(model)">
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                          <polyline points="3,6 5,6 21,6"></polyline>
                          <path d="M19,6v14a2,2,0,0,1-2,2H7a2,2,0,0,1-2-2V6m3,0V4a2,2,0,0,1,2-2h4a2,2,0,0,1,2,2v2"></path>
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              <div class="empty-state" v-else>
                <div class="empty-icon">
                  <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
                    <rect x="2" y="2" width="20" height="20" rx="4" ry="4"></rect>
                    <path d="M7 8h10"></path>
                    <path d="M7 12h7"></path>
                    <path d="M7 16h4"></path>
                  </svg>
                </div>
                <p>还没有本地模型</p>
                <span>在上方输入模型 ID 下载安装</span>
              </div>
            </div>
          </div>
        </div>
      </div>

    <BaseDialog v-model="showDeployConfig" title="部署配置">
      <div class="form-group">
        <label>端口</label>
        <input type="text" value="11434" disabled />
        <span class="form-hint">Ollama 使用固定端口 11434</span>
      </div>
      <div class="form-group">
        <label>上下文长度</label>
        <input type="number" v-model.number="deployConfigForm.ctxSize" placeholder="2048" min="256" />
      </div>
      <div class="form-group">
        <label>线程数</label>
        <input type="number" v-model.number="deployConfigForm.threads" placeholder="4" min="1" />
      </div>
      <div class="form-group">
        <label>GPU 层数</label>
        <input type="number" v-model.number="deployConfigForm.gpuLayers" placeholder="0" min="0" />
        <span class="form-hint">设置为 0 时仅使用 CPU</span>
      </div>
      <template #actions>
        <button class="cancel-btn" @click="showDeployConfig = false">取消</button>
        <button class="save-btn" @click="handleConfirmDeploy">确认部署</button>
      </template>
    </BaseDialog>

    <BaseDialog v-model="showDeleteConfirm" title="确认删除">
      <div class="delete-warning">
        <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="#E74C3C" stroke-width="1.5">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
          <line x1="12" y1="9" x2="12" y2="13"></line>
          <line x1="12" y1="17" x2="12.01" y2="17"></line>
        </svg>
        <p>确定要删除模型「<strong>{{ deletingModel?.displayName }}</strong>」吗？</p>
        <span>此操作不可恢复。</span>
      </div>
      <template #actions>
        <button class="cancel-btn" @click="showDeleteConfirm = false">取消</button>
        <button class="danger-btn" @click="handleConfirmDelete">确认删除</button>
      </template>
    </BaseDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import BaseDialog from './BaseDialog.vue'
import { useModelManager } from '../composables/useModelManager'
import type { LocalModel } from '../stores/localModels'
import { getDefaultDeployConfig, updateLocalModel } from '../stores/localModels'

defineEmits<{
  back: []
}>()

const {
  environmentStatus,
  isDownloading,
  downloadProgress,
  models,
  isInstallingOllama,
  ollamaInstallProgress,
  deployError,
  downloadModel,
  cancelDownload,
  deployModel,
  stopModel,
  deleteModel,
  addAsProvider,
  installOllama,
} = useModelManager()

const modelIdInput = ref('')
const showInstallGuide = ref(false)

const isValidModelId = computed(() => {
  const k = modelIdInput.value.trim()
  return k.includes('/') && k.split('/').length === 2 && k.split('/').every(p => p.length > 0)
})

const showDeployConfig = ref(false)
const showDeleteConfirm = ref(false)
const editingModel = ref<LocalModel | null>(null)
const deletingModel = ref<LocalModel | null>(null)

const deployConfigForm = reactive({
  ctxSize: 2048,
  threads: 4,
  gpuLayers: 0
})

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const k = 1024
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  const value = bytes / Math.pow(k, i)
  return value.toFixed(i === 0 ? 0 : 1) + ' ' + units[i]
}

function getTypeLabel(type: string): string {
  const map: Record<string, string> = {
    llm: '大语言模型',
    embedding: '嵌入模型',
    tts: '语音合成',
    other: '其他'
  }
  return map[type] || '其他'
}

function getTypeColor(type: string): string {
  const map: Record<string, string> = {
    llm: 'linear-gradient(135deg, #E67E22 0%, #F39C12 100%)',
    embedding: 'linear-gradient(135deg, #3498DB 0%, #5DADE2 100%)',
    tts: 'linear-gradient(135deg, #9B59B6 0%, #BB8FCE 100%)',
    other: 'linear-gradient(135deg, #95A5A6 0%, #BDC3C7 100%)'
  }
  return map[type] || map.other
}

function getTypeIcon(type: string): string {
  const map: Record<string, string> = {
    llm: '🤖',
    embedding: '📎',
    tts: '🔊',
    other: '📦'
  }
  return map[type] || '📦'
}

function getStatusText(status: string): string {
  const map: Record<string, string> = {
    running: '运行中',
    ready: '已停止',
    downloading: '下载中',
    error: '错误',
    stopping: '停止中'
  }
  return map[status] || status
}

async function handleInstallModel() {
  const id = modelIdInput.value.trim()
  if (!id || !isValidModelId.value) return
  const displayName = id.split('/').pop() || id
  downloadModel(id, displayName)
}

function handleCancelDownload() {
  if (downloadProgress.value) {
    cancelDownload(downloadProgress.value.model_id)
  }
}

function openDeployConfig(model: LocalModel) {
  editingModel.value = model
  const config = model.deployConfig || getDefaultDeployConfig()
  deployConfigForm.ctxSize = config.ctxSize
  deployConfigForm.threads = config.threads
  deployConfigForm.gpuLayers = config.gpuLayers
  showDeployConfig.value = true
}

function handleConfirmDeploy() {
  if (!editingModel.value) return
  updateLocalModel(editingModel.value.id, {
    deployConfig: {
      port: 11434,
      ctxSize: deployConfigForm.ctxSize,
      threads: deployConfigForm.threads,
      gpuLayers: deployConfigForm.gpuLayers
    }
  })
  const model = { ...editingModel.value, deployConfig: { port: 11434, ctxSize: deployConfigForm.ctxSize, threads: deployConfigForm.threads, gpuLayers: deployConfigForm.gpuLayers } }
  showDeployConfig.value = false
  deployModel(model)
}

function handleDeploy(model: LocalModel) {
  openDeployConfig(model)
}

function handleStop(modelId: string) {
  stopModel(modelId)
}

function openDeleteConfirm(model: LocalModel) {
  deletingModel.value = model
  showDeleteConfirm.value = true
}

function handleConfirmDelete() {
  if (!deletingModel.value) return
  deleteModel(deletingModel.value.id)
  showDeleteConfirm.value = false
  deletingModel.value = null
}

function handleAddAsProvider(model: LocalModel) {
  addAsProvider(model)
}
</script>

<style scoped>
.local-models-page {
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
  flex-shrink: 0;
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

.settings-content {
  flex: 1;
  padding: 20px 28px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-section {
  width: 100%;
}

.settings-inner-card {
  border: 1px solid var(--color-border);
  border-radius: 14px;
  background: var(--color-surface);
  overflow: hidden;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.settings-inner-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface-card);
}

.settings-inner-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.settings-inner-content {
  padding: 20px;
}

.env-status-row {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.env-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-radius: 10px;
  background: var(--color-surface-card);
  border: 1px solid var(--color-border);
  flex: 1;
  min-width: 140px;
}

.env-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.env-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.env-version {
  font-size: 11px;
  color: var(--color-text-muted);
  margin-left: auto;
}

.env-item.ok {
  border-color: var(--color-accent-success);
  background: var(--color-accent-success-alpha-10);
}

.env-item.fail {
  border-color: var(--color-accent-error);
  background: var(--color-danger-bg);
}

.env-loading {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 0;
  color: var(--color-text-muted);
  font-size: 13px;
}

.env-hint {
  margin-top: 12px;
}

.hint-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 0;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-primary);
  transition: all 0.2s;
}

.hint-toggle:hover {
  color: var(--color-primary-hover);
}

.hint-toggle svg {
  transition: transform 0.2s;
}

.install-guide {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px 16px;
  background: var(--color-surface-card);
  border-radius: 10px;
  border: 1px solid var(--color-border);
}

.guide-item {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: var(--color-text-primary);
  flex-wrap: wrap;
}

.guide-label {
  font-weight: 600;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.guide-code {
  padding: 4px 10px;
  background: var(--color-surface-secondary);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 12px;
  color: var(--color-primary);
}

.install-btn {
  margin-left: 8px;
  padding: 2px 10px;
  border: none;
  border-radius: 10px;
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.install-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px var(--color-shadow-primary-strong);
}

.ollama-install-progress {
  margin-top: 12px;
  padding: 12px;
  background: var(--color-surface-secondary);
  border-radius: 8px;
  border: 1px solid var(--color-border);
}

.install-progress-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.install-progress-log {
  margin-top: 8px;
  padding: 8px;
  background: var(--color-surface);
  border-radius: 6px;
  font-size: 11px;
  color: var(--color-text-secondary);
  font-family: 'Consolas', 'Monaco', monospace;
  word-break: break-all;
  max-height: 80px;
  overflow-y: auto;
}

.install-row {
  display: flex;
  gap: 10px;
  margin-bottom: 16px;
}

.search-input {
  flex: 1;
  padding: 12px 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-surface-card);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.search-input::placeholder {
  color: var(--color-text-muted);
}

.search-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 12px 20px;
  border: none;
  background: var(--color-primary-gradient);
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-inverse);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 8px var(--color-shadow-primary-hover);
  white-space: nowrap;
}

.search-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 16px var(--color-shadow-primary-strong);
}

.search-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.download-progress-area {
  padding: 16px;
  background: var(--color-surface-card);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  margin-bottom: 16px;
}

.download-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.download-file {
  font-size: 12px;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 80%;
}

.download-percent {
  font-size: 13px;
  font-weight: 700;
  color: var(--color-primary);
  flex-shrink: 0;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: var(--color-border);
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 10px;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary-gradient);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.cancel-download-btn {
  padding: 6px 14px;
  border: 1px solid var(--color-accent-error);
  background: transparent;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-accent-error);
  cursor: pointer;
  transition: all 0.2s;
}

.cancel-download-btn:hover {
  background: var(--color-accent-error);
  color: var(--color-text-inverse);
}

.download-log {
  margin-bottom: 10px;
  padding: 8px 10px;
  background: var(--color-surface);
  border-radius: 6px;
  font-size: 11px;
  color: var(--color-text-secondary);
  font-family: 'Consolas', 'Monaco', monospace;
  word-break: break-all;
  max-height: 60px;
  overflow-y: auto;
}

.download-log::-webkit-scrollbar {
  width: 3px;
}

.download-log::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 2px;
}

.download-error {
  color: var(--color-accent-error) !important;
  font-weight: 600;
}

.download-actions-row {
  margin-top: 8px;
  display: flex;
  justify-content: flex-end;
}

.models-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.model-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 18px;
  border: 1px solid var(--color-border);
  border-radius: 12px;
  background: var(--color-surface-card);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.model-card:hover {
  border-color: var(--color-border-light);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px var(--color-shadow-alpha-08);
}

.model-left {
  flex-shrink: 0;
}

.model-icon {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  background: linear-gradient(135deg, var(--color-surface-secondary) 0%, var(--color-surface-tertiary) 100%);
  border-radius: 12px;
}

.model-center {
  flex: 1;
  min-width: 0;
}

.model-display-name {
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-id-text {
  font-size: 11px;
  color: var(--color-text-muted);
  margin-bottom: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-meta-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.type-badge {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 6px;
  color: var(--color-text-inverse);
  font-weight: 700;
  letter-spacing: 0.02em;
}

.model-size {
  font-size: 11px;
  color: var(--color-text-muted);
}

.model-path {
  font-size: 11px;
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.deploy-error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: var(--color-accent-error-alpha-10);
  border: 1px solid var(--color-accent-error-alpha-30);
  border-radius: 8px;
  margin-bottom: 12px;
}

.deploy-error-text {
  color: var(--color-accent-error);
  font-size: 13px;
  flex: 1;
  word-break: break-all;
}

.deploy-error-close {
  background: none;
  border: 1px solid var(--color-accent-error-alpha-30);
  color: var(--color-accent-error);
  padding: 2px 10px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  flex-shrink: 0;
  margin-left: 10px;
}

.deploy-error-close:hover {
  background: var(--color-accent-error-alpha-15);
}

.model-right {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 10px;
  flex-shrink: 0;
}

.model-status {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.model-status.ready .status-dot {
  background: var(--color-text-muted);
}

.model-status.running .status-dot {
  background: var(--color-accent-success);
  animation: statusPulse 2s ease-in-out infinite;
}

.model-status.downloading .status-dot {
  background: var(--color-accent-info);
  animation: statusPulse 1.5s ease-in-out infinite;
}

.model-status.error .status-dot {
  background: var(--color-accent-error);
}

.model-status.stopping .status-dot {
  background: var(--color-primary-light);
  animation: statusPulse 1s ease-in-out infinite;
}

@keyframes statusPulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.status-text {
  font-size: 12px;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.model-actions {
  display: flex;
  gap: 6px;
}

.model-action-btn {
  padding: 7px 12px;
  border: 1px solid var(--color-border);
  background: var(--color-surface-card);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 600;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  white-space: nowrap;
}

.model-action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.model-action-btn.deploy {
  border-color: var(--color-accent-success);
  color: var(--color-accent-success);
  background: rgba(39, 174, 96, 0.06);
}

.model-action-btn.deploy:hover:not(:disabled) {
  background: var(--color-accent-success);
  color: var(--color-text-inverse);
}

.model-action-btn.stop {
  border-color: var(--color-accent-error);
  color: var(--color-accent-error);
  background: var(--color-accent-error-alpha-06);
}

.model-action-btn.stop:hover:not(:disabled) {
  background: var(--color-accent-error);
  color: var(--color-text-inverse);
}

.model-action-btn.config {
  background: var(--color-surface-secondary);
  color: var(--color-text-secondary);
}

.model-action-btn.config:hover {
  background: var(--color-primary);
  color: var(--color-text-inverse);
  border-color: var(--color-primary);
}

.model-action-btn.add-api {
  border-color: var(--color-accent-info);
  color: var(--color-accent-info);
  background: rgba(52, 152, 219, 0.06);
}

.model-action-btn.add-api:hover:not(:disabled) {
  background: var(--color-accent-info);
  color: var(--color-text-inverse);
}

.model-action-btn.delete {
  background: var(--color-danger-bg);
  color: var(--color-text-secondary);
}

.model-action-btn.delete:hover {
  border-color: var(--color-accent-error);
  background: var(--color-accent-error);
  color: var(--color-text-inverse);
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.empty-state {
  text-align: center;
  padding: 50px 24px;
  color: var(--color-text-muted);
}

.empty-icon {
  margin-bottom: 16px;
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

.form-group input {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-surface);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
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

.form-group input:disabled {
  opacity: 0.6;
  background: var(--color-border);
  cursor: not-allowed;
}

.form-hint {
  display: block;
  margin-top: 6px;
  font-size: 11px;
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
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
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

.danger-btn {
  flex: 2;
  padding: 14px;
  border: none;
  border-radius: 10px;
  background: linear-gradient(135deg, var(--color-accent-error) 0%, var(--color-accent-dark-red) 100%);
  color: var(--color-text-inverse);
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 16px var(--color-shadow-error);
}

.danger-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px var(--color-shadow-error-hover);
}

.delete-warning {
  text-align: center;
  padding: 10px 0;
}

.delete-warning svg {
  margin-bottom: 16px;
}

.delete-warning p {
  font-size: 14px;
  color: var(--color-text-primary);
  margin-bottom: 8px;
  line-height: 1.5;
}

.delete-warning span {
  font-size: 13px;
  color: var(--color-text-muted);
}
</style>
