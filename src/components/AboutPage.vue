<template>
  <div class="about-page">
    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="19" y1="12" x2="5" y2="12"></line>
            <polyline points="12,19 5,12 12,5"></polyline>
          </svg>
        </button>
        <h2>关于</h2>
      </div>
    </div>

    <div class="about-content">
      <div class="app-info-card">
        <div class="app-icon-wrapper">
          <div class="app-icon">
            <svg viewBox="0 0 200 200" width="72" height="72">
              <defs>
                <clipPath id="aboutLogoClip">
                  <rect x="0" y="0" width="200" height="200" rx="100"/>
                </clipPath>
                <filter id="aboutLogoGlow" filterUnits="objectBoundingBox" color-interpolation-filters="sRGB"
                  x="-0.56" y="-0.56" width="2.12" height="2.12">
                  <feFlood flood-opacity="0" result="BackgroundImageFix"/>
                  <feBlend mode="normal" in="SourceGraphic" in2="BackgroundImageFix" result="shape"/>
                  <feGaussianBlur stdDeviation="20.5" result="effect1_foregroundBlur"/>
                </filter>
                <filter id="aboutLogoShadow" filterUnits="objectBoundingBox" color-interpolation-filters="sRGB"
                  x="-0.24" y="-0.24" width="1.49" height="1.49">
                  <feFlood flood-opacity="0" result="BackgroundImageFix"/>
                  <feColorMatrix in="SourceAlpha" type="matrix"
                    values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"/>
                  <feMorphology radius="2" operator="dilate"/>
                  <feOffset dy="0" dx="0"/>
                  <feGaussianBlur stdDeviation="4"/>
                  <feColorMatrix type="matrix"
                    values="0 0 0 0 0 0 0 0 0 0.55 0 0 0 0 0.89 0 0 0 1 0"/>
                  <feBlend mode="normal" in2="BackgroundImageFix" result="effect1_dropShadow"/>
                  <feBlend mode="normal" in="SourceGraphic" in2="effect1_dropShadow" result="shape"/>
                </filter>
                <filter id="aboutLogoInner" filterUnits="objectBoundingBox" color-interpolation-filters="sRGB"
                  x="-1.16" y="-1.16" width="3.11" height="3.11">
                  <feFlood flood-opacity="0" result="BackgroundImageFix"/>
                  <feColorMatrix in="SourceAlpha" type="matrix"
                    values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"/>
                  <feOffset dy="-2" dx="-2"/>
                  <feGaussianBlur stdDeviation="5"/>
                  <feColorMatrix type="matrix"
                    values="0 0 0 0 1 0 0 0 0 1 0 0 0 0 1 0 0 0 0.6 0"/>
                  <feBlend mode="normal" in2="BackgroundImageFix" result="effect1_dropShadow"/>
                  <feBlend mode="normal" in="SourceGraphic" in2="effect1_dropShadow" result="shape"/>
                </filter>
              </defs>
              <g clip-path="url(#aboutLogoClip)">
                <rect x="0" y="0" width="200" height="200" rx="100" fill="#000"/>
                <ellipse class="about-logo-outer-glow" cx="100" cy="100" rx="73" ry="73" fill="#0050C8" filter="url(#aboutLogoGlow)"/>
                <ellipse class="about-logo-outer-ring" cx="100" cy="100" rx="65" ry="65" fill="#ACFFFF" opacity="0.2"/>
                <ellipse class="about-logo-outer-ring" cx="100" cy="100" rx="56" ry="56" fill="#213B69"/>
                <g class="about-logo-star">
                  <path fill-rule="evenodd" fill="#0E2958"
                    d="M192.79,124.64L192.77,124.65C189.89,125.86,186.86,126.81,183.73,127.45L174,134.34L164.27,127.45C161.13,126.81,158.10,125.86,155.21,124.64C149.44,122.20,144.24,118.67,139.87,114.30C135.78,110.20,132.42,105.38,130.00,100.03C129.98,99.99,129.96,99.94,129.94,99.90C129.80,99.59,129.66,99.27,129.53,98.96L129.53,98.95C129.37,98.59,129.23,98.23,129.09,97.87C128.08,95.31,127.28,92.65,126.72,89.90L119.83,80.17L126.72,70.45C126.76,70.25,126.80,70.05,126.84,69.86C127.48,66.93,128.39,64.09,129.53,61.39C129.68,61.03,129.84,60.67,130.00,60.31C132.42,54.97,135.78,50.14,139.87,46.05C144.24,41.68,149.44,38.14,155.21,35.70Q155.22,35.70,155.22,35.70C158.11,34.48,161.14,33.53,164.27,32.89L174,26L183.73,32.89C186.86,33.53,189.89,34.48,192.77,35.70L192.79,35.70C198.56,38.14,203.76,41.68,208.13,46.05C212.49,50.41,216.03,55.61,218.47,61.39Q218.47,61.39,218.47,61.40C219.69,64.28,220.64,67.31,221.28,70.45L228.17,80.17L221.28,89.90C221.28,89.91,221.28,89.92,221.27,89.94C220.63,93.06,219.69,96.07,218.47,98.95L218.47,98.96C218.32,99.32,218.16,99.68,218.00,100.03C215.58,105.38,212.22,110.20,208.13,114.30C204.03,118.39,199.21,121.75,193.86,124.17C193.51,124.33,193.15,124.49,192.79,124.64Z"
                    transform="matrix(0.5,0.866,-0.866,0.5,82.43,-90.77)"/>
                </g>
                <ellipse cx="100" cy="100" rx="37" ry="37" fill="#FFF" filter="url(#aboutLogoShadow)"/>
                <ellipse cx="100" cy="100" rx="26" ry="26" fill="#A0B7E1"/>
                <ellipse cx="100.5" cy="100.5" rx="18.5" ry="18.5" fill="#2D69AA"/>
                <ellipse cx="100.5" cy="100.5" rx="18" ry="18" fill="none" stroke="#BEDDFF" stroke-width="1"/>
                <ellipse cx="100" cy="100" rx="14" ry="14" fill="#173E83"/>
                <ellipse cx="100" cy="100" rx="13.5" ry="13.5" fill="none" stroke="#4A8BD1" stroke-width="1"/>
                <ellipse class="about-logo-highlight" cx="109.5" cy="109.5" rx="9.5" ry="9.5" fill="#FFF" filter="url(#aboutLogoInner)"/>
              </g>
            </svg>
          </div>
        </div>
        <div class="app-name">AuroraFairy</div>
        <div class="app-version">v{{ appVersion }}</div>
        <div class="app-desc">AI 智能体桌面客户端</div>
      </div>

      <div class="info-section">
        <div class="section-header">
          <h3>技术栈</h3>
        </div>
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">前端框架</span>
            <span class="info-value">Vue 3</span>
          </div>
          <div class="info-item">
            <span class="info-label">桌面框架</span>
            <span class="info-value">Tauri v2</span>
          </div>
          <div class="info-item">
            <span class="info-label">构建工具</span>
            <span class="info-value">Vite</span>
          </div>
          <div class="info-item">
            <span class="info-label">后端语言</span>
            <span class="info-value">Rust</span>
          </div>
        </div>
      </div>

      <div class="info-section">
        <div class="section-header">
          <h3>应用信息</h3>
        </div>
        <div class="info-list">
          <div class="info-row">
            <span class="info-row-label">应用标识</span>
            <span class="info-row-value">com.aurorafairy.app</span>
          </div>
          <div class="info-row">
            <span class="info-row-label">构建版本</span>
            <span class="info-row-value">{{ buildVersion }}</span>
          </div>
          <div class="info-row">
            <span class="info-row-label">运行时</span>
            <span class="info-row-value">{{ runtimeInfo }}</span>
          </div>
        </div>
      </div>

      <div class="info-section">
        <div class="section-header">
          <h3>开源鸣谢</h3>
        </div>
        <button class="dev-btn" @click="showOpensource = true">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2L2 7l10 5 10-5-10-5z"/>
            <path d="M2 17l10 5 10-5"/>
            <path d="M2 12l10 5 10-5"/>
          </svg>
          <span>查看开源项目</span>
        </button>
        <p class="dev-desc">本项目基于以下优秀的开源项目构建</p>
      </div>

      <div class="info-section developer-section">
        <div class="section-header">
          <h3>开发者</h3>
        </div>
        <button class="dev-btn" @click="openDebugWindow">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="16,18 22,12 16,6"></polyline>
            <polyline points="8,6 2,12 8,18"></polyline>
          </svg>
          <span>开发者调试窗口</span>
        </button>
        <p class="dev-desc">查看 API 请求/响应日志，用于开发调试</p>
      </div>

      <div class="copyright-section">
        <p>&copy; {{ currentYear }} AuroraFairy. All rights reserved.</p>
      </div>
    </div>

    <OpensourceDialog v-model="showOpensource" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import OpensourceDialog from './OpensourceDialog.vue'

defineEmits<{
  back: []
}>()

const appVersion = ref('0.1.0')
const showOpensource = ref(false)
const buildVersion = ref('dev')
const runtimeInfo = ref('Tauri')
const currentYear = new Date().getFullYear()

let debugWindow: WebviewWindow | null = null

async function openDebugWindow() {
  if (debugWindow) {
    try {
      await debugWindow.setFocus()
      return
    } catch {
      debugWindow = null
    }
  }

  const existing = await WebviewWindow.getByLabel('debug')
  if (existing) {
    await existing.setFocus()
    debugWindow = existing
    return
  }

  const debugUrl = `${window.location.origin}/debug.html`

  debugWindow = new WebviewWindow('debug', {
    title: 'AuroraFairy - 开发者调试窗口',
    url: debugUrl,
    width: 900,
    height: 600,
    minWidth: 600,
    minHeight: 400,
    center: true,
    resizable: true,
    decorations: true
  })

  debugWindow.once('tauri://created', () => {
    console.log('Debug window created')
  })

  debugWindow.once('tauri://error', (e) => {
    console.error('Debug window error:', e)
    debugWindow = null
  })
}

onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = '0.1.0'
  }
  runtimeInfo.value = navigator.userAgent.includes('Windows') ? 'Windows' : 'Unknown'
})
</script>

<style scoped>
.about-page {
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

.about-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
}

.app-info-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px 24px;
  border: 1px solid var(--color-border);
  border-radius: 16px;
  background: var(--color-surface-card);
  margin-bottom: 24px;
}

.app-icon-wrapper {
  margin-bottom: 16px;
}

.app-icon {
  width: 72px;
  height: 72px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.about-logo-outer-glow {
  animation: aboutLogoPulse 3s ease-in-out infinite, aboutLogoScale 4s ease-in-out infinite;
  transform-origin: 100px 100px;
}

.about-logo-outer-ring {
  animation: aboutLogoRingScale 4s ease-in-out infinite;
  transform-origin: 100px 100px;
}

.about-logo-star {
  animation: aboutLogoSpin 20s linear infinite;
  transform-origin: 100px 100px;
}

.about-logo-highlight {
  animation: aboutLogoHighlight 3s ease-in-out infinite;
}

@keyframes aboutLogoPulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.7;
  }
}

@keyframes aboutLogoScale {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.5);
  }
}

@keyframes aboutLogoRingScale {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.08);
  }
}

@keyframes aboutLogoSpin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@keyframes aboutLogoHighlight {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.app-name {
  font-size: 24px;
  font-weight: 800;
  color: var(--color-text-primary);
  letter-spacing: -0.03em;
  margin-bottom: 6px;
}

.app-version {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-primary);
  background: var(--color-primary-alpha-10);
  padding: 3px 12px;
  border-radius: 20px;
  margin-bottom: 8px;
}

.app-desc {
  font-size: 14px;
  color: var(--color-text-muted);
}

.info-section {
  margin-bottom: 24px;
}

.section-header {
  margin-bottom: 16px;
}

.section-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 20px;
  border: 1px solid var(--color-border);
  border-radius: 14px;
  background: var(--color-surface);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.info-item:hover {
  border-color: var(--color-border-light);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px var(--color-shadow-alpha-08);
}

.info-label {
  font-size: 13px;
  color: var(--color-text-muted);
  font-weight: 500;
}

.info-value {
  font-size: 13px;
  color: var(--color-text-primary);
  font-weight: 700;
}

.info-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 20px;
  border: 1px solid var(--color-border);
  border-radius: 14px;
  background: var(--color-surface);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.info-row:hover {
  border-color: var(--color-border-light);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px var(--color-shadow-alpha-08);
}

.info-row-label {
  font-size: 13px;
  color: var(--color-text-muted);
  font-weight: 500;
}

.info-row-value {
  font-size: 13px;
  color: var(--color-text-primary);
  font-weight: 600;
}

.copyright-section {
  text-align: center;
  padding: 20px 0 8px;
  border-top: 1px solid var(--color-border);
  margin-top: 8px;
}

.copyright-section p {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-muted);
}

.developer-section {
  border-top: 1px solid var(--color-border);
  padding-top: 20px;
}

.dev-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 14px 20px;
  border: 1px solid var(--color-border);
  border-radius: 14px;
  background: var(--color-surface);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  color: var(--color-text-primary);
  font-size: 14px;
  font-weight: 600;
}

.dev-btn:hover {
  border-color: var(--color-primary);
  background: var(--color-primary-alpha-06);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px var(--color-shadow-alpha-08);
}

.dev-btn svg {
  color: var(--color-primary);
}

.dev-desc {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--color-text-muted);
  padding-left: 4px;
}
</style>
