<template>
  <div class="agent-memory-page">
    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="19" y1="12" x2="5" y2="12"></line>
            <polyline points="12,19 5,12 12,5"></polyline>
          </svg>
        </button>
        <h2>记忆</h2>
      </div>
      <div class="header-right">
        <span class="status-indicator" :class="{ active: stats.enabled }">
          <span class="status-dot"></span>
          {{ stats.enabled ? 'FBM 已启用' : 'FBM 未启用' }}
        </span>
        <button class="config-btn" @click="showFbmConfig = true">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"></circle>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
          </svg>
          FBM 配置
        </button>
      </div>
    </div>

    <div class="page-content">
      <div class="config-summary" v-if="stats.enabled">
        <div class="summary-row">
          <span class="summary-label">记忆副模型</span>
          <span class="summary-value">{{ llmProviderName || '未配置' }}</span>
        </div>
        <div class="summary-row">
          <span class="summary-label">Embedding 模型</span>
          <span class="summary-value">{{ embeddingProviderName || '未配置' }}</span>
        </div>
        <div class="summary-row">
          <span class="summary-label">记忆整合</span>
          <span class="summary-value">{{ fbmConfig.consolidationEnabled ? '已启用' : '已禁用' }}</span>
        </div>
        <div class="summary-row">
          <span class="summary-label">检索精炼</span>
          <span class="summary-value">{{ fbmConfig.refineResults ? '已启用' : '已禁用' }}</span>
        </div>
      </div>

      <div class="stats-bar">
        <div class="stat-item">
          <span class="stat-value">{{ stats.totalMemories }}</span>
          <span class="stat-label">记忆区块</span>
        </div>
        <div class="stat-divider"></div>
        <div class="stat-item">
          <span class="stat-value">{{ stats.vectorCount }}</span>
          <span class="stat-label">目录条目</span>
        </div>
        <div class="stat-actions">
          <button
            class="stat-btn"
            @click="handleConsolidate"
            :disabled="isConsolidating"
            v-if="fbmConfig.enabled"
          >
            <svg v-if="isConsolidating" class="spinning" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 11-6.219-8.56"></path>
            </svg>
            <svg v-else viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 20h9"></path>
              <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path>
            </svg>
            {{ isConsolidating ? '整合中' : '整合' }}
          </button>
          <button
            class="stat-btn"
            @click="handleReindex"
            :disabled="isReindexing"
          >
            <svg v-if="isReindexing" class="spinning" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 11-6.219-8.56"></path>
            </svg>
            <svg v-else viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="23,4 23,10 17,10"></polyline>
              <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
            </svg>
            {{ isReindexing ? '重建中' : '重建' }}
          </button>
          <button class="stat-btn danger" @click="handleClearVectors" :disabled="isClearing">
            <svg v-if="isClearing" class="spinning" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 11-6.219-8.56"></path>
            </svg>
            <svg v-else viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3,6 5,6 21,6"></polyline>
              <path d="M19,6v14a2,2,0,0,1-2,2H7a2,2,0,0,1-2-2V6m3,0V4a2,2,0,0,1,2-2h4a2,2,0,0,1,2,2v2"></path>
            </svg>
            {{ isClearing ? '清空中' : '清空' }}
          </button>
          <button class="stat-btn" @click="handleRefreshStats" :disabled="isRefreshingStats">
            <svg :class="{ spinning: isRefreshingStats }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="23,4 23,10 17,10"></polyline>
              <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
            </svg>
            刷新
          </button>
        </div>
      </div>

      <div v-if="reindexResult !== null" class="result-msg">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="20,6 9,17 4,12"></polyline>
        </svg>
        {{ reindexResult }}
      </div>
      <div v-if="consolidateResult" class="result-msg">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="20,6 9,17 4,12"></polyline>
        </svg>
        {{ consolidateResult }}
      </div>

      <div class="search-bar">
        <svg class="search-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索记忆..."
          class="search-input"
          :class="{ 'has-value': searchQuery.trim() }"
        />
        <button class="search-clear" v-if="searchQuery.trim()" @click="searchQuery = ''">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
      <div class="search-status" v-if="searchQuery.trim() && hasMemories">
        <span>{{ filteredItems.length === 0 ? '没有找到匹配的记忆' : `找到 ${filteredItems.length} 条结果` }}</span>
      </div>

      <div class="memory-list" v-if="hasMemories">
        <div v-for="item in filteredItems" :key="item.path" class="memory-card" @click="openDetail(item)">
          <div class="card-header">
            <span class="card-title">{{ item.title }}</span>
            <button class="delete-btn" @click.stop="confirmDelete(item)" title="删除">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3,6 5,6 21,6"></polyline>
                <path d="M19,6v14a2,2,0,0,1-2,2H7a2,2,0,0,1-2-2V6m3,0V4a2,2,0,0,1,2-2h4a2,2,0,0,1,2,2v2"></path>
              </svg>
            </button>
          </div>
          <p class="card-preview">{{ item.preview }}</p>
        </div>
        <div class="no-results" v-if="searchQuery.trim() && filteredItems.length === 0">
          <p>未找到匹配「{{ searchQuery }}」的记忆</p>
        </div>
      </div>

      <div class="empty-state" v-if="!hasMemories">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M12 2a7 7 0 0 1 7 7c0 2.38-1.19 4.47-3 5.74V17a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2v-2.26C6.19 13.47 5 11.38 5 9a7 7 0 0 1 7-7z"></path>
            <line x1="9" y1="21" x2="15" y2="21"></line>
          </svg>
        </div>
        <p>暂无记忆</p>
        <span>开始对话后记忆会自动积累</span>
      </div>
    </div>

    <BaseDialog v-model="showFbmConfig" title="FBM 记忆系统配置" width="92%" maxWidth="560px" maxHeight="85vh">
      <div class="fbm-config">
        <div class="config-section">
          <div class="config-section-header">
            <h4>基本设置</h4>
          </div>
          <div class="toggle-row">
            <div class="toggle-info">
              <span class="toggle-label">启用 FBM 记忆系统</span>
              <span class="toggle-desc">启用后智能体将自动积累和检索记忆</span>
            </div>
            <label class="toggle-switch">
              <input type="checkbox" v-model="fbmConfig.enabled" />
              <span class="toggle-slider"></span>
            </label>
          </div>
          <div class="toggle-row" v-if="fbmConfig.enabled">
            <div class="toggle-info">
              <span class="toggle-label">记忆整合</span>
              <span class="toggle-desc">对话空闲时自动提取并整合记忆</span>
            </div>
            <label class="toggle-switch">
              <input type="checkbox" v-model="fbmConfig.consolidationEnabled" />
              <span class="toggle-slider"></span>
            </label>
          </div>
          <div class="toggle-row" v-if="fbmConfig.enabled">
            <div class="toggle-info">
              <span class="toggle-label">检索精炼</span>
              <span class="toggle-desc">使用副模型精炼检索结果，关闭后返回原始内容</span>
            </div>
            <label class="toggle-switch">
              <input type="checkbox" v-model="fbmConfig.refineResults" />
              <span class="toggle-slider"></span>
            </label>
          </div>
          <div class="toggle-row" v-if="fbmConfig.enabled">
            <div class="toggle-info">
              <span class="toggle-label">智能召回</span>
              <span class="toggle-desc">开启后模型自主决定何时搜索记忆，关闭后每次自动搜索</span>
            </div>
            <label class="toggle-switch">
              <input type="checkbox" v-model="fbmConfig.smartRecall" />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>

        <div class="config-section" v-if="fbmConfig.enabled">
          <div class="config-section-header">
            <h4>模型配置</h4>
          </div>
          <div class="form-group">
            <label class="form-label">记忆副模型</label>
            <BaseSelect
              :modelValue="fbmConfig.providerId"
              :options="providerOptions"
              @update:modelValue="fbmConfig.providerId = $event"
            />
          </div>
          <div class="form-group">
            <label class="form-label">Embedding Provider</label>
            <BaseSelect
              :modelValue="fbmConfig.embeddingProviderId"
              :options="providerOptions"
              @update:modelValue="fbmConfig.embeddingProviderId = $event"
            />
          </div>
        </div>
      </div>
      <template #actions>
        <button class="cancel-btn" @click="handleCancelFbmConfig">取消</button>
        <button class="save-btn" @click="handleSaveFbmConfig">保存</button>
      </template>
    </BaseDialog>

    <BaseDialog v-model="showDetail" :title="detailItem?.title ?? '记忆详情'" width="90%" maxWidth="680px" maxHeight="80vh" @close="showDetail = false">
      <div class="detail-content">
        <span class="detail-path">{{ detailItem?.path }}</span>
        <pre class="detail-body">{{ detailContent }}</pre>
      </div>
      <template #actions>
        <button class="cancel-btn" @click="showDetail = false">关闭</button>
      </template>
    </BaseDialog>

    <BaseDialog v-model="showDeleteConfirm" title="确认删除" width="90%" maxWidth="400px" @close="showDeleteConfirm = false">
      <div class="confirm-content">
        <p>确定要删除记忆 <strong>{{ deletingItem?.title }}</strong> 吗？</p>
        <span>此操作不可撤销</span>
      </div>
      <template #actions>
        <button class="cancel-btn" @click="showDeleteConfirm = false">取消</button>
        <button class="save-btn danger" @click="handleDelete" :disabled="isDeleting">
          {{ isDeleting ? '删除中...' : '删除' }}
        </button>
      </template>
    </BaseDialog>

    <BaseDialog v-model="showReindexConfirm" title="确认重建索引" width="90%" maxWidth="400px" @close="showReindexConfirm = false">
      <div class="confirm-content">
        <p>确定要重建向量索引吗？</p>
        <span>将重新生成所有记忆的向量索引，可能需要一些时间</span>
      </div>
      <template #actions>
        <button class="cancel-btn" @click="showReindexConfirm = false">取消</button>
        <button class="save-btn" @click="doReindex" :disabled="isReindexing">
          {{ isReindexing ? '重建中...' : '确认重建' }}
        </button>
      </template>
    </BaseDialog>

    <BaseDialog v-model="showClearConfirm" title="确认清空" width="90%" maxWidth="400px" @close="showClearConfirm = false">
      <div class="confirm-content">
        <p>确定要清空所有向量数据吗？</p>
        <span>此操作不可撤销，所有记忆索引将被删除</span>
      </div>
      <template #actions>
        <button class="cancel-btn" @click="showClearConfirm = false">取消</button>
        <button class="save-btn danger" @click="doClearVectors" :disabled="isClearing">
          {{ isClearing ? '清空中...' : '确认清空' }}
        </button>
      </template>
    </BaseDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import BaseDialog from './BaseDialog.vue'
import BaseSelect from './BaseSelect.vue'
import type { SelectOption } from './BaseSelect.vue'
import { fbmStore } from '../stores/fbmStore'
import { loadSettings, saveSettings } from '../stores/settings'
import type { ApiProvider } from '../stores/settings'

defineEmits<{
  back: []
}>()

interface MemoryItem {
  title: string
  path: string
  preview: string
  content: string
}

const stats = reactive({ totalMemories: 0, vectorCount: 0, enabled: false })
const searchQuery = ref('')
const memoryItems = ref<MemoryItem[]>([])
const isReindexing = ref(false)
const isClearing = ref(false)
const showReindexConfirm = ref(false)
const showClearConfirm = ref(false)
const isDeleting = ref(false)
const isConsolidating = ref(false)
const isRefreshingStats = ref(false)
const consolidateResult = ref('')
const reindexResult = ref<string | null>(null)
const showDetail = ref(false)
const detailItem = ref<MemoryItem | null>(null)
const detailContent = ref('')
const showDeleteConfirm = ref(false)
const deletingItem = ref<MemoryItem | null>(null)

const showFbmConfig = ref(false)
const fbmConfig = reactive({
  enabled: false,
  providerId: null as string | null,
  embeddingProviderId: null as string | null,
  consolidationEnabled: true,
  refineResults: true,
  smartRecall: true,
})
const providers = ref<ApiProvider[]>([])

const providerOptions = computed<SelectOption[]>(() => {
  return providers.value.map(p => ({
    value: p.id,
    label: p.displayName,
    sub: p.model || undefined
  }))
})

const hasMemories = computed(() => {
  return memoryItems.value.length > 0
})

const filteredItems = computed(() => {
  if (!searchQuery.value.trim()) return memoryItems.value
  const q = searchQuery.value.toLowerCase()
  return memoryItems.value.filter(
    item =>
      item.title.toLowerCase().includes(q) ||
      item.preview.toLowerCase().includes(q) ||
      item.content.toLowerCase().includes(q)
  )
})

const llmProviderName = computed(() => {
  if (!fbmConfig.providerId) return ''
  return providers.value.find(p => p.id === fbmConfig.providerId)?.displayName ?? ''
})

const embeddingProviderName = computed(() => {
  if (!fbmConfig.embeddingProviderId) return ''
  return providers.value.find(p => p.id === fbmConfig.embeddingProviderId)?.displayName ?? ''
})

function loadSettingsToConfig() {
  const s = loadSettings()
  fbmConfig.enabled = s.fbmEnabled ?? false
  fbmConfig.providerId = s.fbmProviderId ?? null
  fbmConfig.embeddingProviderId = s.fbmEmbeddingProviderId ?? null
  fbmConfig.consolidationEnabled = s.fbmConsolidationEnabled ?? true
  fbmConfig.refineResults = s.fbmRefineResults !== false
  fbmConfig.smartRecall = s.fbmSmartRecall !== false
  providers.value = s.providers
}

function saveFbmConfig() {
  const s = loadSettings()
  s.fbmEnabled = fbmConfig.enabled
  s.fbmProviderId = fbmConfig.providerId
  s.fbmEmbeddingProviderId = fbmConfig.embeddingProviderId
  s.fbmConsolidationEnabled = fbmConfig.consolidationEnabled
  s.fbmRefineResults = fbmConfig.refineResults
  s.fbmSmartRecall = fbmConfig.smartRecall
  saveSettings(s)
}

function handleCancelFbmConfig() {
  loadSettingsToConfig()
  showFbmConfig.value = false
}

async function handleSaveFbmConfig() {
  const wasEnabled = stats.enabled
  const oldProviderId = loadSettings().fbmProviderId
  const oldEmbeddingProviderId = loadSettings().fbmEmbeddingProviderId
  saveFbmConfig()
  const providerChanged = oldProviderId !== fbmConfig.providerId
  const embeddingChanged = oldEmbeddingProviderId !== fbmConfig.embeddingProviderId

  if (fbmConfig.enabled && !wasEnabled) {
    await fbmStore.ensureInit()
  } else if (!fbmConfig.enabled && wasEnabled) {
    await fbmStore.shutdown()
  } else if (fbmConfig.enabled && wasEnabled && (providerChanged || embeddingChanged)) {
    await fbmStore.reinitialize()
  }
  await refreshStats()
  showFbmConfig.value = false
}

async function loadMemories() {
  try {
    memoryItems.value = []
    if (!fbmStore.isReady()) return

    const directoryItems = await fbmStore.listDirectory()
    for (const item of directoryItems) {
      const preview = item.summary || item.keywords?.join(', ') || ''
      memoryItems.value.push({
        title: item.title,
        path: item.blockId,
        preview,
        content: preview,
      })
    }

    memoryItems.value.sort((a, b) => a.title.localeCompare(b.title))
  } catch (err) {
    console.error('Failed to load memories:', err)
  }
}

async function refreshStats() {
  loadSettingsToConfig()
  const s = await fbmStore.getMemoryStats()
  stats.totalMemories = s.totalMemories
  stats.vectorCount = s.vectorCount
  stats.enabled = s.enabled
}

async function handleRefreshStats() {
  isRefreshingStats.value = true
  try {
    await loadMemories()
    await refreshStats()
  } finally {
    isRefreshingStats.value = false
  }
}

async function openDetail(item: MemoryItem) {
  detailItem.value = item
  try {
    const blockData = await fbmStore.getBlockDetail(item.path)
    detailContent.value = blockData || item.content || '无法读取内容'
  } catch {
    detailContent.value = item.content || '无法读取内容'
  }
  showDetail.value = true
}

function confirmDelete(item: MemoryItem) {
  deletingItem.value = item
  showDeleteConfirm.value = true
}

async function handleDelete() {
  if (!deletingItem.value) return
  isDeleting.value = true
  try {
    await fbmStore.deleteBlock(deletingItem.value.path)
    showDeleteConfirm.value = false
    deletingItem.value = null
    await loadMemories()
    await refreshStats()
  } catch (err) {
    console.error('Delete memory failed:', err)
  } finally {
    isDeleting.value = false
  }
}

async function handleReindex() {
  showReindexConfirm.value = true
}

async function doReindex() {
  showReindexConfirm.value = false
  isReindexing.value = true
  reindexResult.value = null
  try {
    const count = await fbmStore.reindexVectors()
    reindexResult.value = `索引重建完成，共 ${count} 条向量`
    await refreshStats()
  } catch (err) {
    reindexResult.value = '索引重建失败'
    console.error('Reindex failed:', err)
  } finally {
    isReindexing.value = false
  }
}

async function handleClearVectors() {
  showClearConfirm.value = true
}

async function doClearVectors() {
  showClearConfirm.value = false
  isClearing.value = true
  reindexResult.value = null
  try {
    await fbmStore.clearVectors()
    reindexResult.value = '向量已全部清空'
    await refreshStats()
  } catch (err) {
    reindexResult.value = '清空向量失败'
    console.error('Clear vectors failed:', err)
  } finally {
    isClearing.value = false
  }
}

async function handleConsolidate() {
  isConsolidating.value = true
  consolidateResult.value = ''
  try {
    const result = await fbmStore.consolidate([])
    if (result) {
      consolidateResult.value = `整合完成: 创建 ${result.created} 条, 更新 ${result.updated} 条, 删除 ${result.deleted} 条, 跳过 ${result.skipped} 条`
      await loadMemories()
      await refreshStats()
    } else {
      consolidateResult.value = '整合完成，未提取到新记忆'
    }
  } catch (e) {
    consolidateResult.value = `整合失败: ${e instanceof Error ? e.message : String(e)}`
  } finally {
    isConsolidating.value = false
  }
}

watch(showFbmConfig, (val) => {
  if (val) loadSettingsToConfig()
})

onMounted(async () => {
  loadSettingsToConfig()
  await fbmStore.ensureInit()
  await refreshStats()
  await loadMemories()
})
</script>

<style scoped>
.agent-memory-page {
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
  flex-shrink: 0;
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

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
  padding: 6px 14px;
  border-radius: 20px;
  background: var(--color-surface-secondary);
}

.status-indicator.active {
  color: var(--color-accent-success);
  background: var(--color-bg-success);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-border-medium);
}

.status-indicator.active .status-dot {
  background: var(--color-accent-success);
  box-shadow: 0 0 6px rgba(39, 174, 96, 0.4);
}

.config-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.config-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-alpha-06);
}

.page-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
}

.config-summary {
  display: flex;
  gap: 0;
  border: 1px solid var(--color-border);
  border-radius: 14px;
  background: var(--color-surface);
  margin-bottom: 20px;
  overflow: hidden;
}

.summary-row {
  flex: 1;
  padding: 14px 20px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  border-right: 1px solid var(--color-border);
}

.summary-row:last-child {
  border-right: none;
}

.summary-label {
  font-size: 11px;
  color: var(--color-text-muted);
  font-weight: 600;
}

.summary-value {
  font-size: 13px;
  color: var(--color-text-primary);
  font-weight: 700;
}

.stats-bar {
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 18px 20px;
  border: 1px solid var(--color-border);
  border-radius: 14px;
  background: var(--color-surface);
  margin-bottom: 20px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.stat-value {
  font-size: 22px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.stat-label {
  font-size: 11px;
  color: var(--color-text-muted);
}

.stat-divider {
  width: 1px;
  height: 36px;
  background: var(--color-border);
}

.stat-actions {
  display: flex;
  gap: 8px;
  margin-left: auto;
}

.stat-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 6px 12px;
  border: 1px solid var(--color-border);
  background: var(--color-surface);
  border-radius: 8px;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  white-space: nowrap;
}

.stat-btn:hover:not(:disabled) {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.stat-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.stat-btn.danger {
  color: var(--color-accent-error);
  border-color: var(--color-accent-error-alpha-30);
}

.stat-btn.danger:hover:not(:disabled) {
  background: var(--color-accent-error-alpha-08);
}

.search-bar {
  position: relative;
  margin-bottom: 20px;
}

.search-icon {
  position: absolute;
  left: 14px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--color-text-muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 12px 16px 12px 40px;
  border: 1px solid var(--color-border);
  border-radius: 12px;
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-surface);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-sizing: border-box;
}

.search-input.has-value {
  padding-right: 36px;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary);
  background: var(--color-surface-card);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.search-input::placeholder {
  color: var(--color-text-muted);
}

.search-clear {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 28px;
  height: 28px;
  border: none;
  background: var(--color-surface-secondary);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.2s ease;
}

.search-clear:hover {
  background: var(--color-surface-tertiary);
  color: var(--color-text-primary);
}

.search-status {
  margin-top: -12px;
  margin-bottom: 12px;
  padding: 0 4px;
}

.search-status span {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted);
}

.memory-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 28px;
}

.memory-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px 20px;
  border: 1px solid var(--color-border);
  border-radius: 14px;
  background: var(--color-surface);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.memory-card:hover {
  border-color: var(--color-border-light);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px var(--color-shadow-alpha-08);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  margin-right: 12px;
}

.delete-btn {
  width: 30px;
  height: 30px;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
}

.delete-btn:hover {
  color: var(--color-accent-error);
  background: var(--color-danger-bg);
}

.card-preview {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-muted);
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.no-results {
  text-align: center;
  padding: 40px 24px;
  color: var(--color-text-muted);
}

.no-results p {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
}

.empty-state {
  text-align: center;
  padding: 60px 24px;
  color: var(--color-text-muted);
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
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.empty-state span {
  font-size: 13px;
}

.result-msg {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
  padding: 10px 14px;
  border-radius: 10px;
  background: var(--color-accent-success-alpha-08);
  color: var(--color-accent-success);
  font-size: 13px;
  font-weight: 600;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.fbm-config {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.config-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.config-section-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.toggle-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  border: 1px solid var(--color-border);
  border-radius: 12px;
  background: var(--color-surface);
}

.toggle-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toggle-label {
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.toggle-desc {
  font-size: 11px;
  color: var(--color-text-muted);
}

.toggle-switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  flex-shrink: 0;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-border-medium);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: 24px;
}

.toggle-slider::before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background: var(--color-text-inverse);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: 50%;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
}

.toggle-switch input:checked + .toggle-slider {
  background: var(--color-primary-gradient);
}

.toggle-switch input:checked + .toggle-slider::before {
  transform: translateX(20px);
}

.form-group {
  display: flex;
  flex-direction: column;
}

.form-label {
  font-size: 12px;
  font-weight: 700;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
  display: block;
}

.text-input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  font-size: 13px;
  color: var(--color-text-primary);
  background: var(--color-surface);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-sizing: border-box;
}

.text-input:focus {
  outline: none;
  border-color: var(--color-primary);
  background: var(--color-surface-card);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.text-input::placeholder {
  color: var(--color-text-muted);
}

.detail-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.detail-path {
  font-size: 11px;
  color: var(--color-text-muted);
  word-break: break-all;
}

.detail-body {
  margin: 0;
  padding: 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  background: var(--color-surface);
  font-size: 13px;
  line-height: 1.7;
  color: var(--color-text-primary);
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 50vh;
  overflow-y: auto;
  font-family: inherit;
}

.confirm-content {
  text-align: center;
}

.confirm-content p {
  font-size: 14px;
  color: var(--color-text-primary);
  margin-bottom: 8px;
}

.confirm-content span {
  font-size: 12px;
  color: var(--color-text-muted);
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
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
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
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 16px var(--color-shadow-primary-hover);
}

.save-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px var(--color-shadow-primary-strong);
}

.save-btn.danger {
  background: linear-gradient(135deg, var(--color-accent-error) 0%, var(--color-accent-dark-red) 100%);
  box-shadow: 0 4px 16px var(--color-shadow-error);
}

.save-btn.danger:hover:not(:disabled) {
  box-shadow: 0 6px 20px var(--color-shadow-error-hover);
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}
</style>
