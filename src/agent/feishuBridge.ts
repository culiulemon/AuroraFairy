import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { dispatchMessage, type DispatcherCallbacks, type DispatcherDeps } from './messageDispatcher'
import { useConversationStore } from '../stores/useConversationStore'
import { createTextContent, updateConversationMeta } from '../stores/conversation'
import type { ChannelBridge, StatusChangeCallback, ReplyContext } from './channelBridge'
import type { ConfigFieldDefinition, ChannelAccountConfig } from '../types/channel'

interface FeishuInboundEvent {
  accountId: string
  eventType: string
  event: {
    sender?: {
      sender_id?: { open_id?: string; user_id?: string; union_id?: string }
      sender_type?: string
      tenant_key?: string
    }
    message?: {
      message_id: string
      chat_id: string
      chat_type: string
      message_type: string
      content: string
      create_time?: string
    }
  }
}

interface FeishuSession {
  accountConfig: {
    appId: string
    appSecret: string
    domain: string
  }
  lastMessageId: string
}

function extractTextContent(messageType: string, contentStr: string): string {
  try {
    const parsed = JSON.parse(contentStr)
    if (messageType === 'text') {
      return parsed.text || ''
    }
    if (messageType === 'post') {
      const localeContent = parsed.zh_cn || parsed.en_us || parsed
      if (localeContent && localeContent.content) {
        return localeContent.content
          .flat()
          .map((node: any) => {
            if (node.tag === 'text' || node.tag === 'md') return node.text || ''
            return ''
          })
          .join('')
      }
    }
    return contentStr
  } catch {
    return contentStr
  }
}

export class FeishuChannelBridge implements ChannelBridge {
  readonly channelId = 'feishu'
  readonly displayName = '飞书'
  readonly badgeIcon = {
    backgroundColor: 'transparent',
    svgContent: '<svg viewBox="0 0 56 56" fill="none"><path d="M29.0655 29.0669C29.1232 29.0093 29.1809 28.9515 29.1809 28.9515C29.2962 28.8362 29.3539 28.7786 29.4692 28.6632L29.6422 28.4902L30.219 27.9135L30.9686 27.1637L31.6607 26.5293L32.2951 25.8949L32.9295 25.2605L33.5062 24.6838L34.3136 23.8765C34.4867 23.7034 34.6597 23.588 34.775 23.415C35.0633 23.1267 35.4094 22.896 35.6977 22.6077C35.9861 22.377 36.2745 22.1463 36.6205 21.9155C37.0243 21.6272 37.4856 21.3389 37.947 21.0505C38.4083 20.7621 38.8697 20.5315 39.331 20.3008C39.7924 20.0701 40.1961 19.8971 40.6576 19.724C40.8883 19.6088 41.1766 19.551 41.4073 19.4357C41.5226 19.378 41.6956 19.3203 41.811 19.3203C40.6576 14.7066 38.466 10.4389 35.5824 6.86326C35.0057 6.17115 34.1406 5.76746 33.2179 5.76746L9.16878 5.76746C8.70745 5.76746 8.53438 6.34413 8.88044 6.57484C17.3004 12.4574 24.1057 20.1854 29.0655 29.0669C29.0655 29.0669 29.0655 29.1245 29.0655 29.0669Z" fill="#00D6B9"/><path d="M19.5507 50.2325C31.9501 50.2325 42.7924 43.3696 48.4442 33.2769C48.6173 32.9311 48.848 32.5849 49.021 32.1813C48.7327 32.758 48.4442 33.2769 48.0983 33.7383C47.9829 33.9114 47.8676 34.0845 47.6946 34.2574C47.5215 34.4882 47.3485 34.6612 47.1755 34.8918C47.0602 35.0649 46.8871 35.1801 46.7718 35.3532C46.4834 35.6415 46.1951 35.93 45.849 36.2183C45.676 36.3914 45.5031 36.5066 45.33 36.6797C45.0993 36.8526 44.9263 37.0256 44.6956 37.141C44.5802 37.2564 44.4072 37.314 44.2919 37.4295C44.1189 37.5448 44.0036 37.6024 43.8306 37.7177C43.5421 37.8907 43.1962 38.0638 42.8501 38.179C42.5618 38.2944 42.2734 38.4098 41.985 38.5251C41.6967 38.6405 41.3506 38.7558 41.0047 38.8135C40.5432 38.9288 40.0242 39.0441 39.5052 39.1019C39.1591 39.1595 38.7554 39.2171 38.4094 39.2171C38.0057 39.2171 37.6597 39.2749 37.2559 39.2749C36.8522 39.2749 36.3909 39.2171 35.9295 39.2171C35.5834 39.1595 35.2951 39.1595 34.9491 39.1019C34.6607 39.0441 34.3724 38.9866 34.084 38.9288C33.9109 38.8712 33.7957 38.8712 33.6226 38.8135C33.2189 38.6982 32.8152 38.5829 32.4115 38.4675C32.1808 38.4098 32.0079 38.3521 31.7771 38.2944C31.4888 38.179 31.1427 38.1215 30.8544 38.0061C30.6237 37.9484 30.3354 37.8332 30.1047 37.7754C29.874 37.7177 29.6433 37.6024 29.4126 37.5447C29.2396 37.487 29.0666 37.4295 28.9512 37.3717C28.7782 37.314 28.5475 37.2564 28.3745 37.141C28.2592 37.0833 28.0861 37.0256 27.9708 36.9681C27.6824 36.8526 27.4517 36.795 27.1634 36.6797C26.9904 36.6219 26.8751 36.5643 26.702 36.5066C26.4714 36.4489 26.2983 36.3336 26.0677 36.2759C25.837 36.1606 25.6063 36.103 25.3756 35.9875C25.2602 35.93 25.0872 35.8722 24.9719 35.8146C24.7989 35.7569 24.6258 35.6415 24.4528 35.5838C24.3375 35.5263 24.1645 35.4685 24.0491 35.4108C23.9338 35.3532 23.7608 35.2955 23.6454 35.2379C23.5301 35.1801 23.4147 35.1225 23.2994 35.0649C23.184 35.0071 23.0687 34.9495 22.9533 34.8918C22.838 34.8341 22.7227 34.7765 22.6074 34.7188C22.492 34.6612 22.3767 34.6035 22.2613 34.5457C22.0883 34.4882 21.973 34.4304 21.7999 34.315C21.6269 34.2574 21.5116 34.142 21.3386 34.0845C21.1655 34.0267 20.9925 33.9114 20.8195 33.8537C20.7042 33.796 20.5312 33.7383 20.4158 33.6231C12.9762 29.932 6.40159 24.9723 0.749731 18.9744C0.461375 18.686 0 18.859 0 19.2627L0 40.4859L0 42.2161C0 43.1966 0.519043 44.177 1.32645 44.696C6.51694 48.214 12.8031 50.2325 19.5507 50.2325Z" fill="#3370FF"/><path d="M48.4458 33.2195C48.3882 33.2771 48.3882 33.2771 48.4458 33.2195L48.5612 32.9888C48.5035 33.1041 48.4458 33.1618 48.4458 33.2195Z" fill="#133C9A"/><path d="M48.9634 32.2386L48.9634 32.2386C48.9634 32.2386 48.9634 32.2386 48.9634 32.2386Z" fill="#133C9A"/><path d="M56.0001 20.3008C53.4626 19.0897 50.6367 18.34 47.6377 18.34C45.8498 18.34 44.1197 18.5707 42.5049 19.032C42.3319 19.0897 42.1012 19.1474 41.9282 19.2051C41.8128 19.2627 41.6399 19.2627 41.5245 19.3204C41.2362 19.4358 41.0055 19.4934 40.7748 19.6088C40.3133 19.7817 39.852 20.0125 39.4483 20.1854C38.9869 20.4161 38.5255 20.6468 38.0642 20.9352C37.6028 21.2235 37.1415 21.512 36.7378 21.8003C36.4493 22.031 36.1034 22.2617 35.8149 22.4923C35.469 22.723 35.1805 23.0114 34.8922 23.2998C34.7192 23.4728 34.5462 23.5881 34.4309 23.7611L33.6234 24.5685L33.0467 25.1453L32.4123 25.7797L31.778 26.4141L31.0859 27.0485L30.2785 27.9135L29.7018 28.4902L29.5288 28.6632C29.4134 28.7786 29.3557 28.8363 29.2404 28.9516C29.1827 29.0093 29.125 29.0669 29.125 29.0669C29.0674 29.1246 29.0098 29.1823 28.8944 29.24C28.8367 29.2976 28.7213 29.413 28.6637 29.4707C26.5875 31.3738 24.2806 32.9887 21.8008 34.3151C21.9738 34.3728 22.0891 34.4304 22.2621 34.5458C22.3775 34.6034 22.4928 34.6611 22.6082 34.7188C22.7235 34.7765 22.8389 34.8341 22.9542 34.8918C23.0695 34.9495 23.1848 35.0072 23.3002 35.0648C23.4155 35.1225 23.5309 35.1802 23.6462 35.2378C23.7616 35.2955 23.9346 35.3532 24.0499 35.4109C24.1653 35.4685 24.3383 35.5262 24.4536 35.5839C24.6267 35.6415 24.7997 35.7569 24.9727 35.8146C25.088 35.8722 25.261 35.9299 25.3764 35.9876C25.6071 36.1029 25.8378 36.1606 26.0685 36.2759C26.2991 36.3336 26.4722 36.449 26.7029 36.5066C26.8759 36.5643 26.9912 36.622 27.1642 36.6797C27.4525 36.7949 27.6832 36.8527 27.9716 36.968C28.0869 37.0257 28.26 37.0834 28.3754 37.141C28.5483 37.1987 28.779 37.2564 28.952 37.3717C29.125 37.4293 29.2981 37.4871 29.4134 37.5447C29.6441 37.6024 29.8748 37.7178 30.1055 37.7754C30.3362 37.8331 30.6245 37.9484 30.8552 38.0061C31.1436 38.1215 31.4896 38.1791 31.778 38.2944C32.0087 38.3522 32.1816 38.4098 32.4123 38.4674C32.816 38.5828 33.2197 38.6981 33.6234 38.8135C33.7965 38.8712 33.9118 38.8712 34.0848 38.9288C34.3732 38.9865 34.6615 39.0442 34.9499 39.1018C35.2959 39.1596 35.5843 39.2172 35.9303 39.2172C36.3917 39.2749 36.7954 39.2749 37.2568 39.2749C37.6605 39.2749 38.0642 39.2749 38.4102 39.2172C38.7562 39.2172 39.1599 39.1596 39.506 39.1018C40.025 39.0442 40.4864 38.9288 41.0055 38.8135C41.3514 38.6981 41.6399 38.6405 41.9858 38.5252C42.2743 38.4098 42.5626 38.2944 42.8509 38.1791C43.197 38.0061 43.4853 37.8908 43.8314 37.7178C44.0044 37.66 44.1197 37.5447 44.2927 37.4293C44.4081 37.314 44.5811 37.2564 44.6964 37.141C44.9271 36.968 45.1578 36.8527 45.3308 36.6797C45.5039 36.5643 45.6768 36.3913 45.8498 36.2183C46.1383 35.9299 46.4842 35.6415 46.7727 35.3532C46.8879 35.1802 47.061 35.0648 47.1763 34.8918C47.3493 34.7188 47.5223 34.4882 47.6954 34.2574C47.8107 34.0844 47.9261 33.9114 48.0991 33.7384C48.4451 33.277 48.7335 32.7579 49.0218 32.1813L49.3679 31.5469L52.2514 25.7797C53.1165 23.7035 54.4429 21.8579 56.0001 20.3008Z" fill="#133C9A"/></svg>',
  }

  private sessions = new Map<string, FeishuSession>()
  private unlisten: UnlistenFn | null = null

  private getSessionKey(accountId: string, chatId: string): string {
    return `feishu:${accountId}:${chatId}`
  }

  findSessionByExternalChatId(externalChatId: string): FeishuSession | null {
    for (const [key, session] of this.sessions) {
      if (key.endsWith(`:${externalChatId}`)) {
        return session
      }
    }
    return null
  }

  async start(deps: DispatcherDeps): Promise<void> {
    this.unlisten = await listen<FeishuInboundEvent>('feishu-message', async (event) => {
      const payload = event.payload
      if (payload.eventType !== 'im.message.receive_v1') return

      const message = payload.event.message
      const sender = payload.event.sender
      if (!message) return

      if (sender?.sender_type === 'app') return

      const text = extractTextContent(message.message_type, message.content)
      if (!text.trim()) return

      const sessionKey = this.getSessionKey(payload.accountId, message.chat_id)
      let session = this.sessions.get(sessionKey)
      if (!session) {
        const { loadChannelSettings } = await import('../stores/channelStore')
        const settings = loadChannelSettings()
        const account = settings.accounts.find(a => a.id === payload.accountId)
        const config = (account?.config || {}) as any
        session = {
          accountConfig: {
            appId: config.appId || '',
            appSecret: config.appSecret || '',
            domain: config.domain || 'feishu',
          },
          lastMessageId: message.message_id,
        }
        this.sessions.set(sessionKey, session)
      }
      session.lastMessageId = message.message_id

      const store = useConversationStore()

      if (text === '/new' || text === '/reset' || text === '新对话') {
        store.createNewExternalConversation(
          'feishu',
          message.chat_id,
          payload.accountId,
          '飞书对话',
        )
        try {
          await invoke('feishu_reply_message', {
            accountId: payload.accountId,
            domain: session.accountConfig.domain,
            appId: session.accountConfig.appId,
            appSecret: session.accountConfig.appSecret,
            messageId: message.message_id,
            text: '好的，已开启新对话 ✨',
          })
        } catch (err) {
          console.error('[FeishuBridge] Reply /new ack failed:', err)
        }
        return
      }

      const conv = store.findOrCreateByExternalChatId(
        'feishu',
        message.chat_id,
        payload.accountId,
        '飞书对话',
      )

      store.addMessage(conv.id, 'user', [createTextContent(text)])

      let replyBuffer = ''

      const sessionRef = session
      const msgId = message.message_id
      const accountId = payload.accountId

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
              await this.sendReply(accountId, message.chat_id, replyBuffer, {
                lastMessageId: msgId,
              })
              store.addMessage(conv.id, 'assistant', [createTextContent(replyBuffer)])
              updateConversationMeta(conv)
            } catch (err) {
              console.error('[FeishuBridge] Reply failed:', err)
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

      const feishuDeps: DispatcherDeps = {
        ...deps,
        getActiveConversation: () => ({
          id: conv.id,
          title: conv.title,
          messages: conv.messages,
          summary: conv.summary,
          summaryUpdatedAt: conv.summaryUpdatedAt,
        }),
        generateTitleIfNeeded: () => {},
        getConversationMessages: () => store.getSimpleMessages(conv.id) as Array<{ role: 'user' | 'assistant'; content: string }>,
      }

      try {
        await dispatchMessage(text, feishuDeps, callbacks)
      } catch (err) {
        console.error('[FeishuBridge] dispatchMessage failed:', err)
        try {
          await invoke('feishu_reply_message', {
            accountId: payload.accountId,
            domain: sessionRef.accountConfig.domain,
            appId: sessionRef.accountConfig.appId,
            appSecret: sessionRef.accountConfig.appSecret,
            messageId: message.message_id,
            text: `抱歉，处理消息时出错: ${err instanceof Error ? err.message : '未知错误'}`,
          })
        } catch (_) {}
      }
    })
  }

  stop(): void {
    if (this.unlisten) {
      this.unlisten()
      this.unlisten = null
    }
    this.sessions.clear()
  }

  async connect(accountId: string, config: ChannelAccountConfig): Promise<void> {
    const cfg = config as any
    await invoke('feishu_connect', {
      accountId,
      appId: cfg.appId,
      appSecret: cfg.appSecret,
      domain: cfg.domain || 'feishu',
    })
  }

  async disconnect(accountId: string): Promise<void> {
    await invoke('feishu_disconnect', { accountId })
  }

  async getStatus(accountId: string): Promise<string> {
    return invoke<string>('feishu_get_status', { accountId })
  }

  async onStatusChange(callback: StatusChangeCallback): Promise<UnlistenFn> {
    return listen<{ accountId: string; status: string; error?: string }>('feishu-status', (event) => {
      callback(event.payload.accountId, event.payload.status, event.payload.error)
    })
  }

  async sendReply(accountId: string, externalChatId: string, messageText: string, _replyContext: ReplyContext): Promise<void> {
    const session = this.findSessionByExternalChatId(externalChatId)
    if (!session) {
      console.warn(`[FeishuBridge] No session found for externalChatId: ${externalChatId}`)
      return
    }
    await invoke('feishu_reply_message', {
      accountId,
      domain: session.accountConfig.domain,
      appId: session.accountConfig.appId,
      appSecret: session.accountConfig.appSecret,
      messageId: session.lastMessageId,
      text: messageText,
    })
  }

  getConfigFields(): ConfigFieldDefinition[] {
    return [
      { key: 'appId', label: 'App ID', type: 'text', placeholder: 'cli_xxxxxxxxxxxx', required: true },
      { key: 'appSecret', label: 'App Secret', type: 'password', placeholder: '输入 App Secret', required: true },
      {
        key: 'connectionMode',
        label: '连接模式',
        type: 'select',
        required: true,
        defaultValue: 'websocket',
        options: [
          { value: 'websocket', label: 'WebSocket' },
          { value: 'webhook', label: 'Webhook' },
        ],
      },
      {
        key: 'domain',
        label: '域名',
        type: 'select',
        required: true,
        defaultValue: 'feishu',
        options: [
          { value: 'feishu', label: 'feishu（国内）' },
          { value: 'lark', label: 'lark（国际）' },
        ],
      },
      { key: 'webhookPort', label: 'Webhook 端口', type: 'number', placeholder: '例如: 8080', condition: { field: 'connectionMode', value: 'webhook' } },
      { key: 'webhookPath', label: 'Webhook 路径', type: 'text', placeholder: '例如: /webhook/feishu', condition: { field: 'connectionMode', value: 'webhook' } },
      { key: 'verificationToken', label: '验证 Token（可选）', type: 'text', placeholder: '输入验证 Token' },
      { key: 'encryptKey', label: '加密密钥（可选）', type: 'text', placeholder: '输入加密密钥' },
    ]
  }

  validateConfig(config: Record<string, unknown>): { valid: boolean; errors?: Record<string, string> } {
    const errors: Record<string, string> = {}
    if (!config.appId || !(config.appId as string).trim()) {
      errors.appId = 'App ID 不能为空'
    }
    if (!config.appSecret || !(config.appSecret as string).trim()) {
      errors.appSecret = 'App Secret 不能为空'
    }
    return { valid: Object.keys(errors).length === 0, errors: Object.keys(errors).length > 0 ? errors : undefined }
  }
}
