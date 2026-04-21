export type ModelType = 'llm' | 'embedding' | 'tts' | 'other'
export type ModelStatus = 'downloading' | 'converting' | 'ready' | 'running' | 'error' | 'stopping'
export type InferenceBackend = 'openvino' | 'llama-cpp' | 'tensorrt-llm' | 'transformers'
export type ModelFormat = 'gguf' | 'openvino-ir' | 'safetensors' | 'unknown'

export interface LocalModel {
  id: string
  modelId: string
  displayName: string
  modelType: ModelType
  localPath: string
  sizeBytes: number
  status: ModelStatus
  port?: number
  deployedAt?: string
  addedAt: string
  modelFormat?: ModelFormat
  convertedToIR?: boolean
  irPath?: string
  deployConfig?: DeployConfig
}

export interface DeployConfig {
  ctxSize: number
  threads: number
  device: string
  port: number
  backend: InferenceBackend
}

const STORAGE_KEY = 'aurorafairy-local-models'

const defaultDeployConfig: DeployConfig = {
  ctxSize: 2048,
  threads: 4,
  device: 'GPU',
  port: 0,
  backend: 'llama-cpp'
}

export function getDefaultDeployConfig(): DeployConfig {
  return { ...defaultDeployConfig }
}

export function getBackendForFormat(format?: ModelFormat): InferenceBackend {
  switch (format) {
    case 'gguf': return 'llama-cpp'
    case 'openvino-ir': return 'openvino'
    case 'safetensors': return 'transformers'
    default: return 'llama-cpp'
  }
}

export function loadLocalModels(): LocalModel[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      const parsed = JSON.parse(stored)
      if (Array.isArray(parsed)) {
        return parsed
      }
    }
  } catch (e) {
    console.error('Failed to load local models:', e)
  }
  return []
}

export function saveLocalModels(models: LocalModel[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(models))
  } catch (e) {
    console.error('Failed to save local models:', e)
    throw e
  }
}

export function addLocalModel(model: LocalModel): void {
  const models = loadLocalModels()
  if (models.some(m => m.localPath === model.localPath)) {
    return
  }
  models.push(model)
  saveLocalModels(models)
}

export function updateLocalModel(id: string, updates: Partial<LocalModel>): void {
  const models = loadLocalModels()
  const index = models.findIndex(m => m.id === id)
  if (index !== -1) {
    models[index] = { ...models[index], ...updates }
    saveLocalModels(models)
  }
}

export function removeLocalModel(id: string): void {
  const models = loadLocalModels()
  const filtered = models.filter(m => m.id !== id)
  saveLocalModels(filtered)
}

export function getLocalModel(id: string): LocalModel | undefined {
  const models = loadLocalModels()
  return models.find(m => m.id === id)
}
