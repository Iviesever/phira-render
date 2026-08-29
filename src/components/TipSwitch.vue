<script setup lang="ts">
import TooltipIcon from './TooltipIcon.vue';

const props = defineProps<{
  modelValue?: boolean;
  label: string;
  tooltip?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

function toggle() {
  emit('update:modelValue', !props.modelValue);
}
</script>

<template>
  <div class="tip-switch-container" @click="toggle">
    <div class="switch-pill" :class="{ 'switch-active': modelValue }">
      <div class="switch-thumb"></div>
    </div>
    <span class="switch-label">
      {{ label }}
      <TooltipIcon v-if="tooltip" :tooltip="tooltip" class="ml-1" @click.stop />
    </span>
  </div>
</template>

<style scoped>
.tip-switch-container {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  user-select: none;
  padding: 4px 0;
}

.switch-pill {
  width: 38px;
  height: 20px;
  background-color: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 9999px;
  position: relative;
  transition: background-color 0.2s, border-color 0.2s;
  flex-shrink: 0;
}

.switch-thumb {
  width: 14px;
  height: 14px;
  background-color: var(--text-muted);
  border-radius: 50%;
  position: absolute;
  top: 2px;
  left: 2px;
  transition: transform 0.2s, background-color 0.2s;
}

.switch-active {
  background-color: var(--primary);
  border-color: var(--primary);
}

.switch-active .switch-thumb {
  transform: translateX(18px);
  background-color: #ffffff;
}

.switch-label {
  font-size: 13px;
  color: var(--text-main);
  display: flex;
  align-items: center;
  gap: 4px;
}

.ml-1 {
  margin-left: 4px;
}
</style>
