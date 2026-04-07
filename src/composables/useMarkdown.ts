import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js/lib/core'
import javascript from 'highlight.js/lib/languages/javascript'
import typescript from 'highlight.js/lib/languages/typescript'
import python from 'highlight.js/lib/languages/python'
import css from 'highlight.js/lib/languages/css'
import json from 'highlight.js/lib/languages/json'
import bash from 'highlight.js/lib/languages/bash'
import xml from 'highlight.js/lib/languages/xml'
import markdown from 'highlight.js/lib/languages/markdown'
import sql from 'highlight.js/lib/languages/sql'

hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('python', python)
hljs.registerLanguage('css', css)
hljs.registerLanguage('json', json)
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('html', xml)
hljs.registerLanguage('markdown', markdown)
hljs.registerLanguage('sql', sql)

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;')
}

const md = new MarkdownIt({
  html: false,
  linkify: true,
  typographer: true,
  breaks: true,
  highlight(str: string, lang: string): string {
    const validLang = lang && hljs.getLanguage(lang) ? lang : ''
    if (validLang) {
      try {
        return '<pre class="hljs-pre"><code class="hljs">' +
          hljs.highlight(str, { language: validLang, ignoreIllegals: true }).value +
          '</code></pre>'
      } catch (_) { /* fall through */ }
    }
    return '<pre class="hljs-pre"><code class="hljs">' + escapeHtml(str) + '</code></pre>'
  }
})

md.renderer.rules.table_open = () => '<div class="table-scroll"><table>'
md.renderer.rules.table_close = () => '</table></div>'

const defaultImageRender = md.renderer.rules.image || function (tokens, idx, options, _env, self) {
  return self.renderToken(tokens, idx, options)
}

md.renderer.rules.image = function (tokens, idx, options, env, self) {
  const token = tokens[idx]
  const srcIndex = token.attrIndex('src')
  if (srcIndex >= 0 && token.attrs) {
    const src = token.attrs[srcIndex][1]
    const alt = token.content || ''
    const escapedAlt = alt.replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    if (src.startsWith('local:///')) {
      const filePath = src.slice(9).replace(/\//g, '\\')
      const escapedPath = filePath.replace(/"/g, '&quot;')
      return `<img src="" alt="${escapedAlt}" class="md-image md-image-local" data-file-path="${escapedPath}" loading="lazy" />`
    }
    return `<img src="${src}" alt="${escapedAlt}" class="md-image" loading="lazy" />`
  }
  return defaultImageRender(tokens, idx, options, env, self)
}

function sanitize(html: string): string {
  const allowedTags = new Set([
    'p', 'br', 'strong', 'em', 'b', 'i', 'u', 's', 'del',
    'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
    'ul', 'ol', 'li', 'a', 'span', 'div',
    'pre', 'code', 'blockquote',
    'table', 'thead', 'tbody', 'tr', 'th', 'td',
    'hr', 'img', 'sup', 'sub', 'input'
  ])
  const allowedAttrs = new Set([
    'href', 'target', 'rel', 'class', 'src', 'alt',
    'type', 'checked', 'disabled', 'align', 'loading',
    'data-file-path'
  ])

  const div = document.createElement('div')
  div.innerHTML = html

  function clean(node: Element) {
    const children = Array.from(node.children)
    for (const child of children) {
      const tag = child.tagName.toLowerCase()
      if (!allowedTags.has(tag)) {
        const frag = document.createDocumentFragment()
        while (child.firstChild) frag.appendChild(child.firstChild)
        node.replaceChild(frag, child)
        continue
      }
      const attrs = Array.from(child.attributes)
      for (const attr of attrs) {
        if (!allowedAttrs.has(attr.name)) {
          child.removeAttribute(attr.name)
        }
      }
      if (tag === 'a') {
        child.setAttribute('target', '_blank')
        child.setAttribute('rel', 'noopener noreferrer')
      }
      clean(child)
    }
  }

  clean(div)
  return div.innerHTML
}

const IMAGE_EXTENSIONS = '(?:png|jpe?g|gif|webp|svg|bmp|ico|avif)'

const LOCAL_IMAGE_PATTERN = new RegExp(
  `([A-Za-z]:\\\\[\\S]+?\\.${IMAGE_EXTENSIONS})` +
  `|` +
  `(/[\\S&&[^\\]\\[(!]]+?\\.${IMAGE_EXTENSIONS})`,
  'gi'
)

const HTTP_IMAGE_PATTERN = new RegExp(
  `(https?:\\/\\/\\S+?\\.${IMAGE_EXTENSIONS})(?=\\s|$|[^\\w.\\-/])`,
  'gi'
)

function preprocessImageUrls(text: string): string {
  return text
    .replace(LOCAL_IMAGE_PATTERN, (_match, winPath: string, unixPath: string) => {
      const rawPath = winPath || unixPath
      return `![图片](local:///${rawPath.replace(/\\/g, '/')})`
    })
    .replace(HTTP_IMAGE_PATTERN, (_match, httpUrl: string) => {
      return `![图片](${httpUrl})`
    })
}

function extractLocalImagePaths(text: string): string[] {
  const paths: string[] = []
  LOCAL_IMAGE_PATTERN.lastIndex = 0
  let match: RegExpExecArray | null
  while ((match = LOCAL_IMAGE_PATTERN.exec(text)) !== null) {
    const raw = match[1] || match[2]
    if (raw) paths.push(raw)
  }
  return paths
}

export { extractLocalImagePaths }

export function renderMarkdown(text: string): string {
  if (!text) return ''
  const withImages = preprocessImageUrls(text)
  const normalized = withImages
    .replace(/([^\n])\n(#{1,6})/g, '$1\n\n$2')
    .replace(/(#{1,6})([^\s#])/g, '$1 $2')
    .replace(/(#{1,6})\s{2,}/g, '$1 ')
  const raw = md.render(normalized)
  return sanitize(raw)
}
