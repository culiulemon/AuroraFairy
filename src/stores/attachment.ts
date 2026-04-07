import { getDB } from './db'
import type { Attachment } from './conversation'

const attachmentUrlCache = new Map<string, string>()

export function saveAttachment(attachment: Attachment): Promise<void> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['attachments'], 'readwrite')
    const store = transaction.objectStore('attachments')
    
    const request = store.put(attachment)
    
    request.onsuccess = () => resolve()
    request.onerror = () => reject(new Error('保存附件失败'))
  })
}

export function loadAttachment(id: string): Promise<Attachment | null> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['attachments'], 'readonly')
    const store = transaction.objectStore('attachments')
    
    const request = store.get(id)
    
    request.onsuccess = () => resolve(request.result || null)
    request.onerror = () => reject(new Error('加载附件失败'))
  })
}

export async function getAttachmentUrl(id: string): Promise<string | null> {
  if (attachmentUrlCache.has(id)) {
    return attachmentUrlCache.get(id)!
  }
  
  const attachment = await loadAttachment(id)
  if (!attachment) {
    return null
  }
  
  const url = URL.createObjectURL(attachment.data)
  attachmentUrlCache.set(id, url)
  
  return url
}

export function revokeAttachmentUrl(id: string): void {
  const url = attachmentUrlCache.get(id)
  if (url) {
    URL.revokeObjectURL(url)
    attachmentUrlCache.delete(id)
  }
}

export function deleteAttachment(id: string): Promise<void> {
  return new Promise((resolve, reject) => {
    revokeAttachmentUrl(id)
    
    const db = getDB()
    const transaction = db.transaction(['attachments'], 'readwrite')
    const store = transaction.objectStore('attachments')
    
    const request = store.delete(id)
    
    request.onsuccess = () => resolve()
    request.onerror = () => reject(new Error('删除附件失败'))
  })
}

export function deleteAttachmentsForConversation(conversationId: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const db = getDB()
    const transaction = db.transaction(['attachments'], 'readwrite')
    const store = transaction.objectStore('attachments')
    
    const request = store.openCursor()
    
    request.onsuccess = (event) => {
      const cursor = (event.target as IDBRequest<IDBCursorWithValue>).result
      if (cursor) {
        const attachment = cursor.value as Attachment
        if (attachment.id.startsWith(conversationId)) {
          revokeAttachmentUrl(attachment.id)
          cursor.delete()
        }
        cursor.continue()
      } else {
        resolve()
      }
    }
    
    request.onerror = () => reject(new Error('删除对话附件失败'))
  })
}

export function createAttachment(
  conversationId: string,
  file: File
): Promise<Attachment> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    
    reader.onload = () => {
      const attachment: Attachment = {
        id: `${conversationId}-${Date.now()}-${file.name}`,
        filename: file.name,
        mimeType: file.type,
        size: file.size,
        data: new Blob([reader.result as ArrayBuffer], { type: file.type }),
        createdAt: new Date().toISOString()
      }
      
      resolve(attachment)
    }
    
    reader.onerror = () => reject(new Error('读取文件失败'))
    reader.readAsArrayBuffer(file)
  })
}
