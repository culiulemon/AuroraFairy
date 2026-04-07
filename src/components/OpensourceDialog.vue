<template>
  <BaseDialog v-model="showDialog" title="开源鸣谢" width="90%" maxWidth="680px" maxHeight="80vh">
    <div class="opensource-content">
      <p class="opensource-desc">AuroraFairy 基于以下优秀的开源项目构建，感谢所有开源贡献者！</p>

      <div class="category" v-for="cat in categories" :key="cat.name">
        <h4 class="category-title">{{ cat.name }}</h4>
        <div class="lib-list">
          <a
            v-for="lib in cat.libs"
            :key="lib.name"
            :href="lib.url"
            class="lib-item"
            target="_blank"
            rel="noopener noreferrer"
          >
            <div class="lib-info">
              <span class="lib-name">{{ lib.name }}</span>
              <span class="lib-desc">{{ lib.desc }}</span>
            </div>
            <span class="lib-version">{{ lib.version }}</span>
          </a>
        </div>
      </div>
    </div>
  </BaseDialog>
</template>

<script setup lang="ts">
import BaseDialog from './BaseDialog.vue'
import { computed } from 'vue'

interface Library {
  name: string
  desc: string
  version: string
  url: string
}

interface Category {
  name: string
  libs: Library[]
}

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const showDialog = computed({
  get: () => props.modelValue,
  set: (val: boolean) => emit('update:modelValue', val)
})

const categories: Category[] = [
  {
    name: '核心框架',
    libs: [
      { name: 'Vue 3', desc: '渐进式 JavaScript 框架', version: '^3.5', url: 'https://vuejs.org/' },
      { name: 'Tauri', desc: '使用 Web 技术构建桌面应用', version: 'v2', url: 'https://tauri.app/' },
      { name: 'Rust', desc: '高性能系统编程语言', version: '2021', url: 'https://www.rust-lang.org/' },
      { name: 'TypeScript', desc: '带类型的 JavaScript 超集', version: '~5.6', url: 'https://www.typescriptlang.org/' },
      { name: 'Vite', desc: '下一代前端构建工具', version: '^6', url: 'https://vitejs.dev/' },
    ]
  },
  {
    name: '代码编辑',
    libs: [
      { name: 'CodeMirror', desc: '浏览器端的代码编辑器', version: 'v6', url: 'https://codemirror.net/' },
    ]
  },
  {
    name: '前端库',
    libs: [
      { name: 'marked', desc: 'Markdown 解析器', version: '^17', url: 'https://marked.js.org/' },
    ]
  },
  {
    name: 'Tauri 插件',
    libs: [
      { name: 'tauri-plugin-fs', desc: '文件系统访问插件', version: '^2', url: 'https://github.com/tauri-apps/plugins-workspace' },
      { name: 'tauri-plugin-dialog', desc: '原生对话框插件', version: '^2', url: 'https://github.com/tauri-apps/plugins-workspace' },
      { name: 'tauri-plugin-http', desc: 'HTTP 客户端插件', version: '^2', url: 'https://github.com/tauri-apps/plugins-workspace' },
      { name: 'tauri-plugin-notification', desc: '系统通知插件', version: '^2', url: 'https://github.com/tauri-apps/plugins-workspace' },
      { name: 'tauri-plugin-clipboard-manager', desc: '剪贴板管理插件', version: '^2', url: 'https://github.com/tauri-apps/plugins-workspace' },
      { name: 'tauri-plugin-global-shortcut', desc: '全局快捷键插件', version: '^2', url: 'https://github.com/tauri-apps/plugins-workspace' },
      { name: 'tauri-plugin-opener', desc: '打开链接/文件插件', version: '^2', url: 'https://github.com/tauri-apps/plugins-workspace' },
    ]
  },
  {
    name: '外部工具',
    libs: [
      { name: 'FairyAction', desc: '浏览器自动化引擎，为 AI Agent 提供网页浏览与操作能力', version: '-', url: 'https://gitcode.com/Nicek/FairyAction' },
    ]
  },
  {
    name: 'Rust 后端库',
    libs: [
      { name: 'tokio', desc: '异步运行时', version: '1', url: 'https://tokio.rs/' },
      { name: 'serde / serde_json', desc: '序列化/反序列化框架', version: '1', url: 'https://serde.rs/' },
      { name: 'reqwest', desc: 'HTTP 客户端', version: '0.12', url: 'https://docs.rs/reqwest/' },
      { name: 'tokio-tungstenite', desc: 'WebSocket 客户端', version: '0.26', url: 'https://docs.rs/tokio-tungstenite/' },
      { name: 'regex', desc: '正则表达式库', version: '1', url: 'https://docs.rs/regex/' },
      { name: 'walkdir', desc: '递归目录遍历', version: '2', url: 'https://docs.rs/walkdir/' },
      { name: 'notify', desc: '文件系统事件监控', version: '7', url: 'https://docs.rs/notify/' },
      { name: 'uuid', desc: 'UUID 生成器', version: '1', url: 'https://docs.rs/uuid/' },
      { name: 'chrono', desc: '日期时间处理库', version: '0.4', url: 'https://docs.rs/chrono/' },
      { name: 'base64', desc: 'Base64 编解码', version: '0.22', url: 'https://docs.rs/base64/' },
      { name: 'sha2', desc: 'SHA-2 哈希算法', version: '0.10', url: 'https://docs.rs/sha2/' },
      { name: 'futures-util', desc: '异步工具库', version: '0.3', url: 'https://docs.rs/futures-util/' },
      { name: 'encoding_rs', desc: '字符编码转换', version: '0.8', url: 'https://docs.rs/encoding_rs/' },
      { name: 'shellexpand', desc: 'Shell 路径展开', version: '3', url: 'https://docs.rs/shellexpand/' },
      { name: 'opener', desc: '打开路径/URL', version: '0.7', url: 'https://docs.rs/opener/' },
    ]
  }
]
</script>

<style scoped>
.opensource-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.opensource-desc {
  margin: 0;
  font-size: 13px;
  color: var(--color-text-muted);
  line-height: 1.6;
}

.category {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.category-title {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
  padding-bottom: 6px;
  border-bottom: 1px solid var(--color-border);
}

.lib-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.lib-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-radius: 10px;
  background: var(--color-surface);
  border: 1px solid transparent;
  text-decoration: none;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
}

.lib-item:hover {
  border-color: var(--color-border-light);
  background: var(--color-surface-secondary);
  transform: translateX(2px);
}

.lib-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.lib-name {
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.lib-desc {
  font-size: 12px;
  color: var(--color-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.lib-version {
  font-size: 12px;
  color: var(--color-primary);
  font-weight: 600;
  background: var(--color-primary-alpha-10);
  padding: 2px 10px;
  border-radius: 12px;
  flex-shrink: 0;
  margin-left: 12px;
}
</style>
