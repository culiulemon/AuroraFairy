import type { FairyDo } from './fairyDo'
import { fairyTask } from './fairyTask'
import { executeSubAgent } from './subAgent'
import type { Tool } from '../types/tool'
import type { TaskItem, TaskStatus } from '../types/task'
import { invoke } from '@tauri-apps/api/core'
import { fbmStore } from '../stores/fbmStore'
import { loadSettings, saveSettings } from '../stores/settings'

export interface VirtualHandlerDeps {
  getCurrentTools: () => Tool[]
  getCurrentProviderId: () => string
}

let currentUserMessage = ''
let recentUserMessages: string[] = []
let conversationContext = ''

export function setCurrentUserMessage(message: string): void {
  currentUserMessage = message
}

export function setRecentUserMessages(messages: string[]): void {
  recentUserMessages = messages
}

export function setConversationContext(context: string): void {
  conversationContext = context
}

export function registerVirtualHandlers(
  fairyDo: FairyDo,
  deps: VirtualHandlerDeps
): void {
  fairyDo.registerVirtualHandler('todo_write', async (input) => {
    const action = input.action as string
    if (action === 'create') {
      const taskName = input.task_name as string
      const todosRaw = input.todos as Array<Record<string, unknown>>
      const todos: TaskItem[] = (todosRaw || []).map((t, i) => ({
        id: String(t.id || i + 1),
        content: String(t.content || ''),
        status: (t.status as TaskStatus) || 'pending',
        priority: (t.priority as TaskItem['priority']) || 'medium'
      }))
      const folder = await fairyTask.createTask(taskName, todos)
      const parsed = await fairyTask.readTaskList()
      return JSON.stringify({ rawContent: parsed.rawContent, folderPath: folder.folderPath })
    } else if (action === 'update') {
      const taskId = input.task_id as string
      const newStatus = input.status as TaskStatus
      if (!taskId || !newStatus) {
        return '错误: update 操作需要 task_id 和 status 参数'
      }
      const updated = await fairyTask.updateTaskStatus(taskId, newStatus)
      const currentFolder = fairyTask.getCurrentFolder()
      const summary = input.summary as string | undefined
      if (summary) {
        await fairyTask.appendSummary(summary)
        const withSummary = await fairyTask.readTaskList()
        return JSON.stringify({ rawContent: withSummary.rawContent, folderPath: currentFolder?.folderPath || '' })
      }
      return JSON.stringify({ rawContent: updated.rawContent, folderPath: currentFolder?.folderPath || '' })
    }
    return '错误: 未知的 action，支持 create 或 update'
  })

  fairyDo.registerVirtualHandler('task_dispatch', async (input, signal) => {
    const description = input.description as string
    if (!description) {
      return JSON.stringify({ success: false, error: '缺少 description 参数' })
    }
    const toolsWhitelist = input.tools as string[] | undefined
    const maxIterations = input.max_iterations as number | undefined

    let tools = deps.getCurrentTools()
    if (toolsWhitelist && toolsWhitelist.length > 0) {
      tools = tools.filter(t => toolsWhitelist.includes(t.invokeName))
    }

    const result = await executeSubAgent({
      providerId: deps.getCurrentProviderId(),
      description,
      tools,
      maxIterations: maxIterations || 20,
      timeout: 300000,
      signal
    })

    return JSON.stringify({
      success: result.success,
      summary: result.summary,
      iterations: result.iterations,
      error: result.error
    }, null, 2)
  })

  fairyDo.registerVirtualHandler('browser', async (input, _signal) => {
    let action = input.action as string

    if (action && !['start', 'stop', 'get_dom', 'get_state',
      'navigate', 'click', 'input', 'scroll', 'send_keys', 'select_option',
      'screenshot', 'extract', 'switch_tab', 'close_tab', 'new_tab',
      'evaluate', 'toggle_annotations', 'search', 'go_back', 'go_forward',
      'reload', 'wait', 'done', 'save_to_file', 'read_file'
    ].includes(action)) {
      const knownActions = ['navigate', 'click', 'input', 'scroll', 'send_keys', 'select_option',
        'screenshot', 'extract', 'switch_tab', 'close_tab', 'new_tab',
        'evaluate', 'toggle_annotations', 'search', 'go_back', 'go_forward',
        'reload', 'wait', 'done', 'save_to_file', 'read_file']
      const cleaned = action.replace(/[\s\n\r]+/g, ' ').trim()
      for (const ka of knownActions) {
        if (cleaned.startsWith(ka)) {
          action = ka
          const tail = cleaned.slice(ka.length).trim()
          if (tail) {
            if (tail.startsWith('url') && input.url === undefined) {
              const urlMatch = tail.match(/^url\s*[:：]\s*(\S+)/)
              if (urlMatch) input.url = urlMatch[1]
            } else if (tail.startsWith('query') && input.query === undefined) {
              const queryMatch = tail.match(/^query\s*[:：]\s*(.+)/)
              if (queryMatch) input.query = queryMatch[1].trim()
            } else if (tail.startsWith('index') && input.index === undefined) {
              const indexMatch = tail.match(/^index\s*[:：]\s*(\d+)/)
              if (indexMatch) input.index = parseInt(indexMatch[1], 10)
            } else {
              input.url = tail
            }
          }
          break
        }
      }
    }

    if (!action) {
      return JSON.stringify({ success: false, error: '缺少 action 参数' })
    }

    if (action === 'start') {
      const showBrowser = input.show_browser as boolean | undefined
      const settings = loadSettings()
      const searchEngine = settings.defaultSearchEngine || 'bing'
      try {
        const result = await invoke('browser_start', { showBrowser: showBrowser !== false, defaultSearchEngine: searchEngine }) as Record<string, unknown>
        return JSON.stringify(result)
      } catch (e) {
        return JSON.stringify({ success: false, error: String(e) })
      }
    }

    if (action === 'stop') {
      try {
        const result = await invoke('browser_stop') as Record<string, unknown>
        return JSON.stringify(result)
      } catch (e) {
        return JSON.stringify({ success: false, error: String(e) })
      }
    }

    if (action === 'get_dom') {
      const showEmpty = input.show_empty as boolean | undefined
      try {
        const request: Record<string, unknown> = { type: 'get_dom' }
        if (showEmpty) request.show_empty = true
        const result = await invoke('browser_execute', { request }) as Record<string, unknown>
        return JSON.stringify(result)
      } catch (e) {
        return JSON.stringify({ success: false, error: String(e) })
      }
    }

    if (action === 'get_state') {
      try {
        const result = await invoke('browser_execute', { request: { type: 'get_state' } }) as Record<string, unknown>
        return JSON.stringify(result)
      } catch (e) {
        return JSON.stringify({ success: false, error: String(e) })
      }
    }

    const executeActions = [
      'navigate', 'click', 'input', 'scroll', 'send_keys', 'select_option',
      'screenshot', 'extract', 'switch_tab', 'close_tab', 'new_tab',
      'evaluate', 'toggle_annotations', 'search', 'go_back', 'go_forward',
      'reload', 'wait', 'done', 'save_to_file', 'read_file'
    ]

    if (executeActions.includes(action)) {
      const params: Record<string, unknown> = {}
      const paramKeys = ['url', 'index', 'text', 'query', 'keys', 'direction', 'amount', 'code', 'value', 'seconds', 'new_tab', 'clear', 'show', 'engine', 'file_name', 'content', 'success']
      for (const key of paramKeys) {
        if (input[key] !== undefined) {
          params[key] = input[key]
        }
      }

      try {
        const result = await invoke('browser_execute', {
          request: { type: 'execute', action, params }
        }) as Record<string, unknown>

        if (action === 'search' && result.type === 'ok') {
          const actionResult = result.result as Record<string, unknown> | undefined
          if (actionResult?.extracted_content) {
            const parts: string[] = [actionResult.extracted_content as string]
            const stateAfter = actionResult.state_after as Record<string, unknown> | undefined
            if (stateAfter?.url) {
              parts.push(`\n搜索页面: ${stateAfter.url}`)
            }
            return parts.join('\n')
          }
        }

        return JSON.stringify(result)
      } catch (e) {
        return JSON.stringify({ success: false, error: String(e) })
      }
    }

    return JSON.stringify({ success: false, error: `未知的浏览器操作: ${action}` })
  })

  fairyDo.registerVirtualHandler('memory_search', async (input) => {
    const customQuery = typeof input.query === 'string' && input.query.trim() ? input.query.trim() : null
    let query: string[]
    if (customQuery) {
      query = [customQuery]
    } else {
      query = recentUserMessages.length > 0 ? recentUserMessages : (currentUserMessage ? [currentUserMessage] : [])
    }
    if (query.length === 0) {
      return '错误: 未获取到用户消息且未提供查询词'
    }
    try {
      await fbmStore.ensureInit()
      const result = await fbmStore.retrieve(query, conversationContext || undefined)
      if (!result || !result.summary) {
        return '没有找到相关记忆'
      }
      return result.summary
    } catch (e) {
      console.warn('[MemorySearch] Error:', e)
      return `记忆搜索失败: ${e instanceof Error ? e.message : String(e)}`
    }
  })

  fairyDo.registerVirtualHandler('role_config', async (input) => {
    const action = input.action as string
    const fields = ['fairyName', 'userName', 'fairyPositioning', 'fairyStyle', 'fairySupplement', 'habitSupplement'] as const

    if (action === 'get') {
      const settings = loadSettings()
      const result: Record<string, string> = {}
      for (const field of fields) {
        result[field] = (settings as any)[field] || ''
      }
      return JSON.stringify(result, null, 2)
    }

    if (action === 'update') {
      const settings = loadSettings()
      const updated: string[] = []
      for (const field of fields) {
        const value = input[field] as string | undefined
        if (value !== undefined) {
          (settings as any)[field] = value
          updated.push(`${field}: "${value}"`)
        }
      }
      if (updated.length === 0) {
        return '错误: 未指定要修改的字段'
      }
      saveSettings(settings)
      try {
        await fbmStore.refreshCoreFiles()
      } catch (e) {
        console.warn('[RoleConfig] refreshCoreFiles failed:', e)
      }
      return `角色配置已更新: ${updated.join(', ')}`
    }

    return '错误: 未知的 action，支持 get 或 update'
  })

  fairyDo.registerVirtualHandler('fap_bridge', async (input) => {
    const action = input.action as string

    if (action === 'list') {
      try {
        const result = await invoke('fap_list') as Record<string, unknown>
        return JSON.stringify(result, null, 2)
      } catch (e) {
        return JSON.stringify({ success: false, error: String(e) })
      }
    }

    async function ensureBridgeAndSend(message: string): Promise<string> {
      try {
        const result = await invoke('fap_bridge_send', { message }) as Record<string, unknown>
        return JSON.stringify(result, null, 2)
      } catch (e) {
        const errStr = String(e)
        if (errStr.includes('触桥未启动')) {
          await invoke('fap_bridge_start')
          const retryResult = await invoke('fap_bridge_send', { message }) as Record<string, unknown>
          return JSON.stringify(retryResult, null, 2)
        }
        return JSON.stringify({ success: false, error: errStr })
      }
    }

    if (action === 'hello') {
      const module = input.module as string
      if (!module) {
        return JSON.stringify({ success: false, error: 'hello 操作需要 module 参数' })
      }
      const message = `bridge://hello\x1F${module}#`
      return await ensureBridgeAndSend(message)
    }

    if (action === 'call') {
      const module = input.module as string
      const channel = input.channel as string
      const fapAction = input.fap_action as string
      if (!module || !channel || !fapAction) {
        return JSON.stringify({ success: false, error: 'call 操作需要 module、channel、fap_action 参数' })
      }
      let paramsStr = '{}'
      if (input.params) {
        if (typeof input.params === 'string') {
          paramsStr = input.params
        } else {
          paramsStr = JSON.stringify(input.params)
        }
      }
      const message = `bridge://call\x1F${module}\x1F${channel}\x1F${fapAction}#${paramsStr}`
      return await ensureBridgeAndSend(message)
    }

    return JSON.stringify({ success: false, error: '未知的 action，支持 list、hello 或 call' })
  })
}
