import { ref } from 'vue'
import { emit } from '@tauri-apps/api/event'

export type LogLevel = 'request' | 'response' | 'error' | 'info'

export interface DebugLog {
  id: string
  timestamp: number
  level: LogLevel
  title: string
  content: string
  meta?: Record<string, unknown>
}

const logs = ref<DebugLog[]>([])
const maxLogs = 500

let idCounter = 0

function generateId(): string {
  idCounter++
  return `log_${Date.now()}_${idCounter}`
}

export function addDebugLog(level: LogLevel, title: string, content: string, meta?: Record<string, unknown>) {
  const log: DebugLog = {
    id: generateId(),
    timestamp: Date.now(),
    level,
    title,
    content,
    meta
  }

  logs.value.push(log)

  if (logs.value.length > maxLogs) {
    logs.value = logs.value.slice(-maxLogs)
  }

  try {
    emit('debug-log', log)
  } catch {
    // debug window may not be open
  }
}

export function clearDebugLogs() {
  logs.value = []
  idCounter = 0
}

export function useDebugStore() {
  return {
    logs
  }
}
