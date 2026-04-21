import { loadSettings } from './settings'
import type { Tool } from '../types/tool'
import type { ToolUseBlock } from '../types/message'
import { addDebugLog } from './debugStore'
import { invoke } from '@tauri-apps/api/core'

function repairTruncatedJson(raw: string): Record<string, unknown> | null {
  const trimmed = raw.trim()
  if (trimmed.startsWith('{') && !trimmed.endsWith('}')) {
    let repaired = trimmed
    if (!repaired.endsWith('"') && !repaired.endsWith(',') && !repaired.endsWith(':') && !repaired.endsWith(' ')) {
      repaired += '"'
    }
    if (repaired.endsWith('"')) {
      const lastColon = repaired.lastIndexOf(':')
      if (lastColon !== -1) {
        const afterColon = repaired.slice(lastColon + 1).trim()
        if (afterColon.startsWith('"') && !afterColon.endsWith('"')) {
          repaired = repaired + '}'
        } else if (afterColon && !afterColon.startsWith('"') && !afterColon.startsWith('{') && !afterColon.startsWith('[')) {
          repaired = repaired + '"}'
        } else if (afterColon === '') {
          repaired = repaired + '"}'
        } else {
          repaired = repaired + '}'
        }
      } else {
        repaired = repaired + '}'
      }
    } else {
      repaired = repaired + '}'
    }
    try {
      return JSON.parse(repaired) as Record<string, unknown>
    } catch {
    }
  }
  if (trimmed.endsWith(',')) {
    try {
      return JSON.parse(trimmed.slice(0, -1) + '}') as Record<string, unknown>
    } catch {
    }
  }
  return null
}

function safeParseToolArgs(rawArgs: string): Record<string, unknown> {
  try {
    return JSON.parse(rawArgs) as Record<string, unknown>
  } catch {
    const repaired = repairTruncatedJson(rawArgs)
    if (repaired) {
      console.warn('[Chat] 工具参数 JSON 截断，已自动修复:', rawArgs, '->', repaired)
      return repaired
    }
    console.error('[Chat] 工具参数最终解析失败:', rawArgs)
    return { _raw: rawArgs }
  }
}

export async function proxyChatRequest(
  url: string,
  headers: Record<string, string>,
  body: Record<string, unknown>
): Promise<{ status: number; data: any }> {
  const proxyHeaders: Record<string, string> = {}
  for (const [key, value] of Object.entries(headers)) {
    proxyHeaders[key] = value
  }

  const result = await invoke<{ status: number; headers: Record<string, string>; body: string }>('proxy_chat', {
    request: {
      url,
      headers: proxyHeaders,
      body
    }
  })

  return {
    status: result.status,
    data: JSON.parse(result.body)
  }
}

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string
  tool_call_id?: string
  name?: string
}

export type ToolChoice = 'auto' | 'none' | 'required' | string

export interface ChatRequest {
  providerId: string
  messages: ChatMessage[]
  model?: string
  tools?: Tool[]
  tool_choice?: ToolChoice
  onChunk?: (chunk: string) => void
  onReasoningChunk?: (chunk: string) => void
  onToolCall?: (toolCall: ToolUseBlock) => void
  signal?: AbortSignal
}

export interface ChatResponse {
  content: string
  usage?: {
    prompt_tokens: number
    completion_tokens: number
    total_tokens: number
  }
  toolCalls?: ToolUseBlock[]
}

export async function generateConversationTitle(
  providerId: string,
  userMessage: string
): Promise<string> {
  const settings = loadSettings()
  const provider = settings.providers.find(p => p.id === providerId)
    || settings.providers.find(p => p.id === settings.defaultProviderId)

  if (!provider) {
    return '新会话'
  }

  const model = provider.model || 'gpt-3.5-turbo'

  const headers: Record<string, string> = {
    'Content-Type': 'application/json'
  }

  if (provider.protocol === 'openai' || provider.protocol === 'custom') {
    headers['Authorization'] = `Bearer ${provider.apiKey}`
  } else if (provider.protocol === 'anthropic') {
    headers['x-api-key'] = provider.apiKey
    headers['anthropic-version'] = '2023-06-01'
  }

  const systemPrompt = '你是一个对话标题生成器。根据用户发送的第一条消息，生成一个简短的对话标题。要求：1) 不超过15个字；2) 概括用户意图；3) 不要使用引号；4) 直接输出标题文本，不要任何解释。'

  let body: Record<string, unknown>

  if (provider.protocol === 'anthropic') {
    body = {
      model,
      max_tokens: 64,
      stream: false,
      system: systemPrompt,
      messages: [
        { role: 'user', content: userMessage }
      ]
    }
  } else {
    body = {
      model,
      messages: [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: userMessage }
      ],
      temperature: 0.5,
      stream: false
    }
  }

  const url = provider.baseUrl.endsWith('/chat/completions')
    ? provider.baseUrl
    : `${provider.baseUrl}/chat/completions`

  const useProxy = settings.useBackendProxy === true

  if (useProxy) {
    try {
      const result = await proxyChatRequest(url, headers, body)

      if (!result.status.toString().startsWith('2')) {
        console.warn('[Chat] 生成标题失败:', result.status)
        return '新会话'
      }

      if (provider.protocol === 'anthropic') {
        return result.data.content?.[0]?.text?.trim() || '新会话'
      }

      const title = result.data.choices?.[0]?.message?.content?.trim()
      return title || '新会话'
    } catch (error) {
      console.warn('[Chat] 生成标题请求失败:', error)
      return '新会话'
    }
  }

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers,
      body: JSON.stringify(body)
    })

    if (!response.ok) {
      console.warn('[Chat] 生成标题失败:', response.status)
      return '新会话'
    }

    const data = await response.json()

    if (provider.protocol === 'anthropic') {
      return data.content?.[0]?.text?.trim() || '新会话'
    }

    const title = data.choices?.[0]?.message?.content?.trim()
    return title || '新会话'
  } catch (error) {
    console.warn('[Chat] 生成标题请求失败:', error)
    return '新会话'
  }
}

export async function sendChatMessage(request: ChatRequest): Promise<ChatResponse> {
  const settings = loadSettings()
  const provider = settings.providers.find(p => p.id === request.providerId) 
    || settings.providers.find(p => p.id === settings.defaultProviderId)

  if (!provider) {
    throw new Error('没有配置 API 提供商')
  }

  const model = request.model || provider.model || 'gpt-3.5-turbo'
  
  const headers: Record<string, string> = {
    'Content-Type': 'application/json'
  }

  if (provider.protocol === 'openai' || provider.protocol === 'custom') {
    headers['Authorization'] = `Bearer ${provider.apiKey}`
  } else if (provider.protocol === 'anthropic') {
    headers['x-api-key'] = provider.apiKey
    headers['anthropic-version'] = '2023-06-01'
  }

  let body: Record<string, unknown>

  if (provider.protocol === 'anthropic') {
    body = {
      model,
      max_tokens: 4096,
      stream: !!request.onChunk,
      messages: request.messages.filter(m => m.role !== 'system').map(m => ({
        role: m.role === 'assistant' ? 'assistant' : 'user',
        content: m.content
      }))
    }
  } else {
    body = {
      model,
      messages: request.messages,
      temperature: 0.7,
      stream: !!request.onChunk
    }

    if (provider.protocol === 'openai' || provider.protocol === 'custom') {
      body.thinking = { type: provider.thinkingEnabled ? 'enabled' : 'disabled' }
    }

    if (request.tools && request.tools.length > 0 && provider.supportsTools !== false) {
      body.tools = request.tools.map(tool => ({
        type: "function",
        function: {
          name: tool.invokeName,
          description: tool.description,
          parameters: {
            type: "object",
            properties: Object.fromEntries(
              tool.parameters.map(p => [p.name, {
                type: p.type,
                description: p.description
              }])
            ),
            required: tool.parameters.filter(p => p.required).map(p => p.name)
          }
        }
      }))
      body.tool_choice = request.tool_choice || "auto"
    }
  }

  const url = provider.baseUrl.endsWith('/chat/completions')
    ? provider.baseUrl
    : `${provider.baseUrl}/chat/completions`

  addDebugLog('request', `POST ${url}`, JSON.stringify(body, null, 2), {
    provider: provider.displayName || provider.id,
    model,
    protocol: provider.protocol,
    messageCount: request.messages.length,
    hasTools: !!request.tools?.length
  })

  const useProxy = settings.useBackendProxy === true

  if (request.onChunk) {
    if (useProxy) {
      return await proxyStreamChat(
        url,
        headers,
        body,
        provider.protocol,
        request.onChunk,
        request.onReasoningChunk,
        request.onToolCall
      )
    }
    return await streamChat(
      url,
      headers,
      body,
      provider.protocol,
      request.onChunk,
      request.onReasoningChunk,
      request.onToolCall,
      request.signal
    )
  }

  if (useProxy) {
    try {
      const result = await proxyChatRequest(url, headers, body)

      addDebugLog('response', `Proxy Response ${result.status}`, JSON.stringify(result.data, null, 2), {
        provider: provider.displayName || provider.id,
        status: result.status
      })

      if (!result.status.toString().startsWith('2')) {
        throw new Error(`API 请求失败: ${result.status} - ${JSON.stringify(result.data)}`)
      }

      if (provider.protocol === 'anthropic') {
        return {
          content: result.data.content?.[0]?.text || '',
          usage: result.data.usage
        }
      }

      const toolCalls = extractToolCalls(result.data.choices?.[0]?.message?.tool_calls)
      return {
        content: result.data.choices?.[0]?.message?.content || '',
        usage: result.data.usage,
        toolCalls
      }
    } catch (error) {
      addDebugLog('error', 'Proxy Call Failed', String(error), { url })
      console.error('代理 API 调用失败:', error)
      throw error
    }
  }

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers,
      body: JSON.stringify(body)
    })

    if (!response.ok) {
      const error = await response.text()
      addDebugLog('error', `API Error ${response.status}`, error, {
        url,
        status: response.status,
        provider: provider.displayName || provider.id
      })
      throw new Error(`API 请求失败: ${response.status} - ${error}`)
    }

    const data = await response.json()

    addDebugLog('response', `Response ${response.status}`, JSON.stringify(data, null, 2), {
      provider: provider.displayName || provider.id,
      status: response.status,
      contentType: response.headers.get('content-type')
    })

    if (provider.protocol === 'anthropic') {
      return {
        content: data.content[0]?.text || '',
        usage: data.usage
      }
    }

    const toolCalls = extractToolCalls(data.choices?.[0]?.message?.tool_calls)
    return {
      content: data.choices[0]?.message?.content || '',
      usage: data.usage,
      toolCalls
    }
  } catch (error) {
    addDebugLog('error', 'API Call Failed', String(error), { url })
    console.error('API 调用失败:', error)
    throw error
  }
}

function extractToolCalls(toolCalls: any[] | undefined): ToolUseBlock[] {
  if (!toolCalls || !Array.isArray(toolCalls)) {
    return []
  }
  
  return toolCalls.map(tc => ({
    type: 'tool_use' as const,
    id: tc.id,
    name: tc.function?.name,
    input: typeof tc.function?.arguments === 'string' 
      ? safeParseToolArgs(tc.function.arguments) 
      : tc.function?.arguments || {}
  }))
}

async function streamChat(
  url: string,
  headers: Record<string, string>,
  body: Record<string, unknown>,
  protocol: string,
  onChunk: (chunk: string) => void,
  onReasoningChunk: ((chunk: string) => void) | undefined,
  onToolCall: ((toolCall: ToolUseBlock) => void) | undefined,
  signal?: AbortSignal
): Promise<ChatResponse> {
  const response = await fetch(url, {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
    signal
  })

  if (!response.ok) {
    const error = await response.text()
    addDebugLog('error', `Stream Error ${response.status}`, error, {
      url,
      status: response.status
    })
    throw new Error(`API 请求失败: ${response.status} - ${error}`)
  }

  const reader = response.body?.getReader()
  if (!reader) {
    throw new Error('无法获取响应流')
  }

  const decoder = new TextDecoder()
  let fullContent = ''
  let usage: ChatResponse['usage']
  const collectedToolCalls: ToolUseBlock[] = []
  const toolCallArgsBuffers: Map<number, string> = new Map()
  let currentToolCallIndex = -1
  let lastFinishReason = ''

  try {
    while (true) {
      const { done, value } = await reader.read()
      if (done) break

      const chunk = decoder.decode(value, { stream: true })
      const lines = chunk.split('\n')

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = line.slice(6)
          if (data === '[DONE]') continue

          try {
            const parsed = JSON.parse(data)
            
            if (protocol === 'anthropic') {
              if (parsed.type === 'content_block_delta') {
                const text = parsed.delta?.text || ''
                fullContent += text
                onChunk(text)
              }
            } else {
              const reasoningContent = parsed.choices?.[0]?.delta?.reasoning_content || ''
              const content = parsed.choices?.[0]?.delta?.content || ''
              console.log('[Chat] 收到 reasoning_content:', JSON.stringify(reasoningContent), 'length:', reasoningContent?.length)
              if (reasoningContent && onReasoningChunk) {
                console.log('[Chat] 调用 onReasoningChunk')
                onReasoningChunk(reasoningContent)
              }
              if (content) {
                fullContent += content
                onChunk(content)
              }
              if (parsed.usage) {
                usage = parsed.usage
              }

              if (parsed.choices?.[0]?.finish_reason) {
                lastFinishReason = parsed.choices[0].finish_reason
              }

              const deltaToolCalls = parsed.choices?.[0]?.delta?.tool_calls
              if (deltaToolCalls && Array.isArray(deltaToolCalls)) {
                for (const delta of deltaToolCalls) {
                  if (delta.index !== undefined) {
                    if (delta.index !== currentToolCallIndex) {
                      currentToolCallIndex = delta.index
                      collectedToolCalls.push({
                        type: 'tool_use',
                        id: delta.id || '',
                        name: '',
                        input: {}
                      })
                    }
                  }
                  
                  if (delta.function?.name) {
                    const lastIndex = collectedToolCalls.length - 1
                    if (lastIndex >= 0) {
                      collectedToolCalls[lastIndex].name = delta.function.name
                    }
                  }
                  
                  if (delta.function?.arguments) {
                    const lastIndex = collectedToolCalls.length - 1
                    if (lastIndex >= 0) {
                      const buf = toolCallArgsBuffers.get(currentToolCallIndex) || ''
                      toolCallArgsBuffers.set(currentToolCallIndex, buf + delta.function.arguments)
                    }
                  }

                  if (delta.index !== undefined && delta.function?.name) {
                    const toolCall = collectedToolCalls.find(tc => tc.name === delta.function?.name)
                    if (toolCall && onToolCall) {
                      onToolCall(toolCall)
                    }
                  }
                }
              }
            }
          } catch {
            // 忽略解析错误
          }
        }
      }
    }
  } finally {
    reader.releaseLock()
  }

  addDebugLog('response', `Stream Complete`, fullContent + (collectedToolCalls.length > 0
    ? '\n\n--- Tool Calls ---\n' + collectedToolCalls.map(tc =>
        `[${tc.name}]\n${JSON.stringify(tc.input, null, 2)}`
      ).join('\n\n')
    : ''
  ), {
    usage,
    toolCallCount: collectedToolCalls.length,
    contentLength: fullContent.length
  })

  console.log(`[Chat] Stream ended. finish_reason: ${lastFinishReason}, content length: ${fullContent.length}, toolCalls: ${collectedToolCalls.length}`)

  for (let i = 0; i < collectedToolCalls.length; i++) {
    const rawArgs = toolCallArgsBuffers.get(i) || ''
    if (rawArgs) {
      collectedToolCalls[i].input = safeParseToolArgs(rawArgs)
    }
  }

  return { content: fullContent, usage, toolCalls: collectedToolCalls }
}

async function proxyStreamChat(
  url: string,
  headers: Record<string, string>,
  body: Record<string, unknown>,
  protocol: string,
  onChunk: (chunk: string) => void,
  onReasoningChunk: ((chunk: string) => void) | undefined,
  onToolCall: ((toolCall: ToolUseBlock) => void) | undefined
): Promise<ChatResponse> {
  const { listen } = await import('@tauri-apps/api/event')

  let fullContent = ''
  let usage: ChatResponse['usage']
  const collectedToolCalls: ToolUseBlock[] = []
  const toolCallArgsBuffers: Map<number, string> = new Map()
  let currentToolCallIndex = -1
  let done = false

  const chunkListener = listen<number[]>('proxy-chat-chunk', (event) => {
    const bytes = new Uint8Array(event.payload)
    const chunk = new TextDecoder().decode(bytes, { stream: true })
    const lines = chunk.split('\n')

    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = line.slice(6)
        if (data === '[DONE]') {
          done = true
          continue
        }

        try {
          const parsed = JSON.parse(data)

          if (protocol === 'anthropic') {
            if (parsed.type === 'content_block_delta') {
              const text = parsed.delta?.text || ''
              fullContent += text
              onChunk(text)
            }
          } else {
            const reasoningContent = parsed.choices?.[0]?.delta?.reasoning_content || ''
            const content = parsed.choices?.[0]?.delta?.content || ''

            if (reasoningContent && onReasoningChunk) {
              onReasoningChunk(reasoningContent)
            }
            if (content) {
              fullContent += content
              onChunk(content)
            }
            if (parsed.usage) {
              usage = parsed.usage
            }

            if (parsed.choices?.[0]?.finish_reason) {
            }

            const deltaToolCalls = parsed.choices?.[0]?.delta?.tool_calls
            if (deltaToolCalls && Array.isArray(deltaToolCalls)) {
              for (const delta of deltaToolCalls) {
                if (delta.index !== undefined) {
                  if (delta.index !== currentToolCallIndex) {
                    currentToolCallIndex = delta.index
                    collectedToolCalls.push({
                      type: 'tool_use',
                      id: delta.id || '',
                      name: '',
                      input: {}
                    })
                  }
                }

                if (delta.function?.name) {
                  const lastIndex = collectedToolCalls.length - 1
                  if (lastIndex >= 0) {
                    collectedToolCalls[lastIndex].name = delta.function.name
                  }
                }

                if (delta.function?.arguments) {
                  const lastIndex = collectedToolCalls.length - 1
                  if (lastIndex >= 0) {
                    const buf = toolCallArgsBuffers.get(currentToolCallIndex) || ''
                    toolCallArgsBuffers.set(currentToolCallIndex, buf + delta.function.arguments)
                  }
                }

                if (delta.index !== undefined && delta.function?.name) {
                  const toolCall = collectedToolCalls.find(tc => tc.name === delta.function?.name)
                  if (toolCall && onToolCall) {
                    onToolCall(toolCall)
                  }
                }
              }
            }
          }
        } catch {
          // 忽略解析错误
        }
      }
    }
  })

  const errorListener = listen<string>('proxy-chat-error', (event) => {
    console.error('[Chat] Proxy stream error:', event.payload)
  })

  const doneListener = listen('proxy-chat-done', () => {
    done = true
  })

  try {
    await invoke('proxy_chat_stream', {
      request: {
        url,
        headers,
        body
      }
    })
  } catch (error) {
    console.error('[Chat] Proxy stream invoke failed:', error)
    addDebugLog('error', 'Proxy Stream Invoke Failed', String(error), { url })
    throw error
  }

  await new Promise<void>((resolve) => {
    const checkDone = setInterval(() => {
      if (done) {
        clearInterval(checkDone)
        resolve()
      }
    }, 50)
  })

  ;(await chunkListener)()
  ;(await errorListener)()
  ;(await doneListener)()

  addDebugLog('response', `Proxy Stream Complete`, fullContent + (collectedToolCalls.length > 0
    ? '\n\n--- Tool Calls ---\n' + collectedToolCalls.map(tc =>
        `[${tc.name}]\n${JSON.stringify(tc.input, null, 2)}`
      ).join('\n\n')
    : ''
  ), {
    usage,
    toolCallCount: collectedToolCalls.length,
    contentLength: fullContent.length
  })

  for (let i = 0; i < collectedToolCalls.length; i++) {
    const rawArgs = toolCallArgsBuffers.get(i) || ''
    if (rawArgs) {
      collectedToolCalls[i].input = safeParseToolArgs(rawArgs)
    }
  }

  return { content: fullContent, usage, toolCalls: collectedToolCalls }
}
