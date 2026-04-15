import { invoke } from '@tauri-apps/api/core'
import type { LLMAdapter } from '../../fbm/src/types/adapter.js'
import type {
  SkillEntry,
  SkillFrontmatter,
  SkillGatingStatus,
  SkillIndex,
  SkillIndexEntry,
  SkillRequires,
} from './types.js'
import { loadAllSkills, resolveSkillDirs } from './skillLoader.js'
import { selectSkills } from './skillSelector.js'

let skillEntries: Map<string, SkillEntry> = new Map()
let skillIndex: SkillIndex | null = null
let initialized = false

function getCurrentOs(): 'windows' | 'macos' | 'linux' {
  const platform = navigator.platform.toLowerCase()
  if (platform.includes('win')) return 'windows'
  if (platform.includes('mac')) return 'macos'
  return 'linux'
}

async function checkBinExists(bin: string): Promise<boolean> {
  const os = getCurrentOs()
  const cmd = os === 'windows' ? `where ${bin} 2>nul` : `which ${bin} 2>/dev/null`
  try {
    await invoke<string>('shell_execute', { command: cmd, timeout: 5 })
    return true
  } catch {
    return false
  }
}

function extractRequires(fm: SkillFrontmatter): SkillRequires | undefined {
  if (fm.requires) return fm.requires
  const openclaw = fm.metadata?.openclaw
  if (!openclaw) return undefined
  const req = openclaw.requires
  if (!req) return undefined
  return {
    env: req.env,
    optionalEnv: req.optionalEnv,
    bins: req.bins,
  }
}

function extractEmoji(fm: SkillFrontmatter): string {
  if (fm.emoji) return fm.emoji
  return fm.metadata?.openclaw?.emoji || '⚡'
}

function extractTags(fm: SkillFrontmatter): string[] {
  if (fm.tags && fm.tags.length > 0) return fm.tags
  return fm.metadata?.openclaw?.tags || []
}

function extractArgumentHint(fm: SkillFrontmatter): string {
  return fm['argument-hint'] || ''
}

async function checkGating(entry: SkillEntry): Promise<SkillGatingStatus> {
  const requires = extractRequires(entry.frontmatter)
  if (!requires) {
    return { eligible: true }
  }

  const status: SkillGatingStatus = { eligible: true }

  if (requires.os && requires.os.length > 0) {
    const currentOs = getCurrentOs()
    if (!requires.os.includes(currentOs)) {
      status.eligible = false
      status.osMismatch = true
    }
  }

  if (requires.bins && requires.bins.length > 0) {
    const missing: string[] = []
    for (const bin of requires.bins) {
      const exists = await checkBinExists(bin)
      if (!exists) {
        missing.push(bin)
      }
    }
    if (missing.length > 0) {
      status.eligible = false
      status.missingBins = missing
    }
  }

  if (requires.env && requires.env.length > 0) {
    status.missingEnv = []
  }

  return status
}

async function getIndexPath(): Promise<string> {
  const dirs = await resolveSkillDirs()
  return `${dirs.skills}/_index.json`
}

async function readIndexFile(indexPath: string): Promise<SkillIndex | null> {
  try {
    const content = await invoke<string>('file_read', { path: indexPath, raw: true })
    return JSON.parse(content) as SkillIndex
  } catch {
    return null
  }
}

async function saveIndex(): Promise<void> {
  if (!skillIndex) return
  const indexPath = await getIndexPath()
  await invoke('file_write', {
    path: indexPath,
    content: JSON.stringify(skillIndex, null, 2),
  })
}

async function buildIndex(): Promise<void> {
  const entries: Record<string, SkillIndexEntry> = {}

  for (const [name, entry] of skillEntries) {
    const gatingStatus = await checkGating(entry)
    entries[name] = {
      name,
      description: entry.frontmatter.description,
      emoji: extractEmoji(entry.frontmatter),
      tags: extractTags(entry.frontmatter),
      argumentHint: extractArgumentHint(entry.frontmatter),
      filePath: entry.filePath,
      skillDir: entry.filePath.replace(/[/\\]SKILL\.md$/i, ''),
      source: entry.source,
      gatingStatus,
      enabled: true,
      readonly: entry.frontmatter.readonly !== false,
    }
  }

  skillIndex = {
    version: CURRENT_INDEX_VERSION,
    entries,
    updatedAt: new Date().toISOString(),
  }

  try {
    await saveIndex()
  } catch (e) {
    console.warn('[skillManager] saveIndex failed:', e)
  }
}

const CURRENT_INDEX_VERSION = 4

async function isIndexUpToDate(): Promise<boolean> {
  const indexPath = await getIndexPath()
  const cached = await readIndexFile(indexPath)
  if (!cached) return false

  if ((cached as any).version !== CURRENT_INDEX_VERSION) return false

  const indexNames = new Set(Object.keys(cached.entries))
  const loadedNames = new Set(skillEntries.keys())

  if (indexNames.size !== loadedNames.size) return false
  for (const name of loadedNames) {
    if (!indexNames.has(name)) return false
  }
  return true
}

export async function loadAllSkillsToManager(): Promise<void> {
  const skills = await loadAllSkills()
  skillEntries.clear()
  for (const skill of skills) {
    skillEntries.set(skill.frontmatter.name, skill)
  }

  const upToDate = await isIndexUpToDate()
  if (upToDate) {
    const indexPath = await getIndexPath()
    skillIndex = await readIndexFile(indexPath)
    if (skillIndex) {
      for (const [, entry] of Object.entries(skillIndex.entries)) {
        const skillEntry = skillEntries.get(entry.name)
        const freshGating = await checkGating({
          frontmatter: skillEntry?.frontmatter || {} as SkillFrontmatter,
        } as SkillEntry)
        entry.gatingStatus = freshGating
      }
      initialized = true
      return
    }
  }

  try {
    await buildIndex()
  } catch (e) {
    console.warn('[skillManager] buildIndex failed:', e)
  }
  initialized = true
}

export async function matchSkillsForMessage(
  userMessage: string,
  llm: LLMAdapter,
  recentMessages?: Array<{ role: string; content: string }>
): Promise<SkillIndexEntry[]> {
  if (!initialized) {
    await loadAllSkillsToManager()
  }
  if (!skillIndex) return []
  const allEntries = Object.values(skillIndex.entries)
  return selectSkills(userMessage, allEntries, llm, recentMessages)
}

export async function getSkillContent(skillName: string): Promise<string> {
  const entry = skillEntries.get(skillName)
  return entry?.content ?? ''
}

export async function getSkillContentMap(names: string[]): Promise<Map<string, string>> {
  const map = new Map<string, string>()
  for (const name of names) {
    const content = await getSkillContent(name)
    map.set(name, content)
  }
  return map
}

export function getAllIndexEntries(): SkillIndexEntry[] {
  if (!skillIndex) return []
  return Object.values(skillIndex.entries)
}

export async function enableSkill(name: string): Promise<void> {
  if (!skillIndex) return
  const entry = skillIndex.entries[name]
  if (entry) {
    entry.enabled = true
    await saveIndex()
  }
}

export async function disableSkill(name: string): Promise<void> {
  if (!skillIndex) return
  const entry = skillIndex.entries[name]
  if (entry) {
    entry.enabled = false
    await saveIndex()
  }
}

async function editFieldInFile(
  filePath: string,
  fieldPattern: string,
  oldValue: string,
  newValue: string
): Promise<boolean> {
  const patterns = [
    `${fieldPattern}: "${oldValue}"`,
    `${fieldPattern}: '${oldValue}'`,
    `${fieldPattern}: ${oldValue}`,
  ]
  for (const oldStr of patterns) {
    try {
      await invoke('file_edit', {
        path: filePath,
        oldStr,
        newStr: `${fieldPattern}: "${newValue}"`,
      })
      return true
    } catch {
      continue
    }
  }
  return false
}

async function editArrayFieldInFile(
  filePath: string,
  fieldPattern: string,
  oldValues: string[],
  newValues: string[]
): Promise<boolean> {
  const oldInline = `${fieldPattern}: [${oldValues.map(v => `"${v}"`).join(', ')}]`
  const oldInlineAlt = `${fieldPattern}: [${oldValues.map(v => `'${v}'`).join(', ')}]`
  const oldInlineBare = `${fieldPattern}: [${oldValues.join(', ')}]`
  const newInline = `${fieldPattern}: [${newValues.map(v => `"${v}"`).join(', ')}]`

  for (const oldStr of [oldInline, oldInlineAlt, oldInlineBare]) {
    try {
      await invoke('file_edit', {
        path: filePath,
        oldStr,
        newStr: newInline,
      })
      return true
    } catch {
      continue
    }
  }

  const oldMultiline = `${fieldPattern}:\n${oldValues.map(v => `  - "${v}"`).join('\n')}`
  const oldMultilineAlt = `${fieldPattern}:\n${oldValues.map(v => `  - ${v}`).join('\n')}`
  const newMultiline = `${fieldPattern}:\n${newValues.map(v => `  - "${v}"`).join('\n')}`

  for (const oldStr of [oldMultiline, oldMultilineAlt]) {
    try {
      await invoke('file_edit', {
        path: filePath,
        oldStr,
        newStr: newMultiline,
      })
      return true
    } catch {
      continue
    }
  }
  return false
}

export async function updateSkillMetadata(
  skillName: string,
  updates: {
    description?: string
    tags?: string[]
    argumentHint?: string
  }
): Promise<void> {
  if (!skillIndex) return
  const indexEntry = skillIndex.entries[skillName]
  const skillEntry = skillEntries.get(skillName)
  if (!indexEntry || !skillEntry) return

  if (updates.description !== undefined && updates.description !== indexEntry.description) {
    indexEntry.description = updates.description
    try {
      const oldVal = skillEntry.frontmatter.description
      await editFieldInFile(skillEntry.filePath, 'description', oldVal, updates.description)
      skillEntry.frontmatter.description = updates.description
    } catch (e) {
      console.warn('[skillManager] updateSkillMetadata description failed:', e)
    }
  }

  if (updates.tags !== undefined) {
    const oldTags = extractTags(skillEntry.frontmatter)
    const newTags = updates.tags
    if (JSON.stringify(oldTags) !== JSON.stringify(newTags)) {
      indexEntry.tags = newTags
      try {
        const tagField = skillEntry.frontmatter.tags ? 'tags' : null
        const openclawTags = skillEntry.frontmatter.metadata?.openclaw?.tags
        if (tagField) {
          await editArrayFieldInFile(skillEntry.filePath, 'tags', oldTags, newTags)
          skillEntry.frontmatter.tags = newTags
        } else if (openclawTags) {
          await editArrayFieldInFile(skillEntry.filePath, 'tags', oldTags, newTags)
          if (skillEntry.frontmatter.metadata?.openclaw) {
            skillEntry.frontmatter.metadata.openclaw.tags = newTags
          }
        }
      } catch (e) {
        console.warn('[skillManager] updateSkillMetadata tags failed:', e)
      }
    }
  }

  if (updates.argumentHint !== undefined) {
    const oldHint = extractArgumentHint(skillEntry.frontmatter)
    const newHint = updates.argumentHint
    if (oldHint !== newHint) {
      indexEntry.argumentHint = newHint
      try {
        await editFieldInFile(skillEntry.filePath, 'argument-hint', oldHint, newHint)
        skillEntry.frontmatter['argument-hint'] = newHint
      } catch (e) {
        console.warn('[skillManager] updateSkillMetadata argumentHint failed:', e)
      }
    }
  }

  await saveIndex()
}

export async function getSkillsDir(): Promise<string> {
  const dirs = await resolveSkillDirs()
  return dirs.skills
}

export async function installFromGit(repoUrl: string): Promise<{ success: boolean; message: string }> {
  const skillsDir = await getSkillsDir()
  const repoName = repoUrl.replace(/\/+$/, '').split('/').pop() || 'skill'
  const skillName = repoName.replace(/\.git$/, '')
  const targetPath = `${skillsDir}/${skillName}`

  try {
    await invoke<string>('shell_execute', {
      command: `mkdir "${skillsDir.replace(/\//g, '\\')}" 2>nul || echo ok`,
      timeout: 5,
    })
  } catch {
    // directory may already exist
  }

  try {
    await invoke<string>('shell_execute', {
      command: `git clone ${repoUrl} ${targetPath}`,
      timeout: 120,
    })
    await forceReloadSkills()
    return { success: true, message: `技能 ${skillName} 安装成功` }
  } catch (e: any) {
    return { success: false, message: `安装失败: ${e?.toString() || '未知错误'}` }
  }
}

export async function forceReloadSkills(): Promise<void> {
  const skills = await loadAllSkills()
  skillEntries.clear()
  for (const skill of skills) {
    skillEntries.set(skill.frontmatter.name, skill)
  }
  try {
    await buildIndex()
  } catch (e) {
    console.warn('[skillManager] forceReloadSkills buildIndex failed:', e)
  }
  initialized = true
}

export async function installByCommand(command: string): Promise<{ success: boolean; message: string }> {
  try {
    await invoke<string>('shell_execute', {
      command,
      timeout: 120,
    })
    await forceReloadSkills()
    return { success: true, message: '安装命令执行完成' }
  } catch (e: any) {
    return { success: false, message: `安装失败: ${e?.toString() || '未知错误'}` }
  }
}

export async function uninstallSkill(name: string): Promise<{ success: boolean; message: string }> {
  if (!skillIndex) return { success: false, message: '索引未初始化' }
  const entry = skillIndex.entries[name]
  if (!entry) return { success: false, message: `技能 ${name} 不存在` }

  try {
    const dir = entry.filePath.replace(/[/\\]SKILL\.md$/, '').replace(/\//g, '\\')
    await invoke('shell_execute', {
      command: `rmdir /s /q "${dir}"`,
      timeout: 10,
    })
    delete skillIndex.entries[name]
    skillEntries.delete(name)
    await saveIndex()
    return { success: true, message: `技能 ${name} 已卸载` }
  } catch (e: any) {
    return { success: false, message: `卸载失败: ${e?.toString() || '未知错误'}` }
  }
}

export async function openSkillsDir(): Promise<void> {
  const skillsDir = await getSkillsDir()
  try {
    await invoke('open_folder', { path: skillsDir })
  } catch {
    console.warn('Failed to open skills directory')
  }
}

export async function setReadonly(name: string, value: boolean): Promise<void> {
  if (!skillIndex) return
  const entry = skillIndex.entries[name]
  if (!entry) return
  entry.readonly = value
  await saveIndex()

  const skillEntry = skillEntries.get(name)
  if (skillEntry) {
    skillEntry.frontmatter.readonly = value
  }
}

export function getWritableSkillDirs(activatedNames: string[]): string[] {
  if (!skillIndex) return []
  return activatedNames
    .map(name => skillIndex!.entries[name])
    .filter(entry => entry && !entry.readonly)
    .map(entry => entry.skillDir)
}

export const skillManager = {
  loadAllSkills: loadAllSkillsToManager,
  matchSkills: matchSkillsForMessage,
  getSkillContent,
  getSkillContentMap,
  getAllIndexEntries,
  enableSkill,
  disableSkill,
  updateSkillMetadata,
  setReadonly,
  getWritableSkillDirs,
  reloadSkills: forceReloadSkills,
  installFromGit,
  installByCommand,
  uninstallSkill,
  openSkillsDir,
  getSkillsDir,
}
