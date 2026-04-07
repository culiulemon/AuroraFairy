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
  ollama: boolean
  ollama_version: string | null
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

interface InstallProgress {
  status: string
  message: string
}

export function useModelManager() {
  const environmentStatus = ref<EnvironmentStatus | null>(null)
  const isDownloading = ref(false)
  const downloadProgress = ref<DownloadProgress | null>(null)
  const serverStatuses = ref<Map<string, ServerStatus>>(new Map())
  const models = ref<LocalModel[]>([])

  const currentDownloadDisplayName = ref<string>('')

  const isInstallingOllama = ref(false)
  const ollamaInstallProgress = ref<InstallProgress | null>(null)

  let unlistenDownload: UnlistenFn | null = null
  let unlistenServer: UnlistenFn | null = null
  let unlistenOllamaInstall: UnlistenFn | null = null

  async function checkEnvironment() {
    try {
      environmentStatus.value = await invoke<EnvironmentStatus>('check_environment')
    } catch (e) {
      console.error('Failed to check environment:', e)
      environmentStatus.value = null
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
      await invoke('deploy_model', {
        modelPath: model.localPath,
        modelName,
        ggufFile: '',
        config
      })
      updateLocalModel(model.id, { status: 'running', port: 11434 })
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
    const newProvider = {
      id: `local-${model.id}`,
      displayName: `${model.displayName} (本地)`,
      baseUrl: 'http://127.0.0.1:11434/v1',
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

  async function installOllama() {
    isInstallingOllama.value = true
    ollamaInstallProgress.value = null
    try {
      await invoke('install_ollama')
    } catch (e) {
      console.error('Failed to start Ollama install:', e)
      isInstallingOllama.value = false
      ollamaInstallProgress.value = null
    }
  }

  async function getOllamaModels(): Promise<any> {
    try {
      return await invoke('get_ollama_models')
    } catch (e) {
      console.error('Failed to get Ollama models:', e)
      return null
    }
  }

  onMounted(async () => {
    models.value = loadLocalModels()
    await checkEnvironment()

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
            addedAt: new Date().toISOString()
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
      }
    })

    unlistenServer = await listen<ServerStatus>('model-server-status', (event) => {
      serverStatuses.value.set(event.payload.model_id, event.payload)
      serverStatuses.value = new Map(serverStatuses.value)
      const model = models.value.find(m => m.modelId === event.payload.model_id)
      if (model) {
        const newStatus: ModelStatus = event.payload.status === 'running' ? 'running' : event.payload.status === 'stopped' ? 'ready' : event.payload.status === 'importing' ? 'downloading' : 'error'
        updateLocalModel(model.id, { status: newStatus, port: event.payload.port })
        if (event.payload.status === 'error' && event.payload.message) {
          deployError.value = `部署失败: ${event.payload.message}`
        }
        refreshModels()
      }
    })

    unlistenOllamaInstall = await listen<InstallProgress>('ollama-install-progress', (event) => {
      ollamaInstallProgress.value = event.payload
      if (event.payload.status === 'completed') {
        isInstallingOllama.value = false
        ollamaInstallProgress.value = null
        checkEnvironment()
      } else if (event.payload.status === 'error') {
        isInstallingOllama.value = false
      }
    })
  })

  onUnmounted(() => {
    unlistenDownload?.()
    unlistenServer?.()
    unlistenOllamaInstall?.()
  })

  return {
    environmentStatus,
    isDownloading,
    downloadProgress,
    serverStatuses,
    models,
    isInstallingOllama,
    ollamaInstallProgress,
    deployError,
    checkEnvironment,
    downloadModel,
    cancelDownload,
    deployModel,
    stopModel,
    deleteModel,
    addAsProvider,
    refreshModels,
    installOllama,
    getOllamaModels,
  }
}
