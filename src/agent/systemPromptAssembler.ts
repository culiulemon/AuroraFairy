import soulFallback from '../fbm/templates/SOUL.md?raw'
import habitFallback from '../fbm/templates/HABIT.md?raw'
import syspromptFallback from '../fbm/templates/SYSPROMPT.md?raw'
import { loadSettings } from '../stores/settings.js'

const FRONTMATTER_REGEX = /^---\n[\s\S]*?\n---\n*/
const COMMENT_BLOCK_REGEX = /<!--[\s\S]*?-->/g

function stripFrontmatter(content: string): string {
  return content.replace(FRONTMATTER_REGEX, '').trim()
}

function stripCommentBlocks(content: string): string {
  return content.replace(COMMENT_BLOCK_REGEX, '').trim()
}

function processTemplate(content: string, variables: Record<string, string>): string {
  let result = content
  for (const [key, value] of Object.entries(variables)) {
    result = result.split(`{{${key}}}`).join(value)
  }
  return result
}

export function assembleMemoryPrompt(smartRecall?: boolean): string {
  const settings = loadSettings()
  if (!settings.fbmEnabled) return ''

  const isSmartRecall = smartRecall !== undefined ? smartRecall : (settings.fbmSmartRecall !== false)

  if (isSmartRecall) {
    return [
      '## 记忆系统',
      '',
      '你可以通过 `memory_search` 工具搜索过往记忆。当你需要回忆之前对话的内容、用户偏好、项目背景等信息时，直接调用该工具，无需传入任何参数。',
      '工具会自动使用用户消息进行检索，并返回相关的记忆片段，请参考它们回答用户问题。',
      '**重要：记忆的存储、检索、总结均由系统自动完成。你绝对不要手动创建、编辑或写入任何记忆文件。**',
    ].join('\n')
  } else {
    return [
      '## 记忆系统',
      '',
      '系统已根据当前对话自动检索相关记忆并提供给你。请参考这些记忆内容回答用户问题。',
      '**重要：记忆的存储、检索、总结均由系统自动完成。你绝对不要手动创建、编辑或写入任何记忆文件。**',
    ].join('\n')
  }
}

function assembleEnvironmentPrompt(): string {
  const now = new Date()
  const timeStr = now.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    weekday: 'long',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  })

  const ua = navigator.userAgent
  let osInfo = 'Unknown'
  if (ua.includes('Windows NT 10')) osInfo = 'Windows 10/11'
  else if (ua.includes('Windows NT 6.3')) osInfo = 'Windows 8.1'
  else if (ua.includes('Windows NT 6.1')) osInfo = 'Windows 7'
  else if (ua.includes('Mac OS X')) {
    const match = ua.match(/Mac OS X ([\d_]+)/)
    osInfo = match ? `macOS ${match[1].replace(/_/g, '.')}` : 'macOS'
  } else if (ua.includes('Linux')) osInfo = 'Linux'

  return [
    '## 环境信息',
    '',
    `- 当前时间: ${timeStr}`,
    `- 操作系统: ${osInfo}`,
  ].join('\n')
}

function assembleRoleConfigPrompt(): string {
  return [
    '## 自进化',
    '',
    '如果有需要，你可以通过 `role_config` 工具管理自己的角色身份。使用 `action=get` 查看当前配置，使用 `action=update` 修改配置。可配置字段：',
    '- fairyName: 你的名字',
    '- userName: 对用户的称呼',
    '- fairyPositioning: 角色定位',
    '- fairyStyle: 个性',
    '- fairySupplement: 你的人格',
    '- habitSupplement: 你的行为习惯',
    '',
    '修改会立即生效。如果用户希望你或者你自己想要调整性格、称呼、角色定位、个性、人格、行为习惯、对话习惯、规则性内容等等，可以使用此工具。',
  ].join('\n')
}

async function readCorefile(fileName: string, fallback: string): Promise<string> {
  try {
    const { loadMiscSettings, getEffectiveWorkingDir } = await import('../stores/miscSettings.js')
    const { invoke } = await import('@tauri-apps/api/core')
    const miscSettings = await loadMiscSettings()
    const baseDir = getEffectiveWorkingDir(miscSettings)
    const corefileDir = `${baseDir}/memories/corefile`
    const filePath = `${corefileDir}/${fileName}`
    try {
      return await invoke<string>('fbm_read_file', { path: filePath })
    } catch {
      await invoke('fbm_mkdir', { path: corefileDir }).catch(() => {})
      await invoke('fbm_write_file', { path: filePath, content: fallback })
      return fallback
    }
  } catch {
    return fallback
  }
}

export async function assembleSystemPrompt(skillsPrompt?: string): Promise<string> {
  const settings = loadSettings()
  const [soulRaw, habitRaw, syspromptRaw] = await Promise.all([
    readCorefile('SOUL.md', soulFallback),
    readCorefile('HABIT.md', habitFallback),
    readCorefile('SYSPROMPT.md', syspromptFallback),
  ])

  const cleanSoul = stripFrontmatter(soulRaw)
  const cleanHabit = stripFrontmatter(habitRaw)
  const cleanSysprompt = stripFrontmatter(syspromptRaw)

  const processedSoul = processTemplate(cleanSoul, {
    FairyName: settings.fairyName || 'Fairy',
    User: settings.userName || '主人',
    Positioning: settings.fairyPositioning || '智能助手',
    Style: settings.fairyStyle || '温柔体贴',
    Supplement: settings.fairySupplement || '',
  })

  const processedHabit = processTemplate(cleanHabit, {
    User: settings.userName || '主人',
    user: settings.userName || '主人',
    HabitSupplement: settings.habitSupplement || '',
  })

  const finalSysprompt = stripCommentBlocks(cleanSysprompt)

  const memoryPrompt = assembleMemoryPrompt()

  const roleConfigPrompt = assembleRoleConfigPrompt()

  const environmentPrompt = assembleEnvironmentPrompt()

  let result = `${processedSoul}\n\n---\n\n${processedHabit}\n\n---\n\n${finalSysprompt}\n\n---\n\n${environmentPrompt}`
  if (memoryPrompt) {
    result += `\n\n---\n\n${memoryPrompt}`
  }
  if (skillsPrompt) {
    result += `\n\n---\n\n${skillsPrompt}`
  }
  if (roleConfigPrompt) {
    result += `\n\n---\n\n${roleConfigPrompt}`
  }
  return result
}
