<template>
  <div class="agent-tools-page">
    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="19" y1="12" x2="5" y2="12"></line>
            <polyline points="12,19 5,12 12,5"></polyline>
          </svg>
        </button>
        <h2>工具管理</h2>
      </div>
      <div class="header-actions">
        <button class="action-btn add" @click="showAddDialog = true">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          <span>新增</span>
        </button>
        <button class="action-btn scan" @click="handleScan" :disabled="isScanning">
          <svg v-if="!isScanning" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
          <svg v-else viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
            <line x1="12" y1="2" x2="12" y2="6"></line>
            <line x1="12" y1="18" x2="12" y2="22"></line>
            <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
            <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
            <line x1="2" y1="12" x2="6" y2="12"></line>
            <line x1="18" y1="12" x2="22" y2="12"></line>
            <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
            <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
          </svg>
          <span>{{ isScanning ? '扫描中...' : '扫描' }}</span>
        </button>
        <button class="action-btn open-folder" @click="handleOpenFolder">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
          <span>打开目录</span>
        </button>
      </div>
    </div>

    <div class="category-tabs">
      <button
        v-for="tab in categoryTabs"
        :key="tab.value"
        :class="['tab-btn', { active: activeCategory === tab.value }]"
        @click="activeCategory = tab.value"
      >
        {{ tab.label }}
        <span class="tab-count">{{ getTabCount(tab.value) }}</span>
      </button>
    </div>

    <div class="tools-list" v-if="filteredTools.length > 0">
      <div v-for="tool in filteredTools" :key="tool.id" class="tool-item">
        <div class="tool-card-header">
          <div class="tool-logo" @click="handleEditTool(tool)">
            <img v-if="tool.logo && tool.logoType" :src="`data:${tool.logoType};base64,${tool.logo}`" alt="logo" class="logo-img" />
            <div v-else class="logo-placeholder">
              <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.5">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <line x1="3" y1="9" x2="21" y2="9"></line>
                <line x1="9" y1="21" x2="9" y2="9"></line>
              </svg>
            </div>
          </div>
          <div class="tool-info" @click="handleEditTool(tool)">
            <h3 class="tool-name">{{ tool.name }}</h3>
            <div class="tool-invoke">
              <code class="invoke-name">{{ tool.invokeName }}</code>
            </div>
          </div>
          <div class="tool-actions">
            <button v-if="!tool.id.startsWith('sys-')" class="tool-action-btn code" @click.stop="handleEditCode(tool)" title="编辑代码">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="16,18 22,12 16,6"></polyline>
                <polyline points="8,6 2,12 8,18"></polyline>
              </svg>
            </button>
            <button v-if="!tool.id.startsWith('sys-')" class="tool-action-btn edit" @click.stop="handleEditTool(tool)" title="编辑信息">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
              </svg>
            </button>
            <button v-if="tool.filePath" class="tool-action-btn delete" @click.stop="handleDeleteTool(tool)" title="删除">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3,6 5,6 21,6"></polyline>
                <path d="M19,6v14a2,2 0 0 1-2,2H7a2,2 0 0 1-2-2V6m3,0V4a2,2 0 0 1 2-2h4a2,2 0 0 1 2,2v2"></path>
              </svg>
            </button>
          </div>
        </div>
        <p class="tool-description">{{ tool.description || '暂无简介' }}</p>
      </div>
    </div>

    <div class="empty-state" v-else>
      <div class="empty-icon">
        <svg viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
          <line x1="3" y1="9" x2="21" y2="9"></line>
          <line x1="9" y1="21" x2="9" y2="9"></line>
        </svg>
      </div>
      <p>还没有工具</p>
      <span>点击"扫描"按钮发现工具，或点击"新增"创建新工具</span>
    </div>

    <BaseDialog v-model="showAddDialog" :title="isViewingBuiltin ? '查看工具' : (editingTool ? '编辑工具' : '新增工具')" @close="closeDialog">
      <div class="form-group">
        <label>工具名称</label>
        <input v-model="toolForm.name" placeholder="例如：计算器" :disabled="isViewingBuiltin" />
      </div>
      <div class="form-group">
        <label>工具简介</label>
        <textarea v-model="toolForm.description" placeholder="简要描述工具的功能" rows="2" :disabled="isViewingBuiltin"></textarea>
      </div>
      <div class="form-group">
        <label>调用名</label>
        <input v-model="toolForm.invokeName" placeholder="例如：calculator" :disabled="isViewingBuiltin" />
      </div>
      <div class="form-group">
        <label>Logo 图标</label>
        <div class="logo-upload">
          <div class="logo-preview" :class="{ 'clickable': !isViewingBuiltin }" @click="!isViewingBuiltin && triggerLogoUpload()">
            <img v-if="toolForm.logoPreview" :src="toolForm.logoPreview" alt="logo" class="preview-img" />
            <div v-else class="preview-placeholder">
              <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <circle cx="8.5" cy="8.5" r="1.5"></circle>
                <polyline points="21,15 16,10 5,21"></polyline>
              </svg>
              <span>点击上传图片</span>
            </div>
          </div>
          <input
            ref="logoInputRef"
            type="file"
            accept="image/*"
            @change="handleLogoChange"
            style="display: none"
          />
          <button v-if="toolForm.logoPreview && !isViewingBuiltin" class="clear-logo-btn" @click="clearLogo" type="button">清除</button>
        </div>
      </div>
      <template #actions>
        <button class="cancel-btn" @click="closeDialog">取消</button>
        <button v-if="!isViewingBuiltin" class="save-btn" @click="handleSaveTool" :disabled="!canSave">
          {{ editingTool ? '保存' : '创建' }}
        </button>
      </template>
    </BaseDialog>

    <BaseDialog v-model="showCodeDialog" title="" :maxWidth=" '800px'" :width=" '90%'" @close="showCodeDialog = false">
      <div class="code-editor-header">
        <h3>编辑代码 - {{ editingCodeTool?.name }}</h3>
      </div>
      <div ref="codeEditorRef" class="code-editor-container"></div>
      <template #actions>
        <button class="cancel-btn" @click="showCodeDialog = false">取消</button>
        <button class="save-btn" @click="handleSaveCode">保存代码</button>
      </template>
    </BaseDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, nextTick } from 'vue'
import BaseDialog from './BaseDialog.vue'
import { invoke } from '@tauri-apps/api/core'
import { EditorState } from '@codemirror/state'
import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightSpecialChars, drawSelection, dropCursor, rectangularSelection, crosshairCursor, highlightActiveLine } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { javascript } from '@codemirror/lang-javascript'
import { oneDark } from '@codemirror/theme-one-dark'
import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, foldGutter, foldKeymap } from '@codemirror/language'
import type { Tool } from '../types/tool'
import type { ExecutorName } from '../agent/fairyDo'
import { sysTools } from '../agent/FairySysTools'
import { useToolWatcher } from '../composables/useToolWatcher'
import { 
  loadAllTools, 
  loadToolByFilePath,
  saveTool, 
  deleteTool as deleteToolFromDb,
  parseAllTools,
  fileToBase64,
  cleanBuiltinToolsFromDb
} from '../stores/toolsStore'

defineEmits<{
  back: []
}>()

const tools = ref<Tool[]>([])
const showAddDialog = ref(false)
const showCodeDialog = ref(false)
const isScanning = ref(false)
const editingTool = ref<Tool | null>(null)
const editingCodeTool = ref<Tool | null>(null)
const logoInputRef = ref<HTMLInputElement | null>(null)
const codeEditorRef = ref<HTMLDivElement | null>(null)
let editorView: EditorView | null = null

const toolForm = reactive({
  name: '',
  description: '',
  invokeName: '',
  logo: '',
  logoType: '',
  logoPreview: '',
  executor: '',
  language: 'typescript' as 'typescript'
})

const codeForm = reactive({
  code: ''
})

const canSave = computed(() => {
  return toolForm.name && toolForm.invokeName
})

type CategoryType = 'all' | 'builtin' | 'custom'

const activeCategory = ref<CategoryType>('all')

const categoryTabs = [
  { label: '全部', value: 'all' as CategoryType },
  { label: '预置', value: 'builtin' as CategoryType },
  { label: '自定义', value: 'custom' as CategoryType }
]

const filteredTools = computed(() => {
  if (activeCategory.value === 'all') {
    return tools.value
  }
  if (activeCategory.value === 'builtin') {
    return tools.value.filter(t => t.id.startsWith('sys-'))
  }
  return tools.value.filter(t => !t.id.startsWith('sys-'))
})

const getTabCount = (category: CategoryType) => {
  if (category === 'all') return tools.value.length
  if (category === 'builtin') return tools.value.filter(t => t.id.startsWith('sys-')).length
  return tools.value.filter(t => !t.id.startsWith('sys-')).length
}

const isViewingBuiltin = computed(() => {
  return editingTool.value?.id.startsWith('sys-') ?? false
})

const defaultTsCode = `interface ToolParams {
  action: string;
  [key: string]: any;
}

interface ToolResult {
  result?: any;
  error?: string;
}

async function execute(params: ToolParams): Promise<ToolResult> {
  const { action, ...rest } = params;
  
  switch (action) {
    case 'example':
      return { result: '示例执行成功' };
    default:
      return { error: '未知操作' };
  }
}

export { execute };
`

onMounted(async () => {
  await cleanBuiltinToolsFromDb()
  await loadTools()
})

const loadTools = async () => {
  try {
    const userTools = await loadAllTools()
    const filtered = userTools.filter(t => !t.id.startsWith('sys-') && !t.id.startsWith('builtin-'))
    tools.value = [...sysTools, ...filtered]
    console.log('[AgentToolsPage] 已加载工具:', tools.value.length, '(预置:', sysTools.length, ', 自定义:', filtered.length, ')')
  } catch (error) {
    console.error('[AgentToolsPage] 加载工具失败:', error)
  }
}

const handleScan = async () => {
  if (isScanning.value) return
  
  isScanning.value = true
  try {
    const result = await invoke<{ tools: Array<{ name: string; path: string; content: string }>; fairy_tool_path: string }>('scan_tools')
    console.log('[AgentToolsPage] 扫描结果:', result)
    
    const scannedPaths = new Set(result.tools.map(t => t.path))
    
    const allTools = await loadAllTools()
    
    for (const tool of allTools) {
      if (tool.filePath && !scannedPaths.has(tool.filePath)) {
        await deleteToolFromDb(tool.id)
        console.log('[AgentToolsPage] 删除不存在文件的工具:', tool.name, tool.filePath)
      }
    }
    
    for (const toolFile of result.tools) {
      const parsedTools = parseAllTools(toolFile.content)
      for (const { metadata, code } of parsedTools) {
        const existingTool = await loadToolByFilePath(toolFile.path)
        const now = new Date().toISOString()
        
        const tool: Tool = {
          id: existingTool?.id || `tool-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          name: metadata.name,
          description: metadata.description,
          logo: existingTool?.logo || '',
          logoType: existingTool?.logoType,
          invokeName: metadata.invokeName,
          filePath: toolFile.path,
          parameters: existingTool?.parameters || [],
          executor: metadata.invokeName as ExecutorName,
          code,
          language: 'typescript',
          createdAt: existingTool?.createdAt || now,
          updatedAt: now
        }
        await saveTool(tool)
      }
    }
    
    tools.value = [...sysTools, ...await loadAllTools()]
  } catch (error) {
    console.error('[AgentToolsPage] 扫描失败:', error)
  } finally {
    isScanning.value = false
  }
}

const { start: startWatcher } = useToolWatcher(async () => {
  await handleScan()
})

onMounted(() => {
  startWatcher()
})

const handleOpenFolder = async () => {
  try {
    const path = await invoke<string>('get_fairy_tool_path')
    await invoke('open_folder', { path })
  } catch (error) {
    console.error('[AgentToolsPage] 打开目录失败:', error)
  }
}

const handleEditTool = (tool: Tool) => {
  editingTool.value = tool
  toolForm.name = tool.name
  toolForm.description = tool.description
  toolForm.invokeName = tool.invokeName
  toolForm.logo = tool.logo || ''
  toolForm.logoType = tool.logoType || ''
  toolForm.logoPreview = tool.logo && tool.logoType ? `data:${tool.logoType};base64,${tool.logo}` : ''
  toolForm.executor = tool.executor || ''
  toolForm.language = tool.language
  showAddDialog.value = true
}

const handleEditCode = (tool: Tool) => {
  editingCodeTool.value = tool
  codeForm.code = tool.code || ''
  showCodeDialog.value = true
  nextTick(() => {
    initCodeMirror()
  })
}

const initCodeMirror = () => {
  if (!codeEditorRef.value) return
  
  if (editorView) {
    editorView.destroy()
  }
  
  const extensions = [
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightSpecialChars(),
    history(),
    foldGutter(),
    drawSelection(),
    dropCursor(),
    EditorState.allowMultipleSelections.of(true),
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    bracketMatching(),
    rectangularSelection(),
    crosshairCursor(),
    highlightActiveLine(),
    keymap.of([
      ...defaultKeymap,
      ...historyKeymap,
      ...foldKeymap
    ]),
    javascript({ typescript: true }),
    oneDark,
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        codeForm.code = update.state.doc.toString()
      }
    }),
    EditorView.theme({
      '&': { height: '100%' },
      '.cm-scroller': { overflow: 'auto' }
    })
  ]
  
  editorView = new EditorView({
    state: EditorState.create({
      doc: codeForm.code,
      extensions
    }),
    parent: codeEditorRef.value
  })
}

onUnmounted(() => {
  if (editorView) {
    editorView.destroy()
  }
})

const handleDeleteTool = async (tool: Tool) => {
  if (!tool.filePath) return
  
  try {
    await deleteToolFromDb(tool.id)
    await invoke('delete_tool_file', { filename: tool.filePath })
    tools.value = tools.value.filter(t => t.id !== tool.id)
    console.log('[AgentToolsPage] 工具已删除:', tool.name)
  } catch (error) {
    console.error('[AgentToolsPage] 删除工具失败:', error)
  }
}

const closeDialog = () => {
  showAddDialog.value = false
  editingTool.value = null
  toolForm.name = ''
  toolForm.description = ''
  toolForm.invokeName = ''
  toolForm.logo = ''
  toolForm.logoType = ''
  toolForm.logoPreview = ''
  toolForm.executor = ''
  toolForm.language = 'typescript'
}

const triggerLogoUpload = () => {
  logoInputRef.value?.click()
}

const handleLogoChange = async (event: Event) => {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  
  try {
    const { data, type } = await fileToBase64(file)
    toolForm.logo = data
    toolForm.logoType = type
    toolForm.logoPreview = `data:${type};base64,${data}`
    console.log('[AgentToolsPage] Logo 上传成功:', type, data.substring(0, 50) + '...')
  } catch (error) {
    console.error('[AgentToolsPage] Logo 上传失败:', error)
  }
}

const clearLogo = () => {
  toolForm.logo = ''
  toolForm.logoType = ''
  toolForm.logoPreview = ''
  if (logoInputRef.value) {
    logoInputRef.value.value = ''
  }
}

const handleSaveCode = async () => {
  if (!editingCodeTool.value || !codeForm.code) return

  try {
    const tool = editingCodeTool.value
    tool.code = codeForm.code
    tool.updatedAt = new Date().toISOString()
    
    await saveTool(tool)
    
    const filename = `${tool.invokeName}.ts`
    const content = `import { defineTool } from '@/agent/fairyDo'

export default defineTool({
  name: '${tool.name}',
  description: '${tool.description}',
  parameters: [],
  executor: '${tool.executor || tool.invokeName}'
})
`
    
    const projectRoot = await invoke<string>('get_fairy_tool_path')
    const fullPath = `${projectRoot}/${filename}`
    tool.filePath = fullPath
    
    await invoke('save_tool_file', { filename, content })
    
    const index = tools.value.findIndex(t => t.id === tool.id)
    if (index !== -1) {
      tools.value[index] = { ...tool }
    }
    
    showCodeDialog.value = false
    console.log('[AgentToolsPage] 代码已保存:', tool.name)
  } catch (error) {
    console.error('[AgentToolsPage] 保存代码失败:', error)
  }
}

const handleSaveTool = async () => {
  if (!canSave.value) return

  try {
    const filename = `${toolForm.invokeName}.ts`
    const codeTemplate = defaultTsCode
    const content = `import { defineTool } from '@/agent/fairyDo'

export default defineTool({
  name: '${toolForm.name}',
  description: '${toolForm.description}',
  parameters: [],
  executor: '${toolForm.executor || toolForm.invokeName}'
})
`

    await invoke('save_tool_file', { filename, content })
    
    const now = new Date().toISOString()
    const projectRoot = await invoke<string>('get_fairy_tool_path')
    const fullPath = `${projectRoot}/${filename}`
    
    const tool: Tool = {
      id: editingTool.value?.id || `tool-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      name: toolForm.name,
      description: toolForm.description,
      logo: toolForm.logo,
      logoType: toolForm.logoType,
      invokeName: toolForm.invokeName,
      filePath: fullPath,
      parameters: editingTool.value?.parameters || [],
      executor: (toolForm.executor || toolForm.invokeName) as ExecutorName,
      code: codeTemplate,
      language: 'typescript',
      createdAt: editingTool.value?.createdAt || now,
      updatedAt: now
    }

    await saveTool(tool)
    
    if (editingTool.value) {
      const index = tools.value.findIndex(t => t.id === tool.id)
      if (index !== -1) {
        tools.value[index] = tool
      }
    } else {
      tools.value.push(tool)
    }

    closeDialog()
    console.log('[AgentToolsPage] 工具已保存:', tool.name)
  } catch (error) {
    console.error('[AgentToolsPage] 保存工具失败:', error)
  }
}
</script>

<style scoped>
.agent-tools-page {
  width: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-surface-secondary) 50%, var(--color-surface-tertiary) 100%);
  overflow: hidden;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 28px;
  background: var(--color-surface-card);
  border-bottom: 1px solid var(--color-border);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.back-btn {
  width: 40px;
  height: 40px;
  border: none;
  background: var(--color-surface-secondary);
  cursor: pointer;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.back-btn:hover {
  background: var(--color-surface-tertiary);
  color: var(--color-text-primary);
}

.page-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  letter-spacing: -0.02em;
}

.category-tabs {
  display: flex;
  gap: 4px;
  padding: 12px 24px;
  background: var(--color-surface-card);
  border-bottom: 1px solid var(--color-border);
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  background: transparent;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab-btn:hover {
  background: var(--color-surface-secondary);
  color: var(--color-primary);
}

.tab-btn.active {
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
}

.tab-count {
  font-size: 11px;
  padding: 2px 6px;
  background: rgba(0, 0, 0, 0.08);
  border-radius: 10px;
}

.tab-btn.active .tab-count {
  background: rgba(255, 255, 255, 0.25);
}

.page-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  letter-spacing: -0.02em;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border: 1px solid var(--color-border);
  background: var(--color-surface-card);
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.action-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-alpha-10);
}

.action-btn.active {
  border-color: var(--color-primary);
  background: var(--color-primary-gradient);
  opacity: 0.1;
  color: var(--color-primary);
}

.action-btn.add {
  background: var(--color-primary-gradient);
  border-color: var(--color-primary);
  color: var(--color-text-inverse);
  box-shadow: 0 2px 8px var(--color-shadow-primary-hover);
}

.action-btn.add:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 16px var(--color-shadow-primary-strong);
}

.action-btn.scan:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.tools-list {
  flex: 1;
  padding: 16px 24px;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  align-content: start;
}

.tool-item {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px;
  background: var(--color-surface-card);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.tool-item:hover {
  border-color: var(--color-border-light);
  background: var(--color-primary-alpha-08);
  box-shadow: 0 2px 8px var(--color-shadow-alpha-06);
}

.tool-card-header {
  display: flex;
  align-items: center;
  gap: 14px;
}

.tool-logo {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: linear-gradient(135deg, var(--color-surface-secondary) 0%, var(--color-surface-tertiary) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  cursor: pointer;
}

.logo-placeholder {
  color: var(--color-primary);
}

.logo-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: 10px;
}

.logo-upload {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-preview {
  width: 80px;
  height: 80px;
  border: 2px dashed var(--color-border);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.25s;
  overflow: hidden;
  background: var(--color-surface);
}

.logo-preview:hover {
  border-color: var(--color-primary);
  background: var(--color-surface-secondary);
}

.preview-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.preview-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  color: var(--color-text-muted);
}

.preview-placeholder span {
  font-size: 10px;
}

.preview-placeholder svg {
  color: var(--color-primary);
}

.clear-logo-btn {
  padding: 6px 12px;
  border: 1px solid var(--color-accent-error);
  background: transparent;
  color: var(--color-accent-error);
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.clear-logo-btn:hover {
  background: var(--color-accent-error);
  color: var(--color-text-inverse);
}

.tool-info {
  flex: 1;
  min-width: 0;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.tool-name {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.tool-description {
  margin: 0;
  padding-left: 54px;
  font-size: 12px;
  color: var(--color-text-secondary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.5;
}

.tool-invoke {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  flex-shrink: 0;
}

.invoke-label {
  color: var(--color-text-muted);
}

.invoke-name {
  background: var(--color-surface-secondary);
  padding: 4px 8px;
  border-radius: 6px;
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 11px;
  color: var(--color-primary);
}

.tool-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.tool-action-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.tool-action-btn.code {
  background: var(--color-accent-info-alpha-10);
  color: var(--color-accent-info);
}

.tool-action-btn.code:hover {
  background: var(--color-accent-info);
  color: var(--color-text-inverse);
}

.tool-action-btn.edit {
  background: var(--color-surface-secondary);
  color: var(--color-text-secondary);
}

.tool-action-btn.edit:hover {
  background: var(--color-primary);
  color: var(--color-text-inverse);
}

.tool-action-btn.delete {
  background: var(--color-danger-bg);
  color: var(--color-text-secondary);
}

.tool-action-btn.delete:hover {
  background: var(--color-accent-error);
  color: var(--color-text-inverse);
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  padding: 40px;
}

.empty-icon {
  margin-bottom: 20px;
  color: var(--color-border);
}

.empty-icon svg {
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { transform: scale(1); opacity: 0.6; }
  50% { transform: scale(1.05); opacity: 1; }
}

.empty-state p {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.empty-state span {
  font-size: 13px;
}

.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-shadow-alpha-40);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.dialog {
  width: 100%;
  max-width: 480px;
  max-height: 80vh;
  background: var(--color-surface-card);
  border-radius: 20px;
  box-shadow: 0 20px 60px var(--color-shadow-alpha-20);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  animation: slideUp 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.code-dialog {
  width: 90%;
  max-width: 800px;
  height: 80vh;
  background: var(--color-surface-card);
  border-radius: 20px;
  box-shadow: 0 20px 60px var(--color-shadow-alpha-20);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: slideUp 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 28px;
  border-bottom: 1px solid var(--color-border);
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.close-btn {
  width: 36px;
  height: 36px;
  border: none;
  background: var(--color-surface-secondary);
  cursor: pointer;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all 0.25s;
}

.close-btn:hover {
  background: var(--color-surface-tertiary);
  color: var(--color-text-primary);
}

.dialog-content {
  padding: 28px;
  overflow-y: auto;
  flex: 1;
}

.form-group {
  margin-bottom: 20px;
}

.form-group:last-child {
  margin-bottom: 0;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.form-group input,
.form-group textarea,
.form-group select {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-surface);
  transition: all 0.25s;
  font-family: inherit;
  resize: vertical;
}

.form-group input:focus,
.form-group textarea:focus,
.form-group select:focus {
  outline: none;
  border-color: var(--color-primary);
  background: var(--color-surface-card);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.form-group input::placeholder,
.form-group textarea::placeholder {
  color: var(--color-text-muted);
}

.form-group input:disabled,
.form-group textarea:disabled {
  background: var(--color-surface-secondary);
  color: var(--color-text-muted);
  cursor: not-allowed;
}

.logo-preview.clickable {
  cursor: pointer;
}

.logo-preview.clickable:hover {
  border-color: var(--color-primary);
  background: var(--color-primary-alpha-10);
}

.code-editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
  margin-bottom: 16px;
}

.code-editor-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.code-editor-header select {
  width: auto;
  padding: 8px 12px;
  font-size: 13px;
}

.code-editor-container {
  flex: 1;
  overflow: hidden;
  min-height: 300px;
}

.code-editor-container .cm-editor {
  height: 100%;
}

.dialog-actions {
  display: flex;
  gap: 12px;
  padding: 20px 28px;
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

.cancel-btn {
  flex: 1;
  padding: 14px;
  border: 1px solid var(--color-border);
  background: var(--color-surface-card);
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.25s;
}

.cancel-btn:hover {
  border-color: var(--color-text-muted);
  background: var(--color-surface-secondary);
}

.save-btn {
  flex: 2;
  padding: 14px;
  border: none;
  border-radius: 10px;
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.25s;
  box-shadow: 0 4px 16px var(--color-shadow-primary-hover);
}

.save-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px var(--color-shadow-primary-strong);
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
