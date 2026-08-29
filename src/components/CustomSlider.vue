<script setup lang="ts">
import TooltipIcon from './TooltipIcon.vue';

const props = withDefaults(
  defineProps<{
    modelValue: number;
    min?: number;
    max?: number;
    step?: number;
    label?: string;
    tooltip?: string;
    formatValue?: (val: number) => string;
  }>(),
  {
    min: 0,
    max: 1,
    step: 0.05,
  }
);

const emit = defineEmits<{
  (e: 'update:modelValue', val: number): void;
}>();

function onInput(e: Event) {
  const target = e.target as HTMLInputElement;
  emit('update:modelValue', parseFloat(target.value));
}
</script>

<template>
  <div class="slider-group">
    <div class="slider-header">
      <label v-if="label" class="form-label">
        {{ label }}
        <TooltipIcon v-if="tooltip" :tooltip="tooltip" />
      </label>
      <span class="slider-value">
        {{ formatValue ? formatValue(modelValue) : modelValue }}
      </span>
    </div>
    <div class="slider-track-wrap">
      <input
        type="range"
        class="clean-slider"
        :min="min"
        :max="max"
        :step="step"
        :value="modelValue"
        @input="onInput"
      />
    </div>
  </div>
</template>

<style scoped>
.slider-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 100%;
}

.slider-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.slider-value {
  font-size: 12px;
  font-weight: 600;
  color: var(--primary);
  background: var(--primary-light);
  padding: 2px 6px;
  border-radius: 4px;
}

.slider-track-wrap {
  display: flex;
  align-items: center;
  height: 24px;
}

.clean-slider {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 6px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  outline: none;
  cursor: pointer;
}

.clean-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--primary);
  cursor: pointer;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
  transition: transform 0.1s, background-color 0.1s;
}

.clean-slider::-webkit-slider-thumb:hover {
  transform: scale(1.15);
  background: var(--primary-hover);
}
</style>
