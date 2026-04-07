import type { ChannelAccount, ChannelId } from '../types/channel'

export interface ChannelSettings {
  accounts: ChannelAccount[]
}

const STORAGE_KEY = 'aurorafairy-channels'

const defaultSettings: ChannelSettings = {
  accounts: [],
}

export function loadChannelSettings(): ChannelSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { ...defaultSettings }
    const parsed = JSON.parse(raw)
    return { ...defaultSettings, ...parsed }
  } catch {
    return { ...defaultSettings }
  }
}

export function saveChannelSettings(settings: Partial<ChannelSettings>): void {
  const current = loadChannelSettings()
  const merged = { ...current, ...settings }
  localStorage.setItem(STORAGE_KEY, JSON.stringify(merged))
}

export function getChannelAccounts(): ChannelAccount[] {
  return loadChannelSettings().accounts
}

export function addChannelAccount(account: ChannelAccount): void {
  const settings = loadChannelSettings()
  settings.accounts.push(account)
  saveChannelSettings(settings)
}

export function updateChannelAccount(accountId: string, updates: Partial<ChannelAccount>): void {
  const settings = loadChannelSettings()
  const index = settings.accounts.findIndex(a => a.id === accountId)
  if (index !== -1) {
    settings.accounts[index] = { ...settings.accounts[index], ...updates }
    saveChannelSettings(settings)
  }
}

export function deleteChannelAccount(accountId: string): void {
  const settings = loadChannelSettings()
  settings.accounts = settings.accounts.filter(a => a.id !== accountId)
  saveChannelSettings(settings)
}

export function generateAccountId(channelId: ChannelId): string {
  return `${channelId}-${Date.now()}-${Math.random().toString(36).substring(2, 8)}`
}
