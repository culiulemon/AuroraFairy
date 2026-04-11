import { loadSettings } from './settings.js'
import type { ApiProvider } from './settings.js'
import type { EmbeddingAdapter, LLMAdapter } from '../fbm/src/types/adapter.js'
import { loadMiscSettings, getEffectiveWorkingDir } from './miscSettings.js'
import { invoke } from '@tauri-apps/api/core'

import soulRaw from '../fbm/templates/SOUL.md?raw'
import syspromptRaw from '../fbm/templates/SYSPROMPT.md?raw'
import habitRaw from '../fbm/templates/HABIT.md?raw'
import aboutuserRaw from '../fbm/templates/ABOUTUSER.md?raw'
import rebirthRaw from '../fbm/templates/REBIRTH.md?raw'

const COREFILE_TEMPLATES: Record<string, string> = {
  'SOUL.md': soulRaw,
  'SYSPROMPT.md': syspromptRaw,
  'HABIT.md': habitRaw,
  'ABOUTUSER.md': aboutuserRaw,
  'REBIRTH.md': rebirthRaw,
}

let fbm: any = null
let initPromise: Promise<void> | null = null
let fbmModule: any = null
let fbmLoadFailed = false
let lastConsolidatedMsgCount = 0
let lastConsolidatedConvId: string | null = null
let lastRetrieveKeywords: string[] = []

function findProvider(providerId: string | null | undefined): ApiProvider | undefined {
  if (!providerId) return undefined
  const settings = loadSettings()
  return settings.providers.find(p => p.id === providerId)
}

function processCorefileTemplate(content: string, variables: Record<string, string>): string {
  let result = content
  for (const [key, value] of Object.entries(variables)) {
    result = result.split(`{{${key}}}`).join(value)
  }
  return result
}

function getTemplateVariables(): Record<string, string> {
  const s = loadSettings()
  return {
    FairyName: s.fairyName || 'Fairy',
    User: s.userName || '主人',
    user: s.userName || '主人',
    Positioning: s.fairyPositioning || '智能助手',
    Style: s.fairyStyle || '温柔体贴',
    Supplement: s.fairySupplement || '',
    HabitSupplement: s.habitSupplement || '',
  }
}

async function loadFBMModule(): Promise<typeof fbmModule> {
  if (fbmModule) return fbmModule
  if (fbmLoadFailed) {
    fbmLoadFailed = false
  }

  try {
    fbmModule = await import('../fbm/src/index.js')
    return fbmModule
  } catch (err) {
    console.warn('[FBM] Failed to load FBM module:', err)
    fbmLoadFailed = true
    return null
  }
}

async function doInit(
  settings: ReturnType<typeof loadSettings>,
  provider: NonNullable<ReturnType<typeof findProvider>>,
  embProvider: NonNullable<ReturnType<typeof findProvider>>,
): Promise<void> {
  try {
    console.log('[FBM] Initializing...')
    const mod = await loadFBMModule()
    if (!mod) {
      console.warn('[FBM] Failed to load FBM module')
      return
    }

    const miscSettings = await loadMiscSettings()
    const baseDir = getEffectiveWorkingDir(miscSettings)
    const memoryDir = `${baseDir}/memories`
    console.log('[FBM] Memory dir:', memoryDir)

    console.log('[FBM] Starting Qdrant...')
    const qdrantPort = await invoke<number>('qdrant_start', { workingDir: baseDir })
    console.log('[FBM] Qdrant started on port:', qdrantPort)

    const llm = new mod.OpenAILLMAdapter({
      baseUrl: provider.baseUrl,
      apiKey: provider.apiKey,
      model: provider.model,
    })

    const embedding = new mod.OpenAIEmbeddingAdapter({
      baseUrl: embProvider.baseUrl,
      apiKey: embProvider.apiKey,
      model: embProvider.model,
    })

    const config = {
      memoryDir,
      qdrant: {
        port: qdrantPort,
      },
      retrieval: {
        refineResults: settings.fbmRefineResults !== false,
        retrievalTopK: 10,
        minScore: 0.3, //向量最小匹配参数，越小越广泛
        directoryThreshold: 50,
      },
      embedding: {
        batchSize: 20,
      },
      lifecycle: {
        enableExpiration: true,
        mergeCheckInterval: 10,
      },
    }

    fbm = new mod.FBM(config, llm, embedding)
    await fbm.init()

    console.log('[FBM] Initialization complete, FBM is ready. embedding dimension:', embedding.getDimension())
  } catch (err) {
    console.warn('[FBM] Initialization failed:', err)
    fbm = null
  }
}

export async function getEmbeddingAdapter(): Promise<EmbeddingAdapter | null> {
  const settings = loadSettings()
  const embProviderId = settings.fbmEmbeddingProviderId
  if (!embProviderId) return null

  const provider = settings.providers.find(p => p.id === embProviderId)
  if (!provider) return null

  const { OpenAIEmbeddingAdapter } = await import('../fbm/src/core/adapters/openai-embedding.js')
  return new OpenAIEmbeddingAdapter({
    baseUrl: provider.baseUrl,
    apiKey: provider.apiKey,
    model: provider.model,
  })
}

export async function getLLMAdapter(): Promise<LLMAdapter | null> {
  const settings = loadSettings()
  const providerId = settings.fbmProviderId || settings.defaultProviderId
  if (!providerId) return null

  const provider = settings.providers.find(p => p.id === providerId)
  if (!provider) return null

  const { OpenAILLMAdapter } = await import('../fbm/src/core/adapters/openai-llm.js')
  return new OpenAILLMAdapter({
    baseUrl: provider.baseUrl,
    apiKey: provider.apiKey,
    model: provider.model,
  })
}

export const fbmStore = {
  isReady(): boolean {
    return fbm?.initialized ?? false
  },

  isEnabled(): boolean {
    const settings = loadSettings()
    return settings.fbmEnabled === true
  },

  async ensureInit(): Promise<void> {
    if (fbm?.initialized) return
    if (initPromise) return initPromise

    const settings = loadSettings()
    if (!settings.fbmEnabled) return

    const provider = findProvider(settings.fbmProviderId)
    if (!provider) {
      console.warn('[FBM] Provider not found, skipping initialization')
      return
    }

    const embProvider = findProvider(settings.fbmEmbeddingProviderId)
    if (!embProvider) {
      console.warn('[FBM] Embedding provider not configured, memory cannot work without embedding')
      return
    }

    initPromise = doInit(settings, provider, embProvider)
    try {
      await initPromise
    } finally {
      initPromise = null
    }

    if (fbm?.initialized) {
      await this.initCoreFiles()
    }
  },

  async shutdown(): Promise<void> {
    if (!fbm) return
    try {
      await fbm.shutdown()
    } catch (err) {
      console.warn('[FBM] Shutdown error:', err)
    }
    fbm = null
    try {
      await invoke('qdrant_stop')
    } catch (err) {
      console.warn('[FBM] Qdrant stop error:', err)
    }
  },

  async reinitialize(): Promise<void> {
    await this.shutdown()
    await this.ensureInit()
  },

  async setWorkingDir(baseDir: string): Promise<void> {
    if (!fbm) return
    try {
      await fbm.setBaseDir(baseDir)
      console.log('[FBM] Working directory updated to:', baseDir)
    } catch (err) {
      console.warn('[FBM] setWorkingDir error:', err)
    }
  },

  async retrieve(query: string | string[]): Promise<any> {
    if (!this.isReady()) {
      await this.ensureInit()
    }
    if (!fbm) return null

    try {
      const result = await fbm.retrieve(query)
      lastRetrieveKeywords = result?.keywords ?? []
      return result
    } catch (err) {
      console.warn('[FBM] retrieve error:', err)
      lastRetrieveKeywords = []
      return null
    }
  },

  getLastRetrieveKeywords(): string[] {
    return lastRetrieveKeywords
  },

  async consolidate(
    messages: Array<{ role: string; content: string; timestamp?: number }>,
    conversationId?: string,
  ): Promise<{ created: number; updated: number; deleted: number; skipped: number } | null> {
    if (!this.isReady()) {
      try {
        await this.ensureInit()
      } catch (e) {
        console.warn('[FBM] consolidate: ensureInit failed:', e)
        return null
      }
    }
    if (!fbm) return null

    try {
      if (conversationId && conversationId !== lastConsolidatedConvId) {
        lastConsolidatedConvId = conversationId
        lastConsolidatedMsgCount = 0
      }
      const newMessages = messages.slice(lastConsolidatedMsgCount)
      if (newMessages.length === 0) return { created: 0, updated: 0, deleted: 0, skipped: 0 }
      lastConsolidatedMsgCount = messages.length
      const result = await fbm.consolidate(newMessages)
      console.log('[FBM] consolidate result:', result)
      return result
    } catch (err) {
      console.warn('[FBM] consolidate error:', err)
      return null
    }
  },

  resetConsolidationState(): void {
    lastConsolidatedMsgCount = 0
    lastConsolidatedConvId = null
  },

  async reindexVectors(): Promise<number> {
    if (!this.isReady()) return 0
    try {
      return await fbm!.reindexVectors()
    } catch (err) {
      console.warn('[FBM] reindexVectors error:', err)
      return 0
    }
  },

  async clearVectors(): Promise<void> {
    if (!this.isReady()) return
    try {
      await fbm!.clearVectors()
    } catch (err) {
      console.warn('[FBM] clearVectors error:', err)
    }
  },

  async getMemoryStats(): Promise<{ totalMemories: number; vectorCount: number; enabled: boolean }> {
    const enabled = this.isEnabled()
    if (!fbm) {
      console.warn('[FBM] getMemoryStats: fbm is null')
      return { totalMemories: 0, vectorCount: 0, enabled }
    }

    try {
      const stats = await fbm.getStats()
      console.log('[FBM] getMemoryStats raw:', stats)
      return {
        totalMemories: stats?.uniqueBlocks ?? 0,
        vectorCount: stats?.directoryEntries ?? 0,
        enabled,
      }
    } catch (err) {
      console.warn('[FBM] getMemoryStats error:', err)
      return { totalMemories: 0, vectorCount: 0, enabled }
    }
  },

  async listDirectory(): Promise<Array<{ blockId: string; title: string; summary: string; keywords: string[] }>> {
    if (!fbm) {
      console.warn('[FBM] listDirectory: fbm is null')
      return []
    }
    try {
      const dirManager = fbm.getDirectoryManager()
      const entries = await dirManager.getAllEntries()
      const seen = new Set<string>()
      const result: Array<{ blockId: string; title: string; summary: string; keywords: string[] }> = []
      for (const e of entries) {
        if (seen.has(e.blockId)) continue
        seen.add(e.blockId)
        result.push({
          blockId: e.blockId,
          title: e.directoryEntry || '未命名',
          summary: e.summary || '',
          keywords: e.keywordAnchors || [],
        })
      }
      return result
    } catch (err) {
      console.warn('[FBM] listDirectory error:', err)
      return []
    }
  },

  async getBlockDetail(blockId: string): Promise<string | null> {
    if (!fbm) return null
    try {
      const store = fbm.getStore()
      const blockData = await store.assembleBlockData(blockId)
      if (!blockData) return null
      const parts: string[] = []
      if (blockData.summary) parts.push(`## 摘要\n${blockData.summary}`)
      if (blockData.rawContents?.length) parts.push(`## 原始上下文\n${blockData.rawContents.join('\n')}`)
      if (blockData.keywordSentences?.length) parts.push(`## 关键词\n${blockData.keywordSentences.join(', ')}`)
      return parts.join('\n\n') || '空区块'
    } catch (err) {
      console.warn('[FBM] getBlockDetail error:', err)
      return null
    }
  },

  async deleteBlock(blockId: string): Promise<void> {
    if (!fbm) return
    try {
      const store = fbm.getStore()
      await store.deleteBlock(blockId)
    } catch (err) {
      console.warn('[FBM] deleteBlock error:', err)
      throw err
    }
  },

  async getCorefileDir(): Promise<string> {
    const miscSettings = await loadMiscSettings()
    const baseDir = getEffectiveWorkingDir(miscSettings)
    return `${baseDir}/memories/corefile`
  },

  async initCoreFiles(force = false): Promise<void> {
    const corefileDir = await this.getCorefileDir()
    const skipAutoRecover = ['REBIRTH.md']
    const variables = getTemplateVariables()
    try {
      await invoke('fbm_mkdir', { path: corefileDir })

      for (const [fileName, content] of Object.entries(COREFILE_TEMPLATES)) {
        const filePath = `${corefileDir}/${fileName}`
        const exists = await invoke<boolean>('fbm_exists', { path: filePath }).catch(() => false)
        if (force || (!exists && !skipAutoRecover.includes(fileName))) {
          const processed = processCorefileTemplate(content, variables)
          await invoke('fbm_write_file', { path: filePath, content: processed })
        }
      }

      console.log('[FBM] CoreFiles initialized at:', corefileDir)
    } catch (err) {
      console.warn('[FBM] initCoreFiles error:', err)
    }
  },

  async refreshCoreFiles(): Promise<void> {
    const corefileDir = await this.getCorefileDir()
    const variables = getTemplateVariables()
    try {
      for (const [fileName, content] of Object.entries(COREFILE_TEMPLATES)) {
        const filePath = `${corefileDir}/${fileName}`
        const exists = await invoke<boolean>('fbm_exists', { path: filePath }).catch(() => false)
        if (exists) {
          const processed = processCorefileTemplate(content, variables)
          await invoke('fbm_write_file', { path: filePath, content: processed })
        }
      }
    } catch (err) {
      console.warn('[FBM] refreshCoreFiles error:', err)
    }
  },

  async readCoreFile(fileName: string): Promise<string | null> {
    try {
      const corefileDir = await this.getCorefileDir()
      return await invoke<string>('fbm_read_file', { path: `${corefileDir}/${fileName}` })
    } catch {
      return COREFILE_TEMPLATES[fileName] ?? null
    }
  },
}
