import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { loadTtsSettings } from '../stores/settings'

const isPlaying = ref(false)
let currentAudio: HTMLAudioElement | null = null
let generationAbortController: AbortController | null = null
let currentMessageId: string | null = null

type StopCallback = () => void
type StartCallback = (messageId: string) => void
const stopCallbacks: Set<StopCallback> = new Set()
const startCallbacks: Set<StartCallback> = new Set()

function stripMarkdown(text: string): string {
  return text
    .replace(/```[\s\S]*?```/g, '')
    .replace(/`[^`]+`/g, '')
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, '')
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/#{1,6}\s+/g, '')
    .replace(/\*\*\*(.+?)\*\*\*/g, '$1')
    .replace(/\*\*(.+?)\*\*/g, '$1')
    .replace(/\*(.+?)\*/g, '$1')
    .replace(/~~(.+?)~~/g, '$1')
    .replace(/__(.+?)__/g, '$1')
    .replace(/_(.+?)_/g, '$1')
    .replace(/^>\s+/gm, '')
    .replace(/^[-*+]\s+/gm, '')
    .replace(/^\d+\.\s+/gm, '')
    .replace(/^---+$/gm, '')
    .replace(/\|\|/g, ' ')
    .replace(/[|]/g, ' ')
    .replace(/\n{2,}/g, '\n')
    .trim()
}

export function useTTS() {
  async function synthesize(text: string, _signal?: AbortSignal): Promise<string> {
    const settings = loadTtsSettings()
    const base64Audio = await invoke<string>('tts_generate', {
      text,
      voice: settings.voice,
      rate: settings.rate,
      pitch: settings.pitch,
      volume: settings.volume
    })
    return base64Audio
  }

  function play(base64Audio: string): void {
    stop()
    const byteCharacters = atob(base64Audio)
    const byteNumbers = new Array(byteCharacters.length)
    for (let i = 0; i < byteCharacters.length; i++) {
      byteNumbers[i] = byteCharacters.charCodeAt(i)
    }
    const byteArray = new Uint8Array(byteNumbers)
    const blob = new Blob([byteArray], { type: 'audio/mpeg' })
    const url = URL.createObjectURL(blob)

    currentAudio = new Audio(url)
    currentAudio.onended = () => {
      isPlaying.value = false
      currentMessageId = null
      URL.revokeObjectURL(url)
      currentAudio = null
      notifyStop()
    }
    currentAudio.onerror = () => {
      isPlaying.value = false
      currentMessageId = null
      URL.revokeObjectURL(url)
      currentAudio = null
      notifyStop()
    }
    isPlaying.value = true
    currentAudio.play().catch(() => {
      isPlaying.value = false
      currentMessageId = null
      URL.revokeObjectURL(url)
      currentAudio = null
      notifyStop()
    })
  }

  function notifyStop() {
    stopCallbacks.forEach(cb => cb())
  }

  function notifyStart(messageId: string) {
    startCallbacks.forEach(cb => cb(messageId))
  }

  function stop(): void {
    if (currentAudio) {
      currentAudio.pause()
      currentAudio.currentTime = 0
      const url = currentAudio.src
      currentAudio = null
      URL.revokeObjectURL(url)
    }
    if (generationAbortController) {
      generationAbortController.abort()
      generationAbortController = null
    }
    isPlaying.value = false
    currentMessageId = null
    notifyStop()
  }

  async function speak(rawText: string, messageId?: string): Promise<void> {
    if (!rawText || rawText.trim().length === 0) return
    stop()
    const text = stripMarkdown(rawText)
    if (!text || text.trim().length === 0) return
    currentMessageId = messageId || null
    notifyStart(messageId || '')
    try {
      generationAbortController = new AbortController()
      const base64Audio = await synthesize(text, generationAbortController.signal)
      generationAbortController = null
      play(base64Audio)
    } catch (e) {
      if ((e as Error).name !== 'AbortError') {
        console.error('TTS synthesis failed:', e)
      }
      isPlaying.value = false
      currentMessageId = null
      generationAbortController = null
      notifyStop()
    }
  }

  function onStop(cb: StopCallback) {
    stopCallbacks.add(cb)
  }

  function offStop(cb: StopCallback) {
    stopCallbacks.delete(cb)
  }

  function onStart(cb: StartCallback) {
    startCallbacks.add(cb)
  }

  function offStart(cb: StartCallback) {
    startCallbacks.delete(cb)
  }

  function getCurrentMessageId(): string | null {
    return currentMessageId
  }

  return {
    isPlaying,
    synthesize,
    play,
    stop,
    speak,
    stripMarkdown,
    onStop,
    offStop,
    onStart,
    offStart,
    getCurrentMessageId
  }
}
