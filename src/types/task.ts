export type TaskStatus = 'pending' | 'in_progress' | 'completed'
export type TaskPriority = 'high' | 'medium' | 'low'

export interface TaskItem {
  id: string
  content: string
  status: TaskStatus
  priority: TaskPriority
}

export interface TaskFolderInfo {
  folderName: string
  folderPath: string
  taskMdPath: string
  taskName: string
  createdAt: string
}

export interface ParsedTaskMd {
  taskName: string
  tasks: TaskItem[]
  summary?: string
  rawContent: string
}
