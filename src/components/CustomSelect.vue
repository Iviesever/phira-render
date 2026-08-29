<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';

interface OptionItem {
  label: string;
  value: any;
}

const props = withDefaults(
  defineProps<{
    modelValue?: any;
    options: Array<OptionItem | string>;
    placeholder?: string;
    disabled?: boolean;
    size?: 'sm' | 'md';
  }>(),
  {
    placeholder: '请选择',
    disabled: false,
    size: 'md',
  }
);

const emit = defineEmits<{
  (e: 'update:modelValue', value: any): void;
  (e: 'change', value: any): void;
}>();

const isOpen = ref(false);
const selectRef = ref<HTMLElement | null>(null);

const normalizedOptions = computed<OptionItem[]>(() => {
  return props.options.map((opt) => {
    if (typeof opt === 'string') {
      return { label: opt, value: opt };
    }
    return opt;
  });
});

const currentLabel = computed(() => {
  const found = normalizedOptions.value.find((opt) => opt.value === props.modelValue);
  return found ? found.label : (props.modelValue ?? props.placeholder);
});

function toggle() {
  if (props.disabled) return;
  isOpen.value = !isOpen.value;
}

function selectOption(opt: OptionItem) {
  emit('update:modelValue', opt.value);
  emit('change', opt.value);
  isOpen.value = false;
}

function handleClickOutside(e: MouseEvent) {
  if (selectRef.value && !selectRef.value.contains(e.target as Node)) {
    isOpen.value = false;
  }
}

onMounted(() => {
  window.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  window.removeEventListener('click', handleClickOutside);
});
</script>

<template>
  <div
    ref="selectRef"
    class="custom-select-wrap"
    :class="{ 'is-open': isOpen, 'is-disabled': disabled, 'size-sm': size === 'sm' }"
  >
    <div class="custom-select-trigger" @click="toggle">
      <span class="selected-text">{{ currentLabel }}</span>
      <i class="mdi mdi-chevron-down select-arrow"></i>
    </div>

    <transition name="dropdown-fade">
      <div v-if="isOpen" class="custom-select-dropdown">
        <div
          v-for="opt in normalizedOptions"
          :key="String(opt.value)"
          class="custom-select-option"
          :class="{ 'is-selected': opt.value === modelValue }"
          @click="selectOption(opt)"
        >
          <span class="option-label">{{ opt.label }}</span>
          <i v-if="opt.value === modelValue" class="mdi mdi-check option-check"></i>
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.custom-select-wrap {
  position: relative;
  width: 100%;
  user-select: none;
  font-size: 13px;
}

.custom-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  padding: 8px 12px;
  color: var(--text-main);
  cursor: pointer;
  transition: all 0.2s ease;
  min-height: 38px;
}

.size-sm .custom-select-trigger {
  padding: 6px 10px;
  min-height: 32px;
  font-size: 13px;
}

.custom-select-trigger:hover {
  border-color: #3b4760;
  background-color: #151924;
}

.is-open .custom-select-trigger {
  border-color: var(--border-focus);
  background-color: var(--bg-input-focus);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.25);
}

.is-disabled .custom-select-trigger {
  opacity: 0.5;
  cursor: not-allowed;
}

.selected-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}

.select-arrow {
  font-size: 18px;
  color: var(--text-sub);
  transition: transform 0.2s ease, color 0.2s ease;
  flex-shrink: 0;
  margin-left: 6px;
}

.is-open .select-arrow {
  transform: rotate(180deg);
  color: var(--primary);
}

/* Floating Dropdown List */
.custom-select-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background-color: #161b26;
  border: 1px solid #2d374d;
  border-radius: var(--radius-sm);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
  max-height: 220px;
  overflow-y: auto;
  z-index: 1200;
  padding: 4px;
}

.custom-select-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 4px;
  color: #e2e8f0;
  cursor: pointer;
  transition: all 0.15s ease;
  font-size: 13px;
}

.custom-select-option:hover {
  background-color: #242c3d;
  color: #ffffff;
}

.custom-select-option.is-selected {
  background-color: rgba(59, 130, 246, 0.2);
  color: var(--primary);
  font-weight: 600;
}

.option-check {
  font-size: 16px;
  color: var(--primary);
}

/* Dropdown Animation */
.dropdown-fade-enter-active,
.dropdown-fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.dropdown-fade-enter-from,
.dropdown-fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
