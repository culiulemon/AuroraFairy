<template>
  <div class="base-select" ref="containerRef">
    <div
      class="select-trigger"
      :class="{ open: isOpen, placeholder: !modelValue }"
      @click="toggle"
    >
      <span>{{ displayText }}</span>
      <svg class="select-arrow" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="6,9 12,15 18,9"></polyline>
      </svg>
    </div>
    <Teleport to="body">
      <Transition name="dropdown-fade">
        <div
          v-if="isOpen"
          ref="dropdownRef"
          class="select-dropdown"
          :style="dropdownStyle"
        >
          <div
            v-for="option in options"
            :key="option.value ?? ''"
            class="select-option"
            :class="{ active: modelValue === option.value, disabled: option.disabled }"
            @click="!option.disabled && select(option.value)"
            :title="option.disabledReason || ''"
          >
            <span class="option-name">{{ option.label }}</span>
            <span v-if="option.disabledReason && option.disabled" class="option-disabled-reason">{{ option.disabledReason }}</span>
            <span v-else-if="option.sub" class="option-sub">{{ option.sub }}</span>
          </div>
          <div v-if="options.length === 0" class="select-empty">{{ emptyText }}</div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick, watch } from 'vue'

export interface SelectOption {
  value: string | null
  label: string
  sub?: string
  disabled?: boolean
  disabledReason?: string
}

interface Props {
  modelValue: string | null
  options: SelectOption[]
  placeholder?: string
  emptyText?: string
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '-- 请选择 --',
  emptyText: '暂无数据'
})

const emit = defineEmits<{
  'update:modelValue': [value: string | null]
}>()

const isOpen = ref(false)
const containerRef = ref<HTMLElement | null>(null)
const dropdownRef = ref<HTMLElement | null>(null)
const dropdownStyle = ref<Record<string, string>>({})

const displayText = computed(() => {
  if (!props.modelValue) return props.placeholder
  return props.options.find(o => o.value === props.modelValue)?.label ?? props.modelValue
})

function updatePosition() {
  if (!containerRef.value) return
  const rect = containerRef.value.getBoundingClientRect()
  dropdownStyle.value = {
    position: 'fixed',
    top: `${rect.bottom + 6}px`,
    left: `${rect.left}px`,
    width: `${rect.width}px`,
    zIndex: '2000'
  }
}

function toggle() {
  isOpen.value = !isOpen.value
}

function select(value: string | null) {
  emit('update:modelValue', value)
  isOpen.value = false
}

function handleClickOutside(e: MouseEvent) {
  const target = e.target as Node
  const inContainer = containerRef.value?.contains(target)
  const inDropdown = dropdownRef.value?.contains(target)
  if (!inContainer && !inDropdown) {
    isOpen.value = false
  }
}

watch(isOpen, (val) => {
  if (val) {
    nextTick(() => updatePosition())
  }
})

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside)
})
</script>

<style scoped>
.base-select {
  position: relative;
  min-width: 180px;
  max-width: 280px;
  flex-shrink: 0;
}

.select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 11px 14px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  background: var(--color-surface);
  color: var(--color-text-primary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-sizing: border-box;
  font-family: inherit;
  user-select: none;
}

.select-trigger:hover {
  border-color: var(--color-border-light);
  background: var(--color-surface-card);
}

.select-trigger.open {
  border-color: var(--color-primary);
  background: var(--color-surface-card);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.select-trigger.placeholder {
  color: var(--color-text-muted);
}

.select-arrow {
  flex-shrink: 0;
  color: var(--color-text-muted);
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.select-trigger.open .select-arrow {
  transform: rotate(180deg);
  color: var(--color-primary);
}

.select-dropdown {
  background: var(--color-surface-card);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  box-shadow: 0 8px 24px var(--color-shadow-alpha-12), 0 2px 8px var(--color-shadow-alpha-06);
  overflow: hidden;
  max-height: 240px;
  overflow-y: auto;
}

.select-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  cursor: pointer;
  transition: all 0.15s ease;
  border-bottom: 1px solid var(--color-surface-secondary);
}

.select-option:last-child {
  border-bottom: none;
}

.select-option:hover {
  background: var(--color-surface);
}

.select-option.active {
  background: var(--color-primary-alpha-08);
}

.select-option.active .option-name {
  color: var(--color-primary);
  font-weight: 700;
}

.select-option.disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.select-option.disabled:hover {
  background: transparent;
}

.option-disabled-reason {
  font-size: 10px;
  color: var(--color-text-muted);
  font-weight: 400;
  font-style: italic;
}

.option-name {
  font-size: 13px;
  color: var(--color-text-primary);
  font-weight: 600;
}

.option-sub {
  font-size: 11px;
  color: var(--color-text-muted);
  font-weight: 500;
}

.select-empty {
  padding: 20px 14px;
  text-align: center;
  font-size: 12px;
  color: var(--color-text-muted);
}

.dropdown-fade-enter-active,
.dropdown-fade-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.dropdown-fade-enter-from,
.dropdown-fade-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
