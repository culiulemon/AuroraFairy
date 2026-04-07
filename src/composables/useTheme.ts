import { ref, onMounted } from 'vue'
import { loadThemeSettings, saveThemeSettings, themeColorPresets, type ThemeName } from '../stores/settings'

export type { ThemeName }

const theme = ref<ThemeName>('light')
const primaryColor = ref('#E67E22')

export function initThemeSettings() {
  const settings = loadThemeSettings()
  theme.value = settings.theme
  primaryColor.value = settings.primaryColor

  document.documentElement.setAttribute('data-theme', settings.theme)
  applyPrimaryColor(settings.primaryColor)
}

function lighten(hex: string, percent: number): string {
  const num = parseInt(hex.replace('#', ''), 16)
  const amt = Math.round(2.55 * percent)
  const R = Math.min(255, (num >> 16) + amt)
  const G = Math.min(255, ((num >> 8) & 0x00FF) + amt)
  const B = Math.min(255, (num & 0x0000FF) + amt)
  return '#' + (0x1000000 + R * 0x10000 + G * 0x100 + B).toString(16).slice(1)
}

function darken(hex: string, percent: number): string {
  const num = parseInt(hex.replace('#', ''), 16)
  const amt = Math.round(2.55 * percent)
  const R = Math.max(0, (num >> 16) - amt)
  const G = Math.max(0, ((num >> 8) & 0x00FF) - amt)
  const B = Math.max(0, (num & 0x0000FF) - amt)
  return '#' + (0x1000000 + R * 0x10000 + G * 0x100 + B).toString(16).slice(1)
}

function hexToRgba(hex: string, alpha: number): string {
  const num = parseInt(hex.replace('#', ''), 16)
  const R = (num >> 16) & 255
  const G = (num >> 8) & 255
  const B = num & 255
  return `rgba(${R}, ${G}, ${B}, ${alpha})`
}

function applyPrimaryColor(color: string) {
  const root = document.documentElement
  root.style.setProperty('--color-primary', color)
  root.style.setProperty('--color-primary-hover', darken(color, 10))
  root.style.setProperty('--color-primary-light', lighten(color, 15))
  root.style.setProperty('--color-primary-gradient', `linear-gradient(135deg, ${color} 0%, ${lighten(color, 15)} 100%)`)
  root.style.setProperty('--color-primary-gradient-horizontal', `linear-gradient(90deg, ${color} 0%, ${lighten(color, 15)} 100%)`)
  root.style.setProperty('--color-primary-alpha-04', hexToRgba(color, 0.04))
  root.style.setProperty('--color-primary-alpha-05', hexToRgba(color, 0.05))
  root.style.setProperty('--color-primary-alpha-06', hexToRgba(color, 0.06))
  root.style.setProperty('--color-primary-alpha-08', hexToRgba(color, 0.08))
  root.style.setProperty('--color-primary-alpha-10', hexToRgba(color, 0.10))
  root.style.setProperty('--color-primary-alpha-12', hexToRgba(color, 0.12))
  root.style.setProperty('--color-primary-alpha-15', hexToRgba(color, 0.15))
  root.style.setProperty('--color-primary-alpha-20', hexToRgba(color, 0.20))
  root.style.setProperty('--color-shadow-primary', hexToRgba(color, 0.25))
  root.style.setProperty('--color-shadow-primary-hover', hexToRgba(color, 0.35))
  root.style.setProperty('--color-shadow-primary-strong', hexToRgba(color, 0.45))
}

export function useTheme() {
  const setTheme = (newTheme: ThemeName) => {
    theme.value = newTheme
    document.documentElement.setAttribute('data-theme', newTheme)
    saveThemeSettings({ theme: newTheme, primaryColor: primaryColor.value })
  }

  const setPrimaryColor = (color: string) => {
    primaryColor.value = color
    applyPrimaryColor(color)
    saveThemeSettings({ theme: theme.value, primaryColor: color })
  }

  const toggleTheme = () => {
    setTheme(theme.value === 'light' ? 'dark' : 'light')
  }

  const initTheme = () => {
    initThemeSettings()
  }

  onMounted(() => {
    initThemeSettings()
  })

  return {
    theme,
    primaryColor,
    setTheme,
    setPrimaryColor,
    toggleTheme,
    initTheme,
    themeColorPresets
  }
}
