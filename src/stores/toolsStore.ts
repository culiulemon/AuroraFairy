import { getDB } from './db'
import type { Tool, ToolMetadata } from '../types/tool'
import type { ExecutorName } from '../agent/fairyDo'

export interface ParsedTool {
  metadata: ToolMetadata
  code: string
}

function extractStringValue(objStr: string, key: string): string | null {
  const patterns = [
    new RegExp(`${key}:\\s*['"]([^'"]*)['"]`),
    new RegExp(`${key}:\\s*["']([^"']*)["']`),
  ]
  for (const pattern of patterns) {
    const match = objStr.match(pattern)
    if (match) return match[1]
  }
  return null
}

function extractDefineToolBlocks(content: string): string[] {
  const blocks: string[] = []
  const regex = /defineTool\s*\(\s*\{([\s\S]*?)\}\s*\)/g
  let match
  
  while ((match = regex.exec(content)) !== null) {
    blocks.push(match[0])
  }
  
  const exportDefaultArray = /export\s+default\s*\[([\s\S]*?)\]/g
  while ((match = exportDefaultArray.exec(content)) !== null) {
    const arrayContent = match[1]
    const toolRefs = arrayContent.match(/[\w]+/g) || []
    for (const ref of toolRefs) {
      const exportPattern = new RegExp(`export\\s+const\\s+${ref}\\s*=\\s*defineTool\\s*\\(`)
      const exportMatch = content.match(exportPattern)
      if (exportMatch) {
        const fullBlockMatch = content.substring(content.indexOf(exportMatch[0])).match(/defineTool\s*\(\s*\{([\s\S]*?)\}\s*\)/)
        if (fullBlockMatch) {
          blocks.push('defineTool({' + fullBlockMatch[1] + '})')
        }
      }
    }
  }
  
  return blocks
}

export function parseToolMetadata(content: string, _filePath: string): ToolMetadata | null {
  const frontmatterRegex = /^---\n([\s\S]*?)\n---/
  const match = content.match(frontmatterRegex)
  
  if (match) {
    const frontmatter = match[1]
    const result: ToolMetadata = {
      name: '',
      description: '',
      logo: '',
      logoType: 'image/svg+xml',
      invokeName: ''
    }
    
    frontmatter.split('\n').forEach(line => {
      const [key, ...valueParts] = line.split(':')
      if (key && valueParts.length > 0) {
        const value = valueParts.join(':').trim()
        if (key.trim() === 'name') result.name = value
        if (key.trim() === 'description') result.description = value
        if (key.trim() === 'logo') result.logo = value
        if (key.trim() === 'logoType') result.logoType = value
        if (key.trim() === 'invokeName') result.invokeName = value
      }
    })
    
    if (!result.name || !result.invokeName) {
      return null
    }
    
    return result
  }
  
  const blocks = extractDefineToolBlocks(content)
  if (blocks.length === 0) {
    return null
  }
  
  const firstBlock = blocks[0]
  const name = extractStringValue(firstBlock, 'name')
  const description = extractStringValue(firstBlock, 'description')
  const executor = extractStringValue(firstBlock, 'executor')
  
  if (!name) {
    return null
  }
  
  return {
    name,
    description: description || '',
    logo: '',
    logoType: 'image/svg+xml',
    invokeName: executor || name
  }
}

export function parseToolContent(content: string): { code: string; language: 'typescript' } | null {
  const frontmatterRegex = /^---\n([\s\S]*?)\n---\n*/
  const match = content.match(frontmatterRegex)
  
  if (match) {
    const codeContent = content.slice(match[0].length).trim()
    if (!codeContent) return null
    return { code: codeContent, language: 'typescript' }
  }
  
  return { code: content.trim(), language: 'typescript' }
}

export function parseAllTools(content: string): ParsedTool[] {
  const results: ParsedTool[] = []
  const frontmatterRegex = /^---\n([\s\S]*?)\n---/
  const frontmatterMatch = content.match(frontmatterRegex)
  
  if (frontmatterMatch) {
    const code = content.slice(frontmatterMatch[0].length).trim()
    const metadata = parseToolMetadata(content, '')
    if (metadata) {
      results.push({ metadata, code })
    }
    return results
  }
  
  const blocks = extractDefineToolBlocks(content)
  for (const block of blocks) {
    const name = extractStringValue(block, 'name')
    const description = extractStringValue(block, 'description')
    const executor = extractStringValue(block, 'executor')
    
    if (name) {
      results.push({
        metadata: {
          name,
          description: description || '',
          logo: '',
          logoType: 'image/svg+xml',
          invokeName: executor || name
        },
        code: content.trim()
      })
    }
  }
  
  return results
}

export function createToolFromMetadata(
  metadata: ToolMetadata, 
  filePath: string, 
  code?: string, 
  _language?: 'typescript',
  executor?: ExecutorName
): Tool {
  const now = new Date().toISOString()
  return {
    id: `tool-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    name: metadata.name,
    description: metadata.description,
    logo: metadata.logo,
    logoType: metadata.logoType,
    invokeName: metadata.invokeName,
    filePath,
    parameters: [],
    executor: executor || 'shell_execute',
    code,
    language: 'typescript',
    createdAt: now,
    updatedAt: now
  }
}

export function loadAllTools(): Promise<Tool[]> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['tools'], 'readonly')
    const store = transaction.objectStore('tools')
    const index = store.index('updatedAt')
    
    console.log('[ToolsStore] 加载所有工具...')
    const request = index.openCursor(null, 'prev')
    const tools: Tool[] = []
    
    request.onsuccess = (event) => {
      const cursor = (event.target as IDBRequest<IDBCursorWithValue>).result
      if (cursor) {
        tools.push(cursor.value)
        cursor.continue()
      } else {
        console.log('[ToolsStore] 加载完成，工具数:', tools.length)
        resolve(tools)
      }
    }
    
    request.onerror = () => {
      console.error('[ToolsStore] 加载工具列表失败:', request.error)
      reject(new Error('加载工具列表失败'))
    }
  })
}

export function loadToolByFilePath(filePath: string): Promise<Tool | undefined> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['tools'], 'readonly')
    const store = transaction.objectStore('tools')
    const index = store.index('filePath')
    
    const request = index.get(filePath)
    
    request.onsuccess = () => {
      resolve(request.result)
    }
    
    request.onerror = () => {
      reject(new Error('加载工具失败'))
    }
  })
}

export function saveTool(tool: Tool): Promise<void> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['tools'], 'readwrite')
    const store = transaction.objectStore('tools')
    
    console.log('[ToolsStore] 保存工具:', tool.id, tool.name)
    
    const plainTool = JSON.parse(JSON.stringify(tool))
    const request = store.put(plainTool)
    
    request.onsuccess = () => {
      console.log('[ToolsStore] 工具保存成功:', tool.id)
      resolve()
    }
    request.onerror = () => {
      console.error('[ToolsStore] 工具保存失败:', tool.id)
      reject(new Error('保存工具失败'))
    }
  })
}

export function deleteTool(id: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['tools'], 'readwrite')
    const store = transaction.objectStore('tools')
    
    const request = store.delete(id)
    
    request.onsuccess = () => {
      console.log('[ToolsStore] 工具删除成功:', id)
      resolve()
    }
    request.onerror = () => {
      console.error('[ToolsStore] 工具删除失败:', id)
      reject(new Error('删除工具失败'))
    }
  })
}

export function clearAllTools(): Promise<void> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['tools'], 'readwrite')
    const store = transaction.objectStore('tools')
    
    const request = store.clear()
    
    request.onsuccess = () => {
      console.log('[ToolsStore] 清空所有工具成功')
      resolve()
    }
    request.onerror = () => {
      console.error('[ToolsStore] 清空所有工具失败:', request.error)
      reject(new Error('清空工具失败'))
    }
  })
}

export function fileToBase64(file: File): Promise<{ data: string; type: string }> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const result = reader.result as string
      const base64 = result.split(',')[1]
      resolve({
        data: base64,
        type: file.type
      })
    }
    reader.onerror = reject
    reader.readAsDataURL(file)
  })
}

export function cleanBuiltinToolsFromDb(): Promise<void> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['tools'], 'readwrite')
    const store = transaction.objectStore('tools')
    const allKeysRequest = store.getAllKeys()

    allKeysRequest.onsuccess = () => {
      const keys = allKeysRequest.result
      const toDelete = (keys as IDBValidKey[]).filter(
        (key) => typeof key === 'string' && (key.startsWith('builtin-') || key.startsWith('sys-'))
      )
      if (toDelete.length === 0) {
        resolve()
        return
      }
      let deleted = 0
      for (const key of toDelete) {
        const req = store.delete(key)
        req.onsuccess = () => {
          deleted++
          if (deleted === toDelete.length) resolve()
        }
        req.onerror = () => reject(new Error(`Failed to delete key ${key}`))
      }
    }
    allKeysRequest.onerror = () => reject(new Error('Failed to get all keys'))
  })
}

