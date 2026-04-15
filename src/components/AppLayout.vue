<template>
  <div class="app-layout">
    <NavigationSidebar
      :activeNav="activeNav"
      @update:activeNav="handleNavChange"
    />
    <SubNavigation
      :activeNav="activeNav"
      :activeSubNav="activeSubNav"
      @update:activeSubNav="activeSubNav = $event"
    />
    <div class="main-content">
      <div class="main-header">
        <div class="header-title">{{ getHeaderTitle() }}</div>
        <div class="window-controls">
          <button class="window-btn" @click="handleMinimize" title="最小化">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
          </button>
          <button class="window-btn" @click="handleMaximize" title="最大化">
            <svg v-if="!isMaximized" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="4" y="4" width="16" height="16" rx="2"></rect>
            </svg>
            <svg v-else viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="6" y="6" width="12" height="12" rx="1"></rect>
              <path d="M8 6V5a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1h-1"></path>
            </svg>
          </button>
          <button class="window-btn close-btn" @click="handleClose" title="关闭">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="6" y1="6" x2="18" y2="18"></line>
              <line x1="6" y1="18" x2="18" y2="6"></line>
            </svg>
          </button>
        </div>
      </div>
      <template v-if="activeNav === 'chat' && activeSubNav === 'chat-list'">
        <ChatPageContent
          @openSettings="handleOpenSettings"
        />
      </template>
      <template v-else-if="activeNav === 'settings' && activeSubNav === 'models'">
        <SettingsPage @back="handleBackFromSettings" />
      </template>
      <template v-else-if="activeNav === 'agent' && activeSubNav === 'tools'">
        <AgentToolsPage @back="handleBackFromTools" />
      </template>
      <template v-else-if="activeNav === 'agent' && activeSubNav === 'skills'">
        <SkillsPage @back="handleBackFromTools" />
      </template>
      <template v-else-if="activeNav === 'agent' && activeSubNav === 'role'">
        <RolePage @back="handleBackFromTools" />
      </template>
      <template v-else-if="activeNav === 'control' && activeSubNav === 'channels'">
        <ChannelsPage @back="handleNavChange('control')" />
      </template>
      <template v-else-if="activeNav === 'control' && activeSubNav === 'security-rules'">
        <SecurityRulesPage @back="handleNavChange('control')" />
      </template>
      <template v-else-if="activeNav === 'agent' && activeSubNav === 'memory'">
        <AgentMemoryPage @back="handleNavChange('agent')" />
      </template>
      <template v-else-if="activeNav === 'settings' && activeSubNav === 'personalization'">
        <PersonalizationPage @back="handleBackFromSettings" />
      </template>
      <template v-else-if="activeNav === 'settings' && activeSubNav === 'local-models'">
        <LocalModelsPage @back="handleBackFromSettings" />
      </template>
      <template v-else-if="activeNav === 'settings' && activeSubNav === 'about'">
        <AboutPage @back="handleBackFromSettings" />
      </template>
      <template v-else-if="activeNav === 'settings' && activeSubNav === 'misc'">
        <MiscSettingsPage @back="handleBackFromSettings" />
      </template>
      <template v-else>
        <div class="placeholder-wrapper">
          <div class="placeholder-content">
            <div class="placeholder-icon">
              <svg viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1.5">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <line x1="9" y1="9" x2="15" y2="15"></line>
                <line x1="15" y1="9" x2="9" y2="15"></line>
              </svg>
            </div>
            <p>{{ getPlaceholderText() }}</p>
            <span>功能开发中，敬请期待...</span>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, defineAsyncComponent, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import NavigationSidebar from './NavigationSidebar.vue'
import SubNavigation from './SubNavigation.vue'
import ChatPageContent from './ChatPageContent.vue'
import { initThemeSettings } from '../composables/useTheme'
import { channelRegistry } from '../agent/channelRegistry'
import { fairyDo } from '../agent/fairyDo'
import { fbmStore } from '../stores/fbmStore'
import { setCurrentUserMessage, setRecentUserMessages, setConversationContext } from '../agent/virtualHandlers'
const SettingsPage = defineAsyncComponent(() => import('./SettingsPage.vue'))
const PersonalizationPage = defineAsyncComponent(() => import('./PersonalizationPage.vue'))
const AgentToolsPage = defineAsyncComponent(() => import('./AgentToolsPage.vue'))
const SecurityRulesPage = defineAsyncComponent(() => import('./SecurityRulesPage.vue'))
const LocalModelsPage = defineAsyncComponent(() => import('./LocalModelsPage.vue'))
const AboutPage = defineAsyncComponent(() => import('./AboutPage.vue'))
const MiscSettingsPage = defineAsyncComponent(() => import('./MiscSettingsPage.vue'))
const AgentMemoryPage = defineAsyncComponent(() => import('./AgentMemoryPage.vue'))
const RolePage = defineAsyncComponent(() => import('./RolePage.vue'))
const SkillsPage = defineAsyncComponent(() => import('./SkillsPage.vue'))
const ChannelsPage = defineAsyncComponent(() => import('./ChannelsPage.vue'))

const activeNav = ref('chat')
const activeSubNav = ref('chat-list')

const isMaximized = ref(false)
const appWindow = getCurrentWindow()

const checkMaximized = async () => {
  isMaximized.value = await appWindow.isMaximized()
}

onMounted(() => {
  initThemeSettings()
  channelRegistry.startAll({
    getActiveConversation: () => null,
    addMessage: () => {},
    getConversationMessages: () => [],
    generateTitleIfNeeded: () => {},
    fbmStore,
    fairyDo,
    setCurrentUserMessage,
    setRecentUserMessages,
    setConversationContext,
    setCurrentTools: () => {},
    setCurrentProviderId: () => {},
    saveConversationSummary: () => {},
  })
})

onUnmounted(() => {
  channelRegistry.stopAll()
})

checkMaximized()

const handleMinimize = async () => {
  await appWindow.minimize()
}

const handleMaximize = async () => {
  if (isMaximized.value) {
    await appWindow.unmaximize()
  } else {
    await appWindow.maximize()
  }
  isMaximized.value = !isMaximized.value
}

const handleClose = async () => {
  await appWindow.close()
}

const getHeaderTitle = (): string => {
  const titleMap: Record<string, Record<string, string>> = {
    chat: { 'chat-list': '对话' },
    control: { channels: '渠道管理', 'scheduled-tasks': '定时任务', life: '生命管理', 'security-rules': '安全规则' },
    agent: { workspace: '工作区', skills: '技能管理', role: '角色', tools: '工具管理', memory: '记忆' },
    settings: { models: '模型设置', 'local-models': '本地模型', personalization: '个性化', misc: '杂项', about: '关于' }
  }
  return titleMap[activeNav.value]?.[activeSubNav.value] || ''
}

const handleNavChange = (nav: string) => {
  activeNav.value = nav
  const subNavMap: Record<string, string> = {
    chat: 'chat-list',
    control: 'channels',
    agent: 'workspace',
    settings: 'models'
  }
  activeSubNav.value = subNavMap[nav] || 'chat-list'
}

const handleOpenSettings = () => {
  activeNav.value = 'settings'
  activeSubNav.value = 'models'
}

const handleBackFromSettings = () => {
  activeNav.value = 'chat'
  activeSubNav.value = 'chat-list'
}

const handleBackFromTools = () => {
  activeNav.value = 'agent'
  activeSubNav.value = 'workspace'
}

const getPlaceholderText = (): string => {
  const textMap: Record<string, string> = {
    channels: '渠道管理',
    'scheduled-tasks': '定时任务',
    life: '生命管理',
    workspace: '工作区',
    skills: '技能管理',
    touchbridge: '触桥管理',
    models: '模型设置',
    personalization: '个性化'
  }
  return textMap[activeSubNav.value] || '功能'
}
</script>

<style scoped>
.app-layout {
  display: flex;
  width: 100%;
  height: 100vh;
  background: var(--color-surface);
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.main-header {
  height: 60px;
  padding: 0 28px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--color-surface-card);
  border-bottom: 1px solid var(--color-border);
  -webkit-app-region: drag;
  flex-shrink: 0;
}

.header-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--color-primary);
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 8px;
  -webkit-app-region: no-drag;
}

.window-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  transition: all 0.2s;
}

.window-btn:hover {
  background: var(--color-surface-secondary);
  color: var(--color-primary);
}

.window-btn.close-btn:hover {
  background: var(--color-accent-error);
  color: var(--color-text-inverse);
}

.placeholder-wrapper {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface);
}

.placeholder-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
}

.placeholder-icon {
  margin-bottom: 20px;
  opacity: 0.4;
  color: var(--color-text-secondary);
}

.placeholder-content p {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.placeholder-content span {
  font-size: 14px;
}
</style>
