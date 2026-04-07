import { BUILTIN_SECURITY_RULES } from '../agent/fairyDo'
import type { SecurityRule, SecurityRuleCategory } from '../types/security'

interface SecurityRulesConfig {
  version: number
  builtinOverrides: Record<string, { enabled: boolean }>
  customRules: SecurityRule[]
}

let rules: SecurityRule[] = []
let loaded = false

function mergeRules(config: SecurityRulesConfig | null): SecurityRule[] {
  const merged = BUILTIN_SECURITY_RULES.map(rule => {
    const override = config?.builtinOverrides[rule.id]
    return override !== undefined
      ? { ...rule, enabled: override.enabled }
      : { ...rule }
  })
  const customRules = config?.customRules || []
  return [...merged, ...customRules]
}

export async function loadRules(): Promise<SecurityRule[]> {
  if (loaded) return [...rules]
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const content = await invoke<string>('load_security_rules')
    if (content) {
      const config = JSON.parse(content) as SecurityRulesConfig
      rules = mergeRules(config)
    } else {
      rules = mergeRules(null)
    }
  } catch {
    rules = mergeRules(null)
  }
  loaded = true
  return rules
}

export async function saveRules(): Promise<void> {
  const builtinOverrides: Record<string, { enabled: boolean }> = {}
  const customRules: SecurityRule[] = []
  for (const rule of rules) {
    if (rule.isBuiltIn) {
      const builtin = BUILTIN_SECURITY_RULES.find(b => b.id === rule.id)
      if (builtin && builtin.enabled !== rule.enabled) {
        builtinOverrides[rule.id] = { enabled: rule.enabled }
      }
    } else {
      customRules.push(rule)
    }
  }
  const config: SecurityRulesConfig = {
    version: 1,
    builtinOverrides,
    customRules,
  }
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('save_security_rules', { content: JSON.stringify(config, null, 2) })
  } catch (e) {
    console.error('[securityStore] 保存失败:', e)
  }
}

export function getRules(): SecurityRule[] {
  return rules
}

export function getRulesByCategory(category: SecurityRuleCategory): SecurityRule[] {
  return rules.filter(r => r.category === category)
}

export function getCustomRules(): SecurityRule[] {
  return rules.filter(r => !r.isBuiltIn)
}

export async function addRule(rule: Omit<SecurityRule, 'id' | 'isBuiltIn' | 'createdAt' | 'updatedAt'>): Promise<SecurityRule> {
  const newRule: SecurityRule = {
    ...rule,
    id: `custom-${Date.now()}`,
    isBuiltIn: false,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  rules.push(newRule)
  await saveRules()
  return newRule
}

export async function updateRule(id: string, updates: Partial<Pick<SecurityRule, 'pattern' | 'description' | 'category' | 'enabled'>>): Promise<SecurityRule | null> {
  const index = rules.findIndex(r => r.id === id)
  if (index === -1) return null
  rules[index] = { ...rules[index], ...updates, updatedAt: new Date().toISOString() }
  await saveRules()
  return rules[index]
}

export async function deleteRule(id: string): Promise<boolean> {
  const index = rules.findIndex(r => r.id === id)
  if (index === -1 || rules[index].isBuiltIn) return false
  rules.splice(index, 1)
  await saveRules()
  return true
}

export async function toggleRule(id: string): Promise<SecurityRule | null> {
  const index = rules.findIndex(r => r.id === id)
  if (index === -1) return null
  rules[index] = { ...rules[index], enabled: !rules[index].enabled, updatedAt: new Date().toISOString() }
  await saveRules()
  return rules[index]
}

export async function resetToDefaults(): Promise<void> {
  rules = mergeRules(null)
  loaded = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('delete_security_rules')
  } catch (e) {
    console.error('[securityStore] 删除配置文件失败:', e)
  }
}
