<template>
  <div class="sub-nav">
    <div class="sub-nav-header">
      <h3>{{ currentGroup?.label }}</h3>
    </div>
    <div class="sub-nav-list">
      <div
        v-for="item in currentGroup?.children"
        :key="item.id"
        class="sub-nav-item"
        :class="{ active: activeSubNav === item.id }"
        @click="$emit('update:activeSubNav', item.id)"
      >
        <span class="sub-nav-label">{{ item.label }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface NavGroup {
  id: string
  label: string
  children: { id: string; label: string }[]
}

const props = defineProps<{
  activeNav: string
  activeSubNav: string
}>()

defineEmits<{
  'update:activeSubNav': [id: string]
}>()

const navGroups: NavGroup[] = [
  {
    id: 'chat',
    label: '聊天',
    children: [
      { id: 'chat-list', label: '聊天' }
    ]
  },
  {
    id: 'control',
    label: '控制',
    children: [
      { id: 'channels', label: '渠道' },
      { id: 'scheduled-tasks', label: '定时任务' },
      { id: 'life', label: '生命' },
      { id: 'security-rules', label: '安全规则' }
    ]
  },
  {
    id: 'agent',
    label: '智能体',
    children: [
      { id: 'workspace', label: '工作区' },
      { id: 'role', label: '角色' },
      { id: 'skills', label: '技能' },
      { id: 'tools', label: '工具' },
      { id: 'memory', label: '记忆' },
      { id: 'touchbridge', label: '触桥' }
    ]
  },
  {
    id: 'settings',
    label: '设置',
    children: [
      { id: 'models', label: '模型' },
      { id: 'local-models', label: '本地模型' },
      { id: 'personalization', label: '个性化' },
      { id: 'misc', label: '杂项' },
      { id: 'about', label: '关于' }
    ]
  }
]

const currentGroup = computed(() => {
  return navGroups.find(g => g.id === props.activeNav)
})
</script>

<style scoped>
.sub-nav {
  width: 180px;
  min-width: 180px;
  height: 100%;
  background: var(--color-surface-card);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
}

.sub-nav-header {
  padding: 0 20px;
  height: 60px;
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--color-border);
}

.sub-nav-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.sub-nav-list {
  flex: 1;
  padding: 12px 8px;
  overflow-y: auto;
}

.sub-nav-item {
  padding: 12px 16px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  margin-bottom: 2px;
}

.sub-nav-item:hover {
  background: var(--color-primary-alpha-10);
}

.sub-nav-item.active:hover {
  background: var(--color-primary-alpha-12);
}

.sub-nav-item.active {
  background: var(--color-primary-alpha-12);
}

.sub-nav-item.active .sub-nav-label {
  color: var(--color-primary);
  font-weight: 700;
}

.sub-nav-label {
  font-size: 13px;
  color: var(--color-text-secondary);
  font-weight: 600;
}
</style>
