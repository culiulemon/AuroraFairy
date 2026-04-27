import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { dispatchMessage, type DispatcherCallbacks, type DispatcherDeps } from './messageDispatcher'
import { useConversationStore } from '../stores/useConversationStore'
import { createTextContent, updateConversationMeta } from '../stores/conversation'
import type { ChannelBridge, StatusChangeCallback, ReplyContext } from './channelBridge'
import type { ConfigFieldDefinition, ChannelAccountConfig } from '../types/channel'

interface WeixinInboundEvent {
  accountId: string
  fromUserId: string
  text: string
  contextToken: string
}

interface WeixinLoginResultEvent {
  accountId: string
  success: boolean
  error?: string
  botToken?: string
  ilinkBotId?: string
  ilinkUserId?: string
}

interface WeixinQrCodeEvent {
  accountId: string
  qrcodeUrl: string
}

type QrCodeStatusListener = (event: WeixinLoginResultEvent) => void
type QrCodeUrlListener = (url: string) => void

const qrCodeStatusListeners = new Set<QrCodeStatusListener>()
const qrCodeUrlListeners = new Map<string, QrCodeUrlListener>()

export function onQrCodeStatus(listener: QrCodeStatusListener): () => void {
  qrCodeStatusListeners.add(listener)
  return () => qrCodeStatusListeners.delete(listener)
}

export function onQrCodeUrl(accountId: string, listener: QrCodeUrlListener): () => void {
  qrCodeUrlListeners.set(accountId, listener)
  return () => qrCodeUrlListeners.delete(accountId)
}

export async function requestQrCode(accountId: string): Promise<string> {
  const result = await invoke<string>('weixin_get_qrcode', { accountId })
  return result
}

export async function checkWeixinCredentials(accountId: string): Promise<boolean> {
  return invoke<boolean>('weixin_has_credentials', { accountId })
}

export class WeixinChannelBridge implements ChannelBridge {
  readonly channelId = 'weixin'
  readonly displayName = '微信'
  readonly badgeIcon = {
    backgroundColor: '#07c160',
    svgContent: '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="56" height="56" viewBox="0 0 56 56" fill="none"><path d="M20.0854 39.1034C17.6694 39.1034 15.7349 38.6017 13.4394 38.0385L6.79751 41.5449L8.6701 35.4759C3.89834 31.9726 1 27.4035 1 21.9613C1 12.3902 9.63565 4.88402 20.0854 4.88402C29.4479 4.88402 37.7237 10.892 39.3547 18.9614C38.7507 18.899 38.1446 18.8358 37.5406 18.8358C28.4803 18.9609 21.3561 26.0299 21.3561 34.7876C21.3561 36.2265 21.5956 37.6018 21.958 38.9779C21.354 39.0407 20.6894 39.1034 20.0854 39.1034ZM13.8824 13.4378C12.4328 13.4378 10.9816 14.4386 10.9816 15.9399C10.9816 17.4411 12.4328 18.4419 13.8824 18.4419C15.3324 18.4419 16.2984 17.4411 16.2984 15.9399C16.2984 14.3763 15.3324 13.4378 13.8824 13.4378ZM27.2915 18.4419C28.7415 18.4419 29.7075 17.3788 29.7075 15.9399C29.7075 14.4386 28.7415 13.4378 27.2915 13.4378C25.8419 13.4378 24.3907 14.4386 24.3907 15.9399C24.3907 17.4411 25.8419 18.4419 27.2915 18.4419ZM48.354 46.1075L49.8044 51.116L44.6088 48.1117C42.7362 48.6122 40.803 49.1117 38.8698 49.1117C29.7493 49.1117 22.5641 42.6063 22.5641 34.5365C22.5039 26.5295 29.6878 20.0219 38.7486 20.0219C47.3859 20.0219 55 26.594 55 34.6015C55 39.1052 52.1593 43.105 48.354 46.1075ZM33.5112 27.9524C32.5448 27.9524 31.6386 29.0142 31.6386 29.9523C31.6386 31.0159 32.605 31.9522 33.5112 31.9522C34.9608 31.9522 35.9272 30.9531 35.9272 29.9523C35.9272 28.9515 34.9608 27.9524 33.5112 27.9524ZM44.0236 27.9524C43.0572 27.9524 42.151 29.0142 42.151 29.9523C42.151 31.0159 43.1178 31.9522 44.0236 31.9522C45.4732 31.9522 46.4396 30.9531 46.4396 29.9523C46.4396 28.9515 45.4732 27.9524 44.0236 27.9524Z" fill-rule="evenodd"  fill="#FFFFFF" ></path></svg>',
  }

  private unlisten: UnlistenFn | null = null
  private unlistenQrCodeStatus: UnlistenFn | null = null
  private unlistenQrCode: UnlistenFn | null = null
  private contextTokenMap = new Map<string, string>()

  async start(deps: DispatcherDeps): Promise<void> {
    this.unlisten = await listen<WeixinInboundEvent>('weixin-message', async (event) => {
      const payload = event.payload
      const text = payload.text?.trim()
      if (!text) return

      if (payload.contextToken) {
        this.contextTokenMap.set(payload.accountId, payload.contextToken)
      }

      const store = useConversationStore()

      if (text === '/new' || text === '/reset' || text === '新对话') {
        store.createNewExternalConversation(
          'weixin',
          payload.fromUserId,
          payload.accountId,
          '微信对话',
        )
        const ctx = this.contextTokenMap.get(payload.accountId) || ''
        try {
          await invoke('weixin_reply_message', {
            accountId: payload.accountId,
            toUserId: payload.fromUserId,
            text: '好的，已开启新对话 ✨',
            contextToken: ctx,
          })
        } catch (err) {
          console.error('[WeixinBridge] Reply /new ack failed:', err)
        }
        return
      }

      const conv = store.findOrCreateByExternalChatId(
        'weixin',
        payload.fromUserId,
        payload.accountId,
        '微信对话',
      )

      store.addMessage(conv.id, 'user', [createTextContent(text)])

      let replyBuffer = ''
      const accountId = payload.accountId
      const fromUserId = payload.fromUserId

      const callbacks: DispatcherCallbacks = {
        onChunk: (chunk) => {
          replyBuffer += chunk
        },
        onReasoningChunk: () => {},
        onToolExecuting: () => {},
        onToolResult: () => {},
        onTurnStart: () => {},
        onTurnEnd: async (_messageId) => {
          if (replyBuffer.trim()) {
            try {
              await this.sendReply(accountId, fromUserId, replyBuffer, {
                lastMessageId: '',
              })
              store.addMessage(conv.id, 'assistant', [createTextContent(replyBuffer)])
              updateConversationMeta(conv)
            } catch (err) {
              console.error('[WeixinBridge] Reply failed:', err)
            }
            replyBuffer = ''
          }
        },
        onApproveAccess: async () => true,
        onMemoryRecallStart: () => {},
        onMemoryRecallComplete: () => {},
        onMemoryRecallError: () => {},
        onConsolidationComplete: (convId) => {
          store.markMessagesConsolidated(convId)
        },
        onContextCompressed: () => {},
      }

      const weixinDeps: DispatcherDeps = {
        ...deps,
        getActiveConversation: () => ({
          id: conv.id,
          title: conv.title,
          messages: conv.messages,
          summary: conv.summary,
          summaryUpdatedAt: conv.summaryUpdatedAt,
        }),
        generateTitleIfNeeded: () => {},
        getConversationMessages: () => store.getSimpleMessages(conv.id) as Array<{ role: 'user' | 'assistant'; content: string; reasoning_content?: string }>,
      }

      try {
        await dispatchMessage(text, weixinDeps, callbacks)
      } catch (err) {
        console.error('[WeixinBridge] dispatchMessage failed:', err)
        try {
          const ctx = this.contextTokenMap.get(accountId) || ''
          await invoke('weixin_reply_message', {
            accountId,
            toUserId: fromUserId,
            text: `抱歉，处理消息时出错: ${err instanceof Error ? err.message : '未知错误'}`,
            contextToken: ctx,
          })
        } catch (_) {}
      }
    })

    this.unlistenQrCodeStatus = await listen<WeixinLoginResultEvent>('weixin-login-status', (event) => {
      console.log('[WeixinBridge] Login status event:', JSON.stringify(event.payload))
      for (const listener of qrCodeStatusListeners) {
        listener(event.payload)
      }
    })

    this.unlistenQrCode = await listen<WeixinQrCodeEvent>('weixin-qrcode', (event) => {
      const listener = qrCodeUrlListeners.get(event.payload.accountId)
      if (listener) listener(event.payload.qrcodeUrl)
    })
  }

  stop(): void {
    if (this.unlisten) { this.unlisten(); this.unlisten = null }
    if (this.unlistenQrCodeStatus) { this.unlistenQrCodeStatus(); this.unlistenQrCodeStatus = null }
    if (this.unlistenQrCode) { this.unlistenQrCode(); this.unlistenQrCode = null }
    this.contextTokenMap.clear()
  }

  async connect(accountId: string, _config: ChannelAccountConfig): Promise<void> {
    await invoke('weixin_connect', { accountId })
  }

  async disconnect(accountId: string): Promise<void> {
    await invoke('weixin_disconnect', { accountId })
  }

  async getStatus(accountId: string): Promise<string> {
    return invoke<string>('weixin_get_status', { accountId })
  }

  async onStatusChange(callback: StatusChangeCallback): Promise<UnlistenFn> {
    return listen<{ accountId: string; status: string; error?: string }>('weixin-status', (event) => {
      callback(event.payload.accountId, event.payload.status, event.payload.error)
    })
  }

  async sendReply(accountId: string, externalChatId: string, messageText: string, _replyContext: ReplyContext): Promise<void> {
    const ctx = this.contextTokenMap.get(accountId) || ''
    await invoke('weixin_reply_message', {
      accountId,
      toUserId: externalChatId,
      text: messageText,
      contextToken: ctx,
    })
  }

  getConfigFields(): ConfigFieldDefinition[] {
    return [
      { key: '_qrcode', label: '微信扫码登录', type: 'qrcode', required: false },
    ]
  }

  validateConfig(_config: Record<string, unknown>): { valid: boolean; errors?: Record<string, string> } {
    return { valid: true }
  }
}
