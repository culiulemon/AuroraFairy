<template>
  <div class="nav-sidebar">
    <div class="nav-logo">
      <svg viewBox="0 0 200 200" width="36" height="36">
        <defs>
          <clipPath id="navLogoClip">
            <rect x="0" y="0" width="200" height="200" rx="100"/>
          </clipPath>
          <filter id="navLogoGlow" filterUnits="objectBoundingBox" color-interpolation-filters="sRGB"
            x="-0.56" y="-0.56" width="2.12" height="2.12">
            <feFlood flood-opacity="0" result="BackgroundImageFix"/>
            <feBlend mode="normal" in="SourceGraphic" in2="BackgroundImageFix" result="shape"/>
            <feGaussianBlur stdDeviation="20.5" result="effect1_foregroundBlur"/>
          </filter>
          <filter id="navLogoShadow" filterUnits="objectBoundingBox" color-interpolation-filters="sRGB"
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
          <filter id="navLogoInner" filterUnits="objectBoundingBox" color-interpolation-filters="sRGB"
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
        <g clip-path="url(#navLogoClip)">
          <rect x="0" y="0" width="200" height="200" rx="100" fill="#000"/>
          <ellipse class="nav-logo-outer-glow" cx="100" cy="100" rx="73" ry="73" fill="#0050C8" filter="url(#navLogoGlow)"/>
          <ellipse class="nav-logo-outer-ring" cx="100" cy="100" rx="65" ry="65" fill="#ACFFFF" opacity="0.2"/>
          <ellipse class="nav-logo-outer-ring" cx="100" cy="100" rx="56" ry="56" fill="#213B69"/>
          <g class="nav-logo-star">
            <path fill-rule="evenodd" fill="#0E2958"
              d="M192.79,124.64L192.77,124.65C189.89,125.86,186.86,126.81,183.73,127.45L174,134.34L164.27,127.45C161.13,126.81,158.10,125.86,155.21,124.64C149.44,122.20,144.24,118.67,139.87,114.30C135.78,110.20,132.42,105.38,130.00,100.03C129.98,99.99,129.96,99.94,129.94,99.90C129.80,99.59,129.66,99.27,129.53,98.96L129.53,98.95C129.37,98.59,129.23,98.23,129.09,97.87C128.08,95.31,127.28,92.65,126.72,89.90L119.83,80.17L126.72,70.45C126.76,70.25,126.80,70.05,126.84,69.86C127.48,66.93,128.39,64.09,129.53,61.39C129.68,61.03,129.84,60.67,130.00,60.31C132.42,54.97,135.78,50.14,139.87,46.05C144.24,41.68,149.44,38.14,155.21,35.70Q155.22,35.70,155.22,35.70C158.11,34.48,161.14,33.53,164.27,32.89L174,26L183.73,32.89C186.86,33.53,189.89,34.48,192.77,35.70L192.79,35.70C198.56,38.14,203.76,41.68,208.13,46.05C212.49,50.41,216.03,55.61,218.47,61.39Q218.47,61.39,218.47,61.40C219.69,64.28,220.64,67.31,221.28,70.45L228.17,80.17L221.28,89.90C221.28,89.91,221.28,89.92,221.27,89.94C220.63,93.06,219.69,96.07,218.47,98.95L218.47,98.96C218.32,99.32,218.16,99.68,218.00,100.03C215.58,105.38,212.22,110.20,208.13,114.30C204.03,118.39,199.21,121.75,193.86,124.17C193.51,124.33,193.15,124.49,192.79,124.64Z"
              transform="matrix(0.5,0.866,-0.866,0.5,82.43,-90.77)"/>
          </g>
          <ellipse cx="100" cy="100" rx="37" ry="37" fill="#FFF" filter="url(#navLogoShadow)"/>
          <ellipse cx="100" cy="100" rx="26" ry="26" fill="#A0B7E1"/>
          <ellipse cx="100.5" cy="100.5" rx="18.5" ry="18.5" fill="#2D69AA"/>
          <ellipse cx="100.5" cy="100.5" rx="18" ry="18" fill="none" stroke="#BEDDFF" stroke-width="1"/>
          <ellipse cx="100" cy="100" rx="14" ry="14" fill="#173E83"/>
          <ellipse cx="100" cy="100" rx="13.5" ry="13.5" fill="none" stroke="#4A8BD1" stroke-width="1"/>
          <ellipse class="nav-logo-highlight" cx="109.5" cy="109.5" rx="9.5" ry="9.5" fill="#FFF" filter="url(#navLogoInner)"/>
        </g>
      </svg>
    </div>
    <div class="nav-list">
      <div
        v-for="item in navItems"
        :key="item.id"
        class="nav-item"
        :class="{ active: activeNav === item.id }"
        @click="handleNavClick(item)"
      >
        <div class="nav-icon" v-html="item.icon"></div>
        <span class="nav-label">{{ item.label }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface NavItem {
  id: string
  label: string
  icon: string
  children?: { id: string; label: string }[]
}

defineProps<{
  activeNav: string
}>()

const emit = defineEmits<{
  'update:activeNav': [id: string]
  'select': [item: NavItem]
}>()

const navItems: NavItem[] = [
  {
    id: 'chat',
    label: '聊天',
    icon: '<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>'
  },
  {
    id: 'control',
    label: '控制',
    icon: '<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>'
  },
  {
    id: 'agent',
    label: '智能体',
    icon: '<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2a2 2 0 0 1 2 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 0 1 7 7h1a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1v1a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-1H2a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h1a7 7 0 0 1 7-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 0 1 2-2z"></path><circle cx="8" cy="14" r="1"></circle><circle cx="12" cy="14" r="1"></circle><circle cx="16" cy="14" r="1"></circle></svg>'
  },
  {
    id: 'settings',
    label: '设置',
    icon: '<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"></path><circle cx="12" cy="12" r="3"></circle></svg>'
  }
]

const handleNavClick = (item: NavItem) => {
  emit('update:activeNav', item.id)
  emit('select', item)
}
</script>

<style scoped>
.nav-sidebar {
  width: 64px;
  min-width: 64px;
  height: 100%;
  background: var(--color-surface-card);
  display: flex;
  flex-direction: column;
  padding: 12px 8px;
  box-shadow: 2px 0 12px var(--color-shadow-alpha-06);
  border-right: 1px solid var(--color-border);
}

.nav-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px 0 16px;
  margin-bottom: 8px;
}

.nav-logo svg {
  cursor: pointer;
  transition: transform 0.3s ease;
}

.nav-logo svg:hover {
  transform: scale(1.08);
}

.nav-logo-outer-glow {
  animation: logoPulse 3s ease-in-out infinite, logoScale 4s ease-in-out infinite;
  transform-origin: 100px 100px;
}

.nav-logo-outer-ring {
  animation: logoRingScale 4s ease-in-out infinite;
  transform-origin: 100px 100px;
}

.nav-logo-star {
  animation: logoSpin 20s linear infinite;
  transform-origin: 100px 100px;
}

.nav-logo-highlight {
  animation: logoHighlight 3s ease-in-out infinite;
}

@keyframes logoPulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.7;
  }
}

@keyframes logoScale {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.5);
  }
}

@keyframes logoRingScale {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.08);
  }
}

@keyframes logoSpin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@keyframes logoHighlight {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.nav-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 10px 4px;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  gap: 4px;
}

.nav-item:hover {
  background: var(--color-primary-alpha-10);
}

.nav-item.active:hover {
  background: var(--color-primary-alpha-12);
}

.nav-item.active {
  background: var(--color-primary-alpha-12);
}

.nav-item.active .nav-icon {
  color: var(--color-primary);
}

.nav-item.active .nav-label {
  color: var(--color-primary);
  font-weight: 700;
}

.nav-icon {
  color: var(--color-text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.25s;
}

.nav-label {
  font-size: 10px;
  color: var(--color-text-muted);
  font-weight: 600;
  transition: color 0.25s;
}
</style>
