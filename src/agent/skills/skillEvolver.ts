import type { LLMAdapter, LLMMessage } from '../../fbm/src/types/adapter.js'
import type { SkillEvolveResult } from './types.js'

const EVOLVE_PROMPT = `你是一个技能优化助手。请分析以下对话片段，判断是否需要改进技能的匹配信息以使其更容易被正确匹配。

## 当前技能信息
- 描述(description): {currentDescription}
- 标签(tags): {currentTags}
- 使用示例(argument-hint): {currentArgumentHint}

## 用户消息
{userMessage}

## 助手回复摘要
{assistantResponse}

## 输出格式
输出JSON：
{
  "needsUpdate": true或false,
  "newDescription": "改进后的描述，如果不需要改则为null",
  "newTags": ["改进后的标签数组，如果不需要改则为null"],
  "newArgumentHint": "改进后的使用示例，如果不需要改则为null",
  "reasoning": "简述改进理由"
}

改进原则：
- 描述应更准确地反映技能的适用场景，添加用户实际使用时提到的相关概念
- 标签应包含用户可能使用的搜索关键词、相关平台名、功能类别等
- 使用示例应反映用户实际的调用方式
- 每个字段如果已经足够好，设为 null 表示不需要修改
- 如果所有字段都不需要修改，设置 needsUpdate 为 false
- 描述保持简洁，不超过两句话
- 标签数量控制在5-15个

只输出JSON，不要输出任何其他内容。`

function parseEvolveResult(content: string): SkillEvolveResult {
  const trimmed = content.trim()
  const jsonMatch = trimmed.match(/\{[\s\S]*\}/)
  if (!jsonMatch) return { needsUpdate: false, newDescription: null, newTags: null, newArgumentHint: null }

  try {
    const parsed = JSON.parse(jsonMatch[0])
    if (!parsed.needsUpdate) {
      return { needsUpdate: false, newDescription: null, newTags: null, newArgumentHint: null }
    }
    return {
      needsUpdate: true,
      newDescription: typeof parsed.newDescription === 'string' ? parsed.newDescription : null,
      newTags: Array.isArray(parsed.newTags) ? parsed.newTags : null,
      newArgumentHint: typeof parsed.newArgumentHint === 'string' ? parsed.newArgumentHint : null,
    }
  } catch {
    // fall through
  }
  return { needsUpdate: false, newDescription: null, newTags: null, newArgumentHint: null }
}

export async function evolveSkillMetadata(
  currentDescription: string,
  currentTags: string[],
  currentArgumentHint: string,
  userMessage: string,
  assistantResponse: string,
  llm: LLMAdapter
): Promise<SkillEvolveResult> {
  const prompt = EVOLVE_PROMPT
    .replace('{currentDescription}', currentDescription)
    .replace('{currentTags}', currentTags.length > 0 ? currentTags.join(', ') : '（无）')
    .replace('{currentArgumentHint}', currentArgumentHint || '（无）')
    .replace('{userMessage}', userMessage)
    .replace('{assistantResponse}', assistantResponse.slice(0, 500))

  try {
    const messages: LLMMessage[] = [{ role: 'system', content: prompt }]
    const response = await llm.chat(messages, { temperature: 0.3 })
    return parseEvolveResult(response.content)
  } catch {
    return { needsUpdate: false, newDescription: null, newTags: null, newArgumentHint: null }
  }
}
