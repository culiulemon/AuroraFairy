import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { LocalModel, ModelType, ModelStatus } from '../stores/localModels'
import { loadLocalModels, addLocalModel, updateLocalModel, removeLocalModel, getDefaultDeployConfig } from '../stores/localModels'
import { loadSettings, saveSettings } from '../stores/settings'

interface EnvironmentStatus {
  python: boolean
  python_version: string | null
  modelscope: boolean
  openvino: boolean
  openvino_version: string | null
  openvino_genai: boolean
  optimum: boolean
  intel_gpu: boolean
}

interface DependencyInstallProgress {
  status: string
  message: string
}

interface DownloadProgress {
  model_id: string
  status: string
  current_file: string
  progress_percent: number
  message: string
}

interface ServerStatus {
  model_id: string
  status: string
  port: number
  message: string
}

export function useModelManager() {
  const environmentStatus = ref<EnvironmentStatus | null>(null)
  const isDownloading = ref(false)
  const downloadProgress = ref<DownloadProgress | null>(null)
  const serverStatuses = ref<Map<string, ServerStatus>>(new Map())
  const models = ref<LocalModel[]>([])

  const currentDownloadDisplayName = ref<string>('')

  let unlistenDownload: UnlistenFn | null = null
  let unlistenServer: UnlistenFn | null = null
  let unlistenDependency: UnlistenFn | null = null

  const installingPackage = ref<string | null>(null)
  const dependencyInstallMessage = ref<string | null>(null)

  const ENV_STORAGE_KEY = 'aurorafairy-env-status'

  function loadCachedEnvironment(): EnvironmentStatus | null {
    try {
      const stored = localStorage.getItem(ENV_STORAGE_KEY)
      if (stored) return JSON.parse(stored)
    } catch { /* ignore */ }
    return null
  }

  function saveEnvironmentStatus(status: EnvironmentStatus) {
    try {
      localStorage.setItem(ENV_STORAGE_KEY, JSON.stringify(status))
    } catch { /* ignore */ }
  }

  async function checkEnvironment() {
    try {
      environmentStatus.value = await invoke<EnvironmentStatus>('check_environment')
      if (environmentStatus.value) {
        saveEnvironmentStatus(environmentStatus.value)
      }
    } catch (e) {
      console.error('Failed to check environment:', e)
    }
  }

  async function downloadModel(modelId: string, displayName: string) {
    isDownloading.value = true
    currentDownloadDisplayName.value = displayName
    downloadProgress.value = null
    try {
      const localDir = `models/${modelId.split('/').pop()}`
      await invoke('download_model', { modelId, localDir })
    } catch (e: any) {
      console.error('Failed to start download:', e)
      downloadProgress.value = {
        model_id: modelId,
        status: 'error',
        current_file: '',
        progress_percent: 0,
        message: String(e)
      }
    }
  }

  async function cancelDownload(modelId: string) {
    try {
      await invoke('cancel_download', { modelId })
    } catch (e) {
      console.error('Failed to cancel download:', e)
    }
  }

  const deployError = ref<string | null>(null)

  async function deployModel(model: LocalModel) {
    deployError.value = null
    try {
      const config = model.deployConfig || getDefaultDeployConfig()
      const modelName = model.localPath.split('/').pop() || model.displayName
      const port = await invoke<number>('deploy_model', {
        modelPath: model.localPath,
        modelName,
        ggufFile: '',
        config
      })
      updateLocalModel(model.id, { status: 'running', port })
      refreshModels()
    } catch (e: any) {
      const msg = String(e)
      console.error('Failed to deploy model:', msg)
      deployError.value = msg
      updateLocalModel(model.id, { status: 'error' })
      refreshModels()
    }
  }

  async function stopModel(modelId: string) {
    const model = models.value.find(m => m.id === modelId)
    if (!model) return
    try {
      await invoke('stop_model', { modelPath: model.localPath })
      updateLocalModel(modelId, { status: 'ready', port: undefined })
      refreshModels()
    } catch (e) {
      console.error('Failed to stop model:', e)
    }
  }

  async function deleteModel(modelId: string) {
    const model = models.value.find(m => m.id === modelId)
    if (!model) return
    try {
      await invoke('delete_model', { localDir: model.localPath })
    } catch (e) {
      console.warn('Delete model directory failed (may not exist):', e)
    }
    removeLocalModel(modelId)
    refreshModels()
  }

  function addAsProvider(model: LocalModel) {
    const settings = loadSettings()
    const rawName = model.localPath.split('/').pop() || model.displayName
    const modelName = rawName
      .split('')
      .filter(c => /[a-zA-Z0-9\-_]/.test(c))
      .join('')
      .toLowerCase()
    const port = model.port || model.deployConfig?.port || 8000
    const newProvider = {
      id: `local-${model.id}`,
      displayName: `${model.displayName} (本地)`,
      baseUrl: `http://127.0.0.1:${port}/v1`,
      apiKey: 'local',
      model: modelName,
      protocol: 'custom' as const,
      supportsTools: false,
    }
    settings.providers = settings.providers || []
    settings.providers.push(newProvider)
    saveSettings(settings)
  }

  function refreshModels() {
    models.value = loadLocalModels()
  }

  async function installDependency(packageName: string) {
    installingPackage.value = packageName
    dependencyInstallMessage.value = null
    try {
      await invoke('install_dependency', { package: packageName })
    } catch (e) {
      console.error('Failed to install dependency:', e)
      installingPackage.value = null
    }
  }

  onMounted(async () => {
    models.value = loadLocalModels()
    environmentStatus.value = loadCachedEnvironment()

    unlistenDownload = await listen<DownloadProgress>('model-download-progress', (event) => {
      downloadProgress.value = event.payload
      if (event.payload.status === 'completed') {
        const modelName = event.payload.model_id.split('/').pop() || event.payload.model_id
        const localPath = `models/${modelName}`
        invoke('get_model_info', { localDir: localPath }).then((info: any) => {
          const newModel: LocalModel = {
            id: crypto.randomUUID(),
            modelId: event.payload.model_id,
            displayName: currentDownloadDisplayName.value || modelName,
            modelType: 'llm' as ModelType,
            localPath,
            sizeBytes: info?.size_bytes || 0,
            status: 'ready' as ModelStatus,
            addedAt: new Date().toISOString(),
            convertedToIR: info?.ir_converted || false,
          }
          addLocalModel(newModel)
          isDownloading.value = false
          downloadProgress.value = null
          currentDownloadDisplayName.value = ''
          refreshModels()
        }).catch((e) => {
          console.error('Failed to get model info:', e)
          addLocalModel({
            id: crypto.randomUUID(),
            modelId: event.payload.model_id,
            displayName: currentDownloadDisplayName.value || modelName,
            modelType: 'llm' as ModelType,
            localPath,
            sizeBytes: 0,
            status: 'ready' as ModelStatus,
            addedAt: new Date().toISOString()
          })
          isDownloading.value = false
          downloadProgress.value = null
          currentDownloadDisplayName.value = ''
          refreshModels()
        })
      } else if (event.payload.status === 'error' || event.payload.status === 'cancelled') {
        isDownloading.value = false
        downloadProgress.value = null
        currentDownloadDisplayName.value = ''
      }
    })

    unlistenServer = await listen<ServerStatus>('model-server-status', (event) => {
      serverStatuses.value.set(event.payload.model_id, event.payload)
      serverStatuses.value = new Map(serverStatuses.value)
      const model = models.value.find(m => m.modelId === event.payload.model_id)
      if (model) {
        const newStatus: ModelStatus = event.payload.status === 'running' ? 'running' : event.payload.status === 'stopped' ? 'ready' : event.payload.status === 'importing' ? 'converting' : event.payload.status === 'imported' ? 'ready' : 'error'
        updateLocalModel(model.id, { status: newStatus, port: event.payload.port })
        if (event.payload.status === 'error' && event.payload.message) {
          deployError.value = `部署失败: ${event.payload.message}`
        }
        refreshModels()
      }
    })

    unlistenDependency = await listen<DependencyInstallProgress>('dependency-install-progress', (event) => {
      dependencyInstallMessage.value = event.payload.message
      if (event.payload.status === 'completed' || event.payload.status === 'error') {
        installingPackage.value = null
        checkEnvironment()
      }
    })
  })

  onUnmounted(() => {
    unlistenDownload?.()
    unlistenServer?.()
    unlistenDependency?.()
  })

  return {
    environmentStatus,
    isDownloading,
    downloadProgress,
    serverStatuses,
    models,
    deployError,
    installingPackage,
    dependencyInstallMessage,
    checkEnvironment,
    downloadModel,
    cancelDownload,
    deployModel,
    stopModel,
    deleteModel,
    addAsProvider,
    refreshModels,
    installDependency,
  }
}
