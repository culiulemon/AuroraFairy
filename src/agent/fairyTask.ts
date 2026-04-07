import { invoke } from '@tauri-apps/api/core'
import { appDataDir } from '@tauri-apps/api/path'
import type { TaskItem, TaskFolderInfo, TaskStatus, ParsedTaskMd } from '../types/task'

const FAIRY_TASK_DIR = 'FairyTask'

async function getTaskBaseDir(): Promise<string> {
  const dataDir = await appDataDir()
  return `${dataDir}/${FAIRY_TASK_DIR}`
}

function generateFolderName(): string {
  const now = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  const rand = String(Math.floor(Math.random() * 1000)).padStart(3, '0')
  return `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}_${rand}`
}

function statusToCheckbox(status: TaskStatus): string {
  switch (status) {
    case 'completed': return 'x'
    case 'in_progress': return '~'
    default: return ' '
  }
}

function checkboxToStatus(checkbox: string): TaskStatus {
  if (checkbox === 'x') return 'completed'
  if (checkbox === '~') return 'in_progress'
  return 'pending'
}

export function buildTaskMdContent(taskName: string, tasks: TaskItem[]): string {
  const lines: string[] = []
  lines.push(`# 任务: ${taskName}`)
  lines.push('')
  lines.push('## 任务清单')
  lines.push('')
  for (const task of tasks) {
    lines.push(`- [${statusToCheckbox(task.status)}] ${task.id}. ${task.content} [${task.priority}]`)
  }
  lines.push('')
  lines.push('## 摘要')
  lines.push('')
  lines.push('')
  return lines.join('\n')
}

export function parseTaskMd(rawContent: string): ParsedTaskMd {
  const taskNameMatch = rawContent.match(/^#\s*任务:\s*(.+)$/m)
  const taskName = taskNameMatch ? taskNameMatch[1].trim() : '未命名任务'

  const tasks: TaskItem[] = []
  const taskRegex = /^-\s*\[([ x~])\]\s*(\d+)\.\s*(.+)\s+\[(high|medium|low)\]$/gm
  let match: RegExpExecArray | null
  while ((match = taskRegex.exec(rawContent)) !== null) {
    tasks.push({
      id: match[2],
      content: match[3].trim(),
      status: checkboxToStatus(match[1]),
      priority: match[4] as TaskItem['priority']
    })
  }

  const summaryLines: string[] = []
  const summaryRegex = /^>\s*(.+)$/gm
  let sMatch: RegExpExecArray | null
  while ((sMatch = summaryRegex.exec(rawContent)) !== null) {
    summaryLines.push(sMatch[1].trim())
  }
  const summary = summaryLines.length > 0 ? summaryLines.join('\n') : undefined

  return { taskName, tasks, summary, rawContent }
}

export class FairyTask {
  private taskFolders: Map<string, TaskFolderInfo> = new Map()

  private currentFolder: TaskFolderInfo | null = null

  getCurrentFolder(): TaskFolderInfo | null {
    return this.currentFolder
  }

  async createTask(taskName: string, todos: TaskItem[]): Promise<TaskFolderInfo> {
    const folderName = generateFolderName()
    const baseDir = await getTaskBaseDir()
    const folderPath = `${baseDir}/${folderName}`
    const taskMdPath = `${folderPath}/task.md`

    const content = buildTaskMdContent(taskName, todos)

    await invoke<string>('file_write', {
      path: taskMdPath,
      content
    })

    const info: TaskFolderInfo = {
      folderName,
      folderPath,
      taskMdPath,
      taskName,
      createdAt: new Date().toISOString()
    }

    this.taskFolders.set(folderName, info)
    this.currentFolder = info

    return info
  }

  async listTaskFolders(): Promise<string[]> {
    try {
      const baseDir = await getTaskBaseDir()
      const results = await invoke<string[]>('file_glob', {
        pattern: `${baseDir}/*/task.md`
      })
      return results.map(p => {
        const parts = p.replace(/\\/g, '/').split('/')
        return parts[parts.length - 2]
      })
    } catch {
      return []
    }
  }

  async recoverCurrentFolder(): Promise<TaskFolderInfo | null> {
    const folders = await this.listTaskFolders()
    if (folders.length === 0) return null

    folders.sort()
    const latestFolder = folders[folders.length - 1]
    const baseDir = await getTaskBaseDir()
    const folderPath = `${baseDir}/${latestFolder}`
    const taskMdPath = `${folderPath}/task.md`

    const rawContent = await invoke<string>('file_read', { path: taskMdPath, raw: true })
    const parsed = parseTaskMd(rawContent)

    const info: TaskFolderInfo = {
      folderName: latestFolder,
      folderPath,
      taskMdPath,
      taskName: parsed.taskName,
      createdAt: ''
    }

    this.taskFolders.set(latestFolder, info)
    this.currentFolder = info

    return info
  }

  async readTaskList(folderName?: string): Promise<ParsedTaskMd> {
    const folder = folderName
      ? this.taskFolders.get(folderName)
      : this.currentFolder

    if (!folder) {
      throw new Error('未找到任务文件夹')
    }

    const rawContent = await invoke<string>('file_read', { path: folder.taskMdPath, raw: true })
    return parseTaskMd(rawContent)
  }

  async updateTaskStatus(taskId: string, newStatus: TaskStatus): Promise<ParsedTaskMd> {
    if (!this.currentFolder) {
      await this.recoverCurrentFolder()
    }

    const folder = this.currentFolder
    if (!folder) {
      throw new Error('未找到任务文件夹，请先使用 todo_write 创建任务清单')
    }

    const current = await this.readTaskList()
    const task = current.tasks.find(t => t.id === taskId)
    if (!task) {
      throw new Error(`未找到任务 ID: ${taskId}`)
    }

    const oldCheckbox = statusToCheckbox(task.status)
    const newCheckbox = statusToCheckbox(newStatus)
    const oldLine = `- [${oldCheckbox}] ${taskId}. ${task.content} [${task.priority}]`
    const newLine = `- [${newCheckbox}] ${taskId}. ${task.content} [${task.priority}]`

    await invoke<string>('file_edit', {
      path: folder.taskMdPath,
      oldStr: oldLine,
      newStr: newLine
    })

    return this.readTaskList()
  }

  async appendSummary(summary: string): Promise<void> {
    if (!this.currentFolder) {
      await this.recoverCurrentFolder()
    }

    const folder = this.currentFolder
    if (!folder) {
      throw new Error('未找到任务文件夹，请先使用 todo_write 创建任务清单')
    }

    const rawContent = await invoke<string>('file_read', { path: folder.taskMdPath, raw: true })
    const summaryIndex = rawContent.indexOf('## 摘要')
    if (summaryIndex === -1) {
      throw new Error('task.md 格式错误：缺少 ## 摘要 标记')
    }

    const headerPart = rawContent.substring(0, summaryIndex + '## 摘要'.length)
    const existingBlock = rawContent.substring(summaryIndex + '## 摘要'.length)

    const trimmedBlock = existingBlock.trimEnd()
    const newBlock = trimmedBlock ? `${trimmedBlock}\n> ${summary}\n` : `\n> ${summary}\n`

    await invoke<string>('file_edit', {
      path: folder.taskMdPath,
      oldStr: rawContent,
      newStr: `${headerPart}${newBlock}`
    })
  }
}

export const fairyTask = new FairyTask()
