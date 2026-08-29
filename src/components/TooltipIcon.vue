<script setup lang="ts">
defineProps<{ tooltip: string; btnClass?: string[] | string }>();
</script>

<template>
  <span class="tooltip-wrapper" :class="btnClass">
    <i class="mdi mdi-information-outline tooltip-icon"></i>
    <span class="tooltip-bubble">{{ tooltip }}</span>
  </span>
</template>

<style scoped>
.tooltip-wrapper {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: help;
  color: var(--text-sub);
  transition: color 0.15s;
}

.tooltip-wrapper:hover {
  color: var(--text-muted);
}

.tooltip-icon {
  font-size: 15px;
}

.tooltip-bubble {
  visibility: hidden;
  opacity: 0;
  position: absolute;
  bottom: 125%;
  left: 50%;
  transform: translateX(-50%);
  background-color: #1e2433;
  color: #e2e8f0;
  text-align: left;
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 12px;
  line-height: 1.4;
  white-space: normal;
  width: max-content;
  max-width: 280px;
  z-index: 100;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.4);
  border: 1px solid #334155;
  transition: opacity 0.15s, visibility 0.15s;
  pointer-events: none;
  font-weight: 400;
}

.tooltip-bubble::after {
  content: "";
  position: absolute;
  top: 100%;
  left: 50%;
  margin-left: -4px;
  border-width: 4px;
  border-style: solid;
  border-color: #1e2433 transparent transparent transparent;
}

.tooltip-wrapper:hover .tooltip-bubble {
  visibility: visible;
  opacity: 1;
}
</style>
