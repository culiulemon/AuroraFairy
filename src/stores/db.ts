const DB_NAME = 'aurorafairy'
const DB_VERSION = 6

let dbInstance: IDBDatabase | null = null

export function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    if (dbInstance) {
      console.log('[DB] 数据库已打开，返回缓存实例')
      resolve(dbInstance)
      return
    }

    console.log('[DB] 打开数据库:', DB_NAME, '版本:', DB_VERSION)
    const request = indexedDB.open(DB_NAME, DB_VERSION)

    request.onerror = () => {
      console.error('[DB] 无法打开数据库:', request.error)
      reject(new Error('无法打开数据库'))
    }

    request.onsuccess = () => {
      dbInstance = request.result
      console.log('[DB] 数据库打开成功')
      resolve(dbInstance)
    }

    request.onupgradeneeded = (event) => {
      console.log('[DB] 数据库升级:', event.oldVersion, '->', DB_VERSION)
      const db = (event.target as IDBOpenDBRequest).result
      const tx = (event.target as IDBOpenDBRequest).transaction!

      if (!db.objectStoreNames.contains('conversations')) {
        const store = db.createObjectStore('conversations', { keyPath: 'id' })
        store.createIndex('updatedAt', 'updatedAt', { unique: false })
        console.log('[DB] 创建 conversations store')
      }

      if (!db.objectStoreNames.contains('attachments')) {
        db.createObjectStore('attachments', { keyPath: 'id' })
        console.log('[DB] 创建 attachments store')
      }

      if (!db.objectStoreNames.contains('tools')) {
        const toolsStore = db.createObjectStore('tools', { keyPath: 'id' })
        toolsStore.createIndex('updatedAt', 'updatedAt', { unique: false })
        toolsStore.createIndex('filePath', 'filePath', { unique: false })
        console.log('[DB] 创建 tools store')
      } else if (tx.objectStore('tools').indexNames.contains('filePath')) {
        console.log('[DB] tools store 已有 filePath 索引')
      } else {
        const toolsStore = tx.objectStore('tools')
        toolsStore.createIndex('filePath', 'filePath', { unique: false })
        console.log('[DB] 为 tools store 添加 filePath 索引')
      }
    }
  })
}

export function getDB(): IDBDatabase {
  if (!dbInstance) {
    throw new Error('数据库未初始化，请先调用 openDB()')
  }
  return dbInstance
}

export async function debugCheckIndexedDB(): Promise<void> {
  if (!import.meta.env.DEV) return
  
  console.log('[DB] ========== IndexedDB 调试检查 ==========')
  
  if (!dbInstance) {
    console.log('[DB] 数据库未初始化')
    return
  }
  
  console.log('[DB] 数据库名:', dbInstance.name)
  console.log('[DB] 数据库版本:', dbInstance.version)
  console.log('[DB] Object Stores:', Array.from(dbInstance.objectStoreNames))
  
  const transaction = dbInstance.transaction(['conversations'], 'readonly')
  const store = transaction.objectStore('conversations')
  
  const countRequest = store.count()
  countRequest.onsuccess = () => {
    console.log('[DB] conversations store 中共有', countRequest.result, '条记录')
  }
  
  const getAllRequest = store.getAll()
  getAllRequest.onsuccess = () => {
    const conversations = getAllRequest.result
    console.log('[DB] 对话列表:', conversations.map((c: any) => ({
      id: c.id,
      title: c.title,
      messageCount: c.messageCount,
      updatedAt: c.updatedAt
    })))
    console.log('[DB] ========== 检查结束 ==========')
  }
}
