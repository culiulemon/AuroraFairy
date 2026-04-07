export type SecurityRuleCategory = 'shell' | 'file' | 'cdp'

export interface SecurityRule {
  id: string
  pattern: string
  category: SecurityRuleCategory
  description: string
  enabled: boolean
  isBuiltIn: boolean
  createdAt: string
  updatedAt: string
}
