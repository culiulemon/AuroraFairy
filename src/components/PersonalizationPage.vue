<template>
  <div class="personalization-page">
    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="19" y1="12" x2="5" y2="12"></line>
            <polyline points="12,19 5,12 12,5"></polyline>
          </svg>
        </button>
        <h2>个性化</h2>
      </div>
    </div>

    <div class="settings-content">
      <div class="setting-section">
        <div class="section-header">
          <h3>主题</h3>
        </div>

        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-label">深色模式</span>
            <span class="setting-desc">开启后使用深色配色方案</span>
          </div>
          <div class="toggle-wrapper">
            <input
              type="checkbox"
              :checked="currentTheme === 'dark'"
              class="toggle-input"
              @change="handleThemeToggle"
            />
            <div class="toggle-slider" @click="handleThemeToggle"></div>
          </div>
        </div>

        <div class="setting-item theme-color-item">
          <div class="setting-info">
            <span class="setting-label">主题色</span>
          </div>
          <div class="color-presets">
            <button
              v-for="color in themeColorPresets"
              :key="color.primary"
              class="color-preset-btn"
              :class="{ active: currentPrimaryColor === color.primary }"
              :style="{ backgroundColor: color.primary }"
              :title="color.name"
              @click="handleColorChange(color.primary)"
            >
              <svg v-if="currentPrimaryColor === color.primary" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="white" stroke-width="3">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <div class="setting-section">
        <div class="section-header">
          <h3>语音合成</h3>
        </div>

          <div class="setting-item">
            <div class="setting-info">
              <span class="setting-label">自动朗读 AI 回复</span>
              <span class="setting-desc">AI 回复完成后自动语音朗读</span>
            </div>
            <div class="toggle-wrapper">
              <input
                type="checkbox"
                v-model="settings.enabled"
                class="toggle-input"
                @change="saveSettings"
              />
              <div class="toggle-slider" @click="settings.enabled = !settings.enabled; saveSettings()"></div>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <span class="setting-label">语音</span>
            </div>
            <BaseSelect
              :modelValue="settings.voice"
              :options="voiceOptions"
              @update:modelValue="settings.voice = $event ?? 'zh-CN-XiaoxiaoNeural'; saveSettings()"
            />
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <span class="setting-label">语速</span>
              <span class="setting-value">{{ settings.rate }}</span>
            </div>
            <input
              type="range"
              v-model.number="rateValue"
              min="-50"
              max="100"
              step="10"
              class="setting-slider"
              @input="updateRate"
            />
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <span class="setting-label">音调</span>
              <span class="setting-value">{{ settings.pitch }}</span>
            </div>
            <input
              type="range"
              v-model.number="pitchValue"
              min="-50"
              max="50"
              step="5"
              class="setting-slider"
              @input="updatePitch"
            />
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <span class="setting-label">音量</span>
              <span class="setting-value">{{ settings.volume }}</span>
            </div>
            <input
              type="range"
              v-model.number="volumeValue"
              min="-50"
              max="100"
              step="10"
              class="setting-slider"
              @input="updateVolume"
            />
          </div>

          <div class="preview-section">
            <button class="preview-btn" @click="handlePreview" :disabled="isPreviewing">
              <svg v-if="!isPreviewing" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                <polygon points="5,3 19,12 5,21"></polygon>
              </svg>
              <svg v-else class="spinning" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 12a9 9 0 11-6.219-8.56"></path>
              </svg>
              {{ isPreviewing ? '正在朗读...' : '试听' }}
            </button>
          </div>
        </div>
      </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import BaseSelect from './BaseSelect.vue'
import type { SelectOption } from './BaseSelect.vue'
import { invoke } from '@tauri-apps/api/core'
import { loadTtsSettings, saveTtsSettings, type TtsSettings } from '../stores/settings'
import { useTTS } from '../composables/useTTS'
import { useTheme } from '../composables/useTheme'

defineEmits<{
  back: []
}>()

const { theme: currentTheme, primaryColor: currentPrimaryColor, setTheme, setPrimaryColor, themeColorPresets } = useTheme()

function handleThemeToggle() {
  setTheme(currentTheme.value === 'light' ? 'dark' : 'light')
}

function handleColorChange(color: string) {
  setPrimaryColor(color)
}

interface VoiceInfo {
  name: string
  locale: string
  gender: string
  description: string
}

const settings = reactive<TtsSettings>({
  enabled: false,
  voice: 'zh-CN-XiaoxiaoNeural',
  rate: '+0%',
  pitch: '+0Hz',
  volume: '+0%'
})

const voices = ref<VoiceInfo[]>([])

const voiceOptions = computed<SelectOption[]>(() =>
  voices.value.map(v => ({ value: v.name, label: v.description }))
)
const rateValue = ref(0)
const pitchValue = ref(0)
const volumeValue = ref(0)
const isPreviewing = ref(false)
const { play, stop, onStop, offStop } = useTTS()

const resetPreview = () => { isPreviewing.value = false }
onStop(resetPreview)
onUnmounted(() => { offStop(resetPreview) })

onMounted(async () => {
  const saved = loadTtsSettings()
  Object.assign(settings, saved)
  parseSliderValues()
  try {
    const voiceList = await invoke<VoiceInfo[]>('tts_list_voices')
    voices.value = voiceList
  } catch (e) {
    console.error('Failed to load voices:', e)
    voices.value = [
      { name: 'zh-CN-XiaoxiaoNeural', locale: 'zh-CN', gender: 'Female', description: '晓晓 (中文-女声)' },
      { name: 'zh-CN-YunxiNeural', locale: 'zh-CN', gender: 'Male', description: '云希 (中文-男声)' },
      { name: 'zh-CN-YunyangNeural', locale: 'zh-CN', gender: 'Male', description: '云扬 (中文-男声-新闻)' },
      { name: 'en-US-JennyNeural', locale: 'en-US', gender: 'Female', description: 'Jenny (English-Female)' },
      { name: 'en-US-GuyNeural', locale: 'en-US', gender: 'Male', description: 'Guy (English-Male)' }
    ]
  }
})

function parseSliderValues() {
  rateValue.value = parseInt(settings.rate) || 0
  pitchValue.value = parseInt(settings.pitch) || 0
  volumeValue.value = parseInt(settings.volume) || 0
}

function updateRate() {
  const val = rateValue.value
  settings.rate = val >= 0 ? `+${val}%` : `${val}%`
  saveSettings()
}

function updatePitch() {
  const val = pitchValue.value
  settings.pitch = val >= 0 ? `+${val}Hz` : `${val}Hz`
  saveSettings()
}

function updateVolume() {
  const val = volumeValue.value
  settings.volume = val >= 0 ? `+${val}%` : `${val}%`
  saveSettings()
}

function saveSettings() {
  saveTtsSettings({ ...settings })
}

async function handlePreview() {
  if (isPreviewing.value) {
    stop()
    return
  }
  stop()
  isPreviewing.value = true
  try {
    const base64 = await invoke<string>('tts_generate', {
      text: '你好，我是Fairy，协议已签订，主人！',
      voice: settings.voice,
      rate: settings.rate,
      pitch: settings.pitch,
      volume: settings.volume
    })
    play(base64)
  } catch (e) {
    console.error('Preview failed:', e)
    isPreviewing.value = false
  }
}
</script>

<style scoped>
.personalization-page {
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
  overflow-y: auto;
  padding: 24px 28px;
}

.section-header {
  margin-bottom: 20px;
}

.section-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border: 1px solid var(--color-border);
  border-radius: 14px;
  background: var(--color-surface);
  margin-bottom: 12px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.setting-item:hover {
  border-color: var(--color-border-light);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px var(--color-shadow-alpha-08);
}

.theme-color-item {
  flex-direction: column;
  align-items: flex-start;
}

.theme-color-item .setting-info {
  width: 100%;
  margin-bottom: 12px;
}

.color-presets {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.color-preset-btn {
  width: 40px;
  height: 40px;
  border: 3px solid transparent;
  border-radius: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.color-preset-btn:hover {
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.color-preset-btn.active {
  border-color: var(--color-text-primary);
  transform: scale(1.1);
}

.setting-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.setting-desc {
  font-size: 11px;
  color: var(--color-text-muted);
}

.setting-value {
  font-size: 12px;
  color: var(--color-primary);
  font-weight: 600;
  min-width: 40px;
  text-align: right;
}

.setting-select {
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  font-size: 13px;
  color: var(--color-text-primary);
  background: var(--color-surface-card);
  cursor: pointer;
  min-width: 200px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.setting-select:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.setting-slider {
  width: 160px;
  height: 6px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--color-border-light);
  border-radius: 3px;
  outline: none;
  cursor: pointer;
}

.setting-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--color-primary-gradient);
  cursor: pointer;
  box-shadow: 0 2px 6px var(--color-shadow-primary-strong);
  transition: transform 0.15s;
}

.setting-slider::-webkit-slider-thumb:hover {
  transform: scale(1.15);
}

.setting-slider::-moz-range-thumb {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--color-primary-gradient);
  cursor: pointer;
  border: none;
  box-shadow: 0 2px 6px var(--color-shadow-primary-strong);
}

.toggle-wrapper {
  position: relative;
  width: 48px;
  height: 26px;
  flex-shrink: 0;
}

.toggle-input {
  opacity: 0;
  width: 0;
  height: 0;
  position: absolute;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--color-border-medium);
  transition: 0.3s;
  border-radius: 26px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 20px;
  width: 20px;
  left: 3px;
  bottom: 3px;
  background-color: var(--color-text-inverse);
  transition: 0.3s;
  border-radius: 50%;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.toggle-input:checked + .toggle-slider {
  background: var(--color-primary-gradient);
}

.toggle-input:checked + .toggle-slider:before {
  transform: translateX(22px);
}

.preview-section {
  margin-top: 20px;
  display: flex;
  justify-content: center;
}

.preview-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 28px;
  border: none;
  background: var(--color-primary-gradient);
  border-radius: 12px;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-inverse);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 16px var(--color-shadow-primary-hover);
}

.preview-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px var(--color-shadow-primary-strong);
}

.preview-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
