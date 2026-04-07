import type { ExecutorName } from '../agent/fairyDo'

export interface ToolParameter {
  name: string
  type: 'string' | 'number' | 'integer' | 'boolean' | 'array' | 'object'
  description: string
  required: boolean
  default?: unknown
}

export interface Tool {
  id: string
  name: string
  description: string
  logo: string
  logoType?: string
  invokeName: string
  filePath: string
  parameters: ToolParameter[]
  executor: ExecutorName
  code?: string
  language: 'typescript'
  createdAt: string
  updatedAt: string
}

export interface ToolMetadata {
  name: string
  description: string
  logo: string
  logoType?: string
  invokeName: string
}
