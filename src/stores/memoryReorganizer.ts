import { ref, type Ref } from 'vue'
import type { ReorganizationResult, ReorganizationProgress } from '../fbm/src/types/reorganization.js'
import { MemoryReorganizer } from '../fbm/src/core/memory-reorganizer.js'

const IDLE_START_THRESHOLD = 30 * 60 * 1000

const isReorganizing = ref(false)
const isPaused = ref(false)
const progress: Ref<ReorganizationProgress | null> = ref(null)

let reorganizer: MemoryReorganizer | null = null
let idleTimer: ReturnType<typeof setTimeout> | null = null

function shouldReorganize(): boolean {
  if (isReorganizing.value) return false
  if (MemoryReorganizer.hasCheckpoint()) return true
  return MemoryReorganizer.isDirty()
}

function resetIdleTimer(): void {
  if (idleTimer) clearTimeout(idleTimer)
  if (isReorganizing.value) return
  idleTimer = setTimeout(() => {
    if (shouldReorganize()) {
      startReorganization()
    }
  }, IDLE_START_THRESHOLD)
}

function onConversationSent(): void {
  if (isReorganizing.value) {
    cancelReorganization()
  }
  resetIdleTimer()
}

async function startReorganization(): Promise<ReorganizationResult | null> {
  if (isReorganizing.value || !reorganizer) return null
  isReorganizing.value = true
  isPaused.value = false

  const result = await reorganizer.run()

  isReorganizing.value = false
  isPaused.value = false
  progress.value = null

  return result
}

function cancelReorganization(): void {
  if (!reorganizer) return
  reorganizer.cancel()
  isPaused.value = false
}

function markDirty(): void {
  MemoryReorganizer.markDirty()
}

function initScheduler(reorg: MemoryReorganizer): void {
  reorganizer = reorg
  resetIdleTimer()
}

function destroyScheduler(): void {
  if (idleTimer) clearTimeout(idleTimer)
  cancelReorganization()
  reorganizer = null
}

function updateProgress(p: ReorganizationProgress | null): void {
  progress.value = p
}

export const memoryReorganizer = {
  isReorganizing,
  isPaused,
  progress,

  init: initScheduler,
  destroy: destroyScheduler,
  start: startReorganization,
  cancel: cancelReorganization,
  onConversationSent,
  markDirty,
  shouldReorganize,
  updateProgress,
}
