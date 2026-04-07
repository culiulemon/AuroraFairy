import { listen } from '@tauri-apps/api/event'

interface DebugLog {
  id: string
  timestamp: number
  level: 'request' | 'response' | 'error' | 'info'
  title: string
  content: string
  meta?: Record<string, unknown>
}

const logContainer = document.getElementById('logContainer')!
const logCountEl = document.getElementById('logCount')!
const emptyState = document.getElementById('emptyState')!
const clearBtn = document.getElementById('clearBtn')!
const autoScrollCb = document.getElementById('autoScroll') as HTMLInputElement

const filterBtns = document.querySelectorAll<HTMLButtonElement>('.filter-btn')
let currentFilter = 'all'
const allLogs: DebugLog[] = []

function formatTime(ts: number): string {
  const d = new Date(ts)
  return d.toLocaleTimeString('zh-CN', { hour12: false }) + '.' + String(d.getMilliseconds()).padStart(3, '0')
}

function formatMeta(meta?: Record<string, unknown>): string {
  if (!meta) return ''
  return Object.entries(meta)
    .filter(([_, v]) => v !== undefined && v !== null)
    .map(([k, v]) => `${k}: ${v}`)
    .join(' | ')
}

function renderLog(log: DebugLog): HTMLDivElement {
  const entry = document.createElement('div')
  entry.className = 'log-entry'
  entry.dataset.level = log.level
  entry.dataset.id = log.id

  const metaStr = formatMeta(log.meta)

  entry.innerHTML = `
    <div class="log-header">
      <span class="log-badge ${log.level}">${log.level}</span>
      <span class="log-time">${formatTime(log.timestamp)}</span>
      <span class="log-title">${escapeHtml(log.title)}</span>
      ${metaStr ? `<span class="log-meta">${escapeHtml(metaStr)}</span>` : ''}
    </div>
    <div class="log-body">
      <pre class="log-content">${escapeHtml(log.content)}</pre>
    </div>
  `

  const header = entry.querySelector('.log-header') as HTMLElement
  header.addEventListener('click', () => {
    const body = entry.querySelector('.log-body') as HTMLElement
    body.classList.toggle('expanded')
  })

  return entry
}

function escapeHtml(str: string): string {
  const div = document.createElement('div')
  div.textContent = str
  return div.innerHTML
}

function addLog(log: DebugLog) {
  allLogs.push(log)
  updateLogCount()

  if (currentFilter !== 'all' && log.level !== currentFilter) return

  if (emptyState.style.display !== 'none') {
    emptyState.style.display = 'none'
  }

  const el = renderLog(log)
  logContainer.appendChild(el)

  if (autoScrollCb.checked) {
    logContainer.scrollTop = logContainer.scrollHeight
  }
}

function updateLogCount() {
  logCountEl.textContent = `${allLogs.length} 条日志`
}

function applyFilter(filter: string) {
  currentFilter = filter
  const entries = logContainer.querySelectorAll<HTMLDivElement>('.log-entry')
  let visible = 0

  entries.forEach(entry => {
    const level = entry.dataset.level
    const shouldShow = filter === 'all' || level === filter
    entry.style.display = shouldShow ? '' : 'none'
    if (shouldShow) visible++
  })

  if (visible === 0 && allLogs.length > 0) {
    emptyState.style.display = ''
  } else {
    emptyState.style.display = 'none'
  }
}

function clearLogs() {
  allLogs.length = 0
  updateLogCount()
  const entries = logContainer.querySelectorAll('.log-entry')
  entries.forEach(e => e.remove())
  emptyState.style.display = ''
}

filterBtns.forEach(btn => {
  btn.addEventListener('click', () => {
    filterBtns.forEach(b => b.classList.remove('active'))
    btn.classList.add('active')
    applyFilter(btn.dataset.filter || 'all')
  })
})

clearBtn.addEventListener('click', clearLogs)

listen<DebugLog>('debug-log', (event) => {
  addLog(event.payload)
}).catch(err => {
  console.error('Failed to listen debug-log:', err)
})
