import type { UnlistenFn } from '@tauri-apps/api/event'
import type { ConfigFieldDefinition, ChannelAccountConfig } from '../types/channel'
import type { DispatcherDeps } from './messageDispatcher'

export type StatusChangeCallback = (accountId: string, status: string, error?: string) => void

export interface ReplyContext {
  lastMessageId: string
  [key: string]: unknown
}

export interface ChannelBadgeIcon {
  svgContent: string
  backgroundColor: string
}

export interface ChannelBridge {
  readonly channelId: string
  readonly displayName: string
  readonly badgeIcon: ChannelBadgeIcon

  start(deps: DispatcherDeps): Promise<void>
  stop(): void
  connect(accountId: string, config: ChannelAccountConfig): Promise<void>
  disconnect(accountId: string): Promise<void>
  getStatus(accountId: string): Promise<string>
  onStatusChange(callback: StatusChangeCallback): Promise<UnlistenFn>
  sendReply(accountId: string, externalChatId: string, messageText: string, replyContext: ReplyContext): Promise<void>
  getConfigFields(): ConfigFieldDefinition[]
  validateConfig(config: Record<string, unknown>): { valid: boolean; errors?: Record<string, string> }
}
