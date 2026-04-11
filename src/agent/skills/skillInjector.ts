import type { SkillIndexEntry } from './types.js'

export function formatActiveSkillPrompt(
  activeSkills: SkillIndexEntry[],
  skillContentMap: Map<string, string>
): string {
  if (activeSkills.length === 0) return ''

  const sections = activeSkills.map((skill) => {
    const { name, emoji, readonly, skillDir } = skill
    const title = emoji ? `${emoji} ${name}` : name
    const content = skillContentMap.get(name) ?? '（技能文件内容不可用）'
    const permNote = readonly
      ? '（只读：仅允许自进化匹配描述，不可修改技能文件）'
      : `（可写：技能目录 ${skillDir}，你可以通过文件工具修改技能文件实现自进化）`
    return `### 技能: ${title} ${permNote}\n${content}`
  })

  return `## 激活的技能\n\n以下技能与当前任务相关，请参考其中的指南：\n\n${sections.join('\n\n---\n\n')}\n\n---`
}

export function buildSkillsPrompt(
  activatedSkills: SkillIndexEntry[],
  skillContentMap: Map<string, string>
): string {
  const result = formatActiveSkillPrompt(activatedSkills, skillContentMap)
  return result === '' ? '' : result
}
