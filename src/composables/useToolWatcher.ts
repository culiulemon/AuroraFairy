import { onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export function useToolWatcher(onChange: (changedFiles: string[]) => Promise<void>) {
  let unlisten: UnlistenFn | null = null

  const start = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('start_tool_watcher')
    } catch (e) {
      console.error('[useToolWatcher] 启动监听失败:', e)
    }

    unlisten = await listen<string[]>('fairy-tool-changed', async (event) => {
      console.log('[useToolWatcher] 检测到变更:', event.payload)
      await onChange(event.payload)
    })
  }

  const stop = () => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }

  onUnmounted(() => {
    stop()
  })

  return { start, stop }
}
