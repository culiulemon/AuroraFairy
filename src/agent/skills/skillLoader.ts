import { invoke } from '@tauri-apps/api/core'
import yaml from 'js-yaml'
import { loadMiscSettings, getEffectiveWorkingDir } from '../../stores/miscSettings.js'
import type { SkillEntry, SkillFrontmatter, SkillSource } from './types.js'

export function parseFrontmatter(content: string): { frontmatter: Record<string, any>; body: string } {
  if (!content.startsWith('---')) {
    return { frontmatter: {}, body: content }
  }
  const endIndex = content.indexOf('---\n', 3)
  if (endIndex === -1) {
    const endAlt = content.indexOf('---\r\n', 3)
    if (endAlt === -1) {
      return { frontmatter: {}, body: content }
    }
    const fmStr = content.slice(3, endAlt)
    const body = content.slice(endAlt + 5)
    return { frontmatter: yaml.load(fmStr) as Record<string, any>, body }
  }
  const fmStr = content.slice(3, endIndex)
  const body = content.slice(endIndex + 4)
  return { frontmatter: yaml.load(fmStr) as Record<string, any>, body }
}

export async function parseSkillMd(filePath: string): Promise<SkillEntry | null> {
  try {
    const content = await invoke<string>('file_read', { path: filePath, raw: true })
    const { frontmatter, body } = parseFrontmatter(content)
    if (typeof frontmatter.name !== 'string' || typeof frontmatter.description !== 'string') {
      console.warn(`Skill file missing name or description: ${filePath}`)
      return null
    }
    return {
      frontmatter: frontmatter as SkillFrontmatter,
      content: body,
      filePath,
      source: 'workspace' as SkillSource,
    }
  } catch (e) {
    return null
  }
}

export async function scanSkillDir(dir: string, source: SkillSource): Promise<SkillEntry[]> {
  let skillPaths: string[]
  try {
    skillPaths = await invoke<string[]>('file_glob', { pattern: `${dir}/**/SKILL.md` })
  } catch {
    return []
  }
  const entries = await Promise.all(skillPaths.map(p => parseSkillMd(p)))
  return entries.filter((e): e is SkillEntry => e !== null).map(e => {
    e.source = source
    return e
  })
}

function ensureDirPath(base: string, ...parts: string[]): string {
  const sep = base.includes('\\') ? '\\' : '/'
  let result = base.replace(/[\/\\]+$/, '')
  for (const part of parts) {
    result = result + sep + part
  }
  return result
}

export async function resolveSkillDirs(): Promise<{ skills: string }> {
  const settings = await loadMiscSettings()
  const workDir = getEffectiveWorkingDir(settings)
  return {
    skills: ensureDirPath(workDir, 'skills'),
  }
}

export async function loadAllSkills(): Promise<SkillEntry[]> {
  const dirs = await resolveSkillDirs()
  const skills = await scanSkillDir(dirs.skills, 'user')
  return skills
}
