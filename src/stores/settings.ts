export interface ApiProvider {
  id: string
  displayName: string
  baseUrl: string
  apiKey: string
  model: string
  protocol: 'openai' | 'anthropic' | 'google' | 'custom'
  isDefault?: boolean
  thinkingEnabled?: boolean
  supportsTools?: boolean
  contextWindowSize?: number
}

export interface ApiSettings {
  providers: ApiProvider[]
  defaultProviderId: string | null
  useBackendProxy?: boolean
  fbmEnabled?: boolean
  fbmProviderId?: string | null
  fbmEmbeddingProviderId?: string | null
  fbmConsolidationEnabled?: boolean
  fbmRefineResults?: boolean
  fbmSmartRecall?: boolean
  defaultSearchEngine?: string
  corefileInitialized?: boolean
  fairyName?: string
  userName?: string
  fairyPositioning?: string
  fairyStyle?: string
  fairySupplement?: string
  habitSupplement?: string
  contextRecentRounds?: number
}

const STORAGE_KEY = 'aurorafairy-settings'

const defaultSettings: ApiSettings = {
  providers: [],
  defaultProviderId: null,
  useBackendProxy: true,
  fbmEnabled: false,
  fbmProviderId: null,
  fbmEmbeddingProviderId: null,
  fbmConsolidationEnabled: true,
  fbmRefineResults: true,
  fbmSmartRecall: true,
  defaultSearchEngine: 'bing',
}

export const protocolOptions = [
  { value: 'openai', label: 'OpenAI 兼容 (Chat Completions)' },
  { value: 'anthropic', label: 'Anthropic (Claude)' },
  { value: 'google', label: 'Google (Gemini)' },
  { value: 'custom', label: '自定义协议' }
]

export const providerPresets: Record<string, { baseUrl: string; protocol: 'openai' | 'anthropic' | 'google' }> = {
  openai: {
    baseUrl: 'https://api.openai.com/v1',
    protocol: 'openai'
  },
  anthropic: {
    baseUrl: 'https://api.anthropic.com/v1',
    protocol: 'anthropic'
  },
  google: {
    baseUrl: 'https://generativelanguage.googleapis.com/v1beta',
    protocol: 'google'
  },
  dashscope: {
    baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    protocol: 'openai'
  }
}

export function loadSettings(): ApiSettings {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      const parsed = JSON.parse(stored)
      return { ...defaultSettings, ...parsed }
    }
  } catch (e) {
    console.error('Failed to load settings:', e)
  }
  return { ...defaultSettings }
}

export function saveSettings(settings: Partial<ApiSettings>): void {
  try {
    const current = loadSettings()
    const merged = { ...current, ...settings }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(merged))
  } catch (e) {
    console.error('Failed to save settings:', e)
    throw e
  }
}

export function validateProviderId(id: string): boolean {
  return id.length > 0
}

export interface TtsSettings {
  enabled: boolean
  voice: string
  rate: string
  pitch: string
  volume: string
}

const TTS_STORAGE_KEY = 'aurorafairy-tts-settings'

const defaultTtsSettings: TtsSettings = {
  enabled: false,
  voice: 'zh-CN-XiaoxiaoNeural',
  rate: '+0%',
  pitch: '+0Hz',
  volume: '+0%'
}

export function loadTtsSettings(): TtsSettings {
  try {
    const stored = localStorage.getItem(TTS_STORAGE_KEY)
    if (stored) {
      const parsed = JSON.parse(stored)
      return { ...defaultTtsSettings, ...parsed }
    }
  } catch (e) {
    console.error('Failed to load TTS settings:', e)
  }
  return { ...defaultTtsSettings }
}

export function saveTtsSettings(settings: TtsSettings): void {
  try {
    localStorage.setItem(TTS_STORAGE_KEY, JSON.stringify(settings))
  } catch (e) {
    console.error('Failed to save TTS settings:', e)
    throw e
  }
}

export type ThemeName = 'light' | 'dark'

export interface ThemeColor {
  name: string
  primary: string
  primaryHover: string
  primaryLight: string
}

export const themeColorPresets: ThemeColor[] = [
  { name: '橙色', primary: '#E67E22', primaryHover: '#D35400', primaryLight: '#F39C12' },
  { name: '蓝色', primary: '#3498DB', primaryHover: '#2980B9', primaryLight: '#5DADE2' },
  { name: '绿色', primary: '#27AE60', primaryHover: '#1E8449', primaryLight: '#2ECC71' },
  { name: '紫色', primary: '#9B59B6', primaryHover: '#8E44AD', primaryLight: '#AF7AC5' },
  { name: '深灰', primary: '#34495E', primaryHover: '#2C3E50', primaryLight: '#5D6D7E' },
]

export interface ThemeSettings {
  theme: ThemeName
  primaryColor: string
}

const THEME_STORAGE_KEY = 'aurorafairy-theme-settings'

const defaultThemeSettings: ThemeSettings = {
  theme: 'light',
  primaryColor: '#E67E22'
}

export function loadThemeSettings(): ThemeSettings {
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY)
    if (stored) {
      const parsed = JSON.parse(stored)
      return { ...defaultThemeSettings, ...parsed }
    }
  } catch (e) {
    console.error('Failed to load theme settings:', e)
  }
  return { ...defaultThemeSettings }
}

export function saveThemeSettings(settings: ThemeSettings): void {
  try {
    localStorage.setItem(THEME_STORAGE_KEY, JSON.stringify(settings))
  } catch (e) {
    console.error('Failed to save theme settings:', e)
    throw e
  }
}
