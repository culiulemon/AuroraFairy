import type { LLMAdapter, LLMMessage } from '../../fbm/src/types/adapter.js'
import type { SkillIndexEntry } from './types.js'

const SKILL_SELECT_PROMPT = `你是一个技能选择助手。以下是当前可用的技能列表，请根据用户的消息选择最相关的技能。

## 可用技能
{indexText}

## 用户消息
{userMessage}

## 输出格式
输出JSON：
{
  "selectedSkills": ["技能名称1", "技能名称2"],
  "reasoning": "简述选择理由"
}

选择标准：
- 只选择与用户消息直接相关的技能
- 最多选择3个技能
- 如果没有任何相关技能，返回空数组
- 宁可多选不要漏选
- 参考每个技能的标签(tags)和使用示例(argument-hint)来判断相关性

只输出JSON，不要输出任何其他内容。`

export function buildSkillIndexText(entries: SkillIndexEntry[]): string {
  return entries
    .map((entry, i) => {
      let line = `${i + 1}. **${entry.name}** (${entry.emoji}): ${entry.description}`
      if (entry.tags && entry.tags.length > 0) {
        line += `\n   标签: ${entry.tags.join(', ')}`
      }
      if (entry.argumentHint) {
        line += `\n   使用示例: ${entry.argumentHint}`
      }
      return line
    })
    .join('\n\n')
}

function parseSelectedSkills(content: string): string[] {
  const trimmed = content.trim()
  const jsonMatch = trimmed.match(/\{[\s\S]*\}/)
  if (!jsonMatch) return []

  try {
    const parsed = JSON.parse(jsonMatch[0])
    if (Array.isArray(parsed.selectedSkills)) {
      return parsed.selectedSkills.filter((id: unknown) => typeof id === 'string')
    }
  } catch {
    // fall through
  }
  return []
}

export async function selectSkills(
  userMessage: string,
  entries: SkillIndexEntry[],
  llm: LLMAdapter
): Promise<SkillIndexEntry[]> {
  const eligible = entries.filter((e) => e.enabled && e.gatingStatus.eligible)
  if (eligible.length === 0) return []

  const indexText = buildSkillIndexText(eligible)
  const prompt = SKILL_SELECT_PROMPT
    .replace('{indexText}', indexText)
    .replace('{userMessage}', userMessage)

  try {
    const messages: LLMMessage[] = [{ role: 'system', content: prompt }]
    const response = await llm.chat(messages, { temperature: 0.3 })
    const selectedNames = parseSelectedSkills(response.content)
    const nameSet = new Set(selectedNames)

    return eligible.filter((entry) => nameSet.has(entry.name))
  } catch {
    return []
  }
}
