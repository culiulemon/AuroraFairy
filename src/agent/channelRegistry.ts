import type { UnlistenFn } from '@tauri-apps/api/event'
import type { ChannelBridge, ChannelBadgeIcon, StatusChangeCallback, ReplyContext } from './channelBridge'
import type { DispatcherDeps } from './messageDispatcher'
import type { ChannelAccount } from '../types/channel'
import { FeishuChannelBridge } from './feishuBridge'
import { WeixinChannelBridge } from './weixinBridge'
import { getChannelAccounts } from '../stores/channelStore'

class ChannelRegistryImpl {
  private bridges = new Map<string, ChannelBridge>()
  private started = false
  private statusUnlisten: UnlistenFn | null = null

  register(bridge: ChannelBridge): void {
    this.bridges.set(bridge.channelId, bridge)
  }

  get(channelId: string): ChannelBridge | undefined {
    return this.bridges.get(channelId)
  }

  getAll(): ChannelBridge[] {
    return Array.from(this.bridges.values())
  }

  getBadge(channelId: string | undefined): ChannelBadgeIcon | undefined {
    if (!channelId) return undefined
    return this.bridges.get(channelId)?.badgeIcon
  }

  startAll(deps: DispatcherDeps): void {
    if (this.started) return
    this.started = true

    Promise.resolve().then(async () => {
      for (const bridge of this.bridges.values()) {
        try {
          await bridge.start(deps)
        } catch (err) {
          console.error(`[ChannelRegistry] Failed to start bridge ${bridge.channelId}:`, err)
        }
      }

      const accounts = getChannelAccounts()
      for (const account of accounts) {
        if (!account.enabled) continue
        try {
          await this.connectAccount(account)
        } catch (err) {
          console.warn(`[ChannelRegistry] Auto-connect failed for ${account.displayName}:`, err)
        }
      }
    })
  }

  stopAll(): void {
    for (const bridge of this.bridges.values()) {
      try {
        bridge.stop()
      } catch (err) {
        console.error(`[ChannelRegistry] Failed to stop bridge ${bridge.channelId}:`, err)
      }
    }
    if (this.statusUnlisten) {
      this.statusUnlisten()
      this.statusUnlisten = null
    }
    this.started = false
  }

  async connectAccount(account: ChannelAccount): Promise<void> {
    const bridge = this.bridges.get(account.channelId)
    if (!bridge) {
      console.warn(`[ChannelRegistry] No bridge registered for channel: ${account.channelId}`)
      return
    }
    try {
      await bridge.connect(account.id, account.config)
    } catch (err) {
      console.warn(`[ChannelRegistry] Auto-connect failed for ${account.displayName}:`, err)
    }
  }

  async disconnectAccount(accountId: string, channelId: string): Promise<void> {
    const bridge = this.bridges.get(channelId)
    if (!bridge) return
    await bridge.disconnect(accountId)
  }

  async sendReply(
    source: string,
    accountId: string,
    externalChatId: string,
    messageText: string,
    replyContext: ReplyContext,
  ): Promise<void> {
    const bridge = this.bridges.get(source)
    if (!bridge) {
      console.warn(`[ChannelRegistry] No bridge for source: ${source}`)
      return
    }
    await bridge.sendReply(accountId, externalChatId, messageText, replyContext)
  }

  async getStatus(channelId: string, accountId: string): Promise<string> {
    const bridge = this.bridges.get(channelId)
    if (!bridge) return 'unknown'
    return bridge.getStatus(accountId)
  }

  async onStatusChange(callback: StatusChangeCallback): Promise<UnlistenFn> {
    const unlistenFns: UnlistenFn[] = []
    for (const bridge of this.bridges.values()) {
      try {
        const unlisten = await bridge.onStatusChange(callback)
        unlistenFns.push(unlisten)
      } catch (err) {
        console.error(`[ChannelRegistry] Failed to listen status for ${bridge.channelId}:`, err)
      }
    }
    return () => {
      for (const fn of unlistenFns) fn()
    }
  }
}

const channelRegistry = new ChannelRegistryImpl()

channelRegistry.register(new FeishuChannelBridge())
channelRegistry.register(new WeixinChannelBridge())

export { channelRegistry }
