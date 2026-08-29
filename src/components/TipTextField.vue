<script setup lang="ts">
import { computed } from 'vue';
import TooltipIcon from './TooltipIcon.vue';

const props = withDefaults(
  defineProps<{
    modelValue?: string | number | null;
    label?: string;
    type?: string;
    placeholder?: string;
    tooltip?: string;
    rules?: Array<(val: any) => boolean | string>;
    disabled?: boolean;
  }>(),
  {
    type: 'text',
    disabled: false,
  }
);

const emit = defineEmits<{
  (e: 'update:modelValue', value: any): void;
}>();

const errorMessage = computed(() => {
  if (!props.rules || props.rules.length === 0) return '';
  const val = props.modelValue == null ? '' : String(props.modelValue);
  for (const rule of props.rules) {
    const res = rule(val);
    if (typeof res === 'string') return res;
    if (res === false) return 'Invalid value';
  }
  return '';
});

function onInput(e: Event) {
  const target = e.target as HTMLInputElement;
  emit('update:modelValue', target.value);
}
</script>

<template>
  <div class="form-group" :class="{ 'has-error': !!errorMessage }">
    <label v-if="label" class="form-label">
      {{ label }}
      <TooltipIcon v-if="tooltip" :tooltip="tooltip" />
    </label>
    <div class="input-wrapper">
      <input
        :type="type"
        class="form-input"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        @input="onInput"
      />
      <slot name="append" />
    </div>
    <span v-if="errorMessage" class="error-text">{{ errorMessage }}</span>
  </div>
</template>

<style scoped>
.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.error-text {
  font-size: 11px;
  color: var(--danger);
  margin-top: 2px;
}

.has-error .form-input {
  border-color: var(--danger);
}
</style>
