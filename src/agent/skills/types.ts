export interface SkillRequires {
  bins?: string[]
  env?: string[]
  optionalEnv?: string[]
  os?: ('windows' | 'macos' | 'linux')[]
}

export interface SkillFrontmatter {
  name: string
  description: string
  emoji?: string
  version?: string
  'argument-hint'?: string
  'allowed-tools'?: string[]
  'user-invocable'?: boolean
  readonly?: boolean
  requires?: SkillRequires
  tags?: string[]
  metadata?: {
    openclaw?: {
      emoji?: string
      requires?: {
        env?: string[]
        optionalEnv?: string[]
        bins?: string[]
      }
      primaryEnv?: string
      tags?: string[]
      files?: string[]
      homepage?: string
    }
    [key: string]: unknown
  }
  [key: string]: unknown
}

export type SkillSource = 'workspace' | 'user' | 'builtin'

export interface SkillEntry {
  frontmatter: SkillFrontmatter
  content: string
  filePath: string
  source: SkillSource
}

export interface SkillGatingStatus {
  eligible: boolean
  missingBins?: string[]
  missingEnv?: string[]
  osMismatch?: boolean
}

export interface SkillIndexEntry {
  name: string
  description: string
  emoji: string
  tags: string[]
  argumentHint: string
  filePath: string
  skillDir: string
  source: SkillSource
  gatingStatus: SkillGatingStatus
  enabled: boolean
  readonly: boolean
}

export interface SkillIndex {
  version: number
  entries: Record<string, SkillIndexEntry>
  updatedAt: string
}

export interface SkillEvolveResult {
  needsUpdate: boolean
  newDescription: string | null
  newTags: string[] | null
  newArgumentHint: string | null
}
