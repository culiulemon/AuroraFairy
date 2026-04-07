export type ChannelId = string

export type FeishuConnectionMode = 'websocket' | 'webhook'

export type FeishuDomain = 'feishu' | 'lark'

export interface FeishuAccountConfig {
  appId: string
  appSecret: string
  connectionMode: FeishuConnectionMode
  domain: FeishuDomain
  verificationToken?: string
  encryptKey?: string
  webhookPort?: number
  webhookPath?: string
}

export type ChannelAccountConfig = FeishuAccountConfig | WeixinAccountConfig | Record<string, unknown>

export interface WeixinAccountConfig {
  botToken: string
  ilinkBotId: string
  ilinkUserId: string
  contextToken?: string
}

export interface ChannelAccount {
  id: string
  channelId: ChannelId
  displayName: string
  enabled: boolean
  config: ChannelAccountConfig
}

export type FeishuAccount = ChannelAccount & { channelId: 'feishu'; config: FeishuAccountConfig }

export function isFeishuAccount(account: ChannelAccount): account is FeishuAccount {
  return account.channelId === 'feishu'
}

export type ConfigFieldType = 'text' | 'password' | 'number' | 'select' | 'qrcode'

export interface ConfigFieldOption {
  value: string
  label: string
}

export interface ConfigFieldDefinition {
  key: string
  label: string
  type: ConfigFieldType
  placeholder?: string
  required?: boolean
  defaultValue?: string | number
  options?: ConfigFieldOption[]
  condition?: { field: string; value: string }
}
