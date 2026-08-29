<i18n>
en:
  render: Render
  rpe: RPE
  tasks: Tasks
  about: About

zh-CN:
  render: 渲染
  rpe: RPE
  tasks: 任务列表
  about: 关于

</i18n>

<script lang="ts">
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { activeToasts } from './common';

const onLoaded = ref<() => void>();
const component = ref();

watch(component, (comp) => {
  if (comp && onLoaded.value) onLoaded.value();
});

export function useOnLoaded() {
  return onLoaded;
}

declare global {
  interface Window {
    goto: (name: string) => void;
  }
}

export default {};
</script>

<script setup lang="ts">
const { t } = useI18n();

const route = useRoute(),
  router = useRouter();

const navItems = [
  { key: 'render', icon: 'mdi-auto-fix' },
  { key: 'rpe', icon: 'mdi-bookshelf' },
  { key: 'tasks', icon: 'mdi-server' },
  { key: 'about', icon: 'mdi-information-outline' },
];

window.goto = (name: string) => {
  router.push({ name });
};
</script>

<template>
  <div class="app-container">
    <!-- Toast notifications -->
    <div class="toast-container">
      <div
        v-for="toast in activeToasts"
        :key="toast.id"
        class="toast-item"
        :class="'toast-' + toast.kind"
      >
        <i
          class="mdi"
          :class="{
            'mdi-check-circle': toast.kind === 'success',
            'mdi-alert-circle': toast.kind === 'error',
            'mdi-alert': toast.kind === 'warning',
            'mdi-information': toast.kind === 'info',
          }"
        ></i>
        <span>{{ toast.message }}</span>
      </div>
    </div>

    <!-- Left Clean Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-brand">
        <div class="brand-icon">
          <i class="mdi mdi-play-circle-outline"></i>
        </div>
        <div class="brand-title">Phira Render</div>
      </div>

      <nav class="sidebar-nav">
        <button
          v-for="item in navItems"
          :key="item.key"
          class="nav-item"
          :class="{ 'nav-item-active': route.name === item.key }"
          @click="router.push({ name: item.key })"
        >
          <i class="mdi" :class="item.icon"></i>
          <span class="nav-label">{{ t(item.key) }}</span>
        </button>
      </nav>

      <div class="sidebar-footer">
        <span class="footer-tag">v0.1.0</span>
      </div>
    </aside>

    <!-- Main Views Area -->
    <main class="main-content">
      <router-view v-slot="{ Component }">
        <Suspense timeout="0">
          <template #default>
            <component :is="Component" ref="component" />
          </template>
          <template #fallback>
            <div class="loading-state">
              <i class="mdi mdi-loading mdi-spin"></i>
              <span>加载中...</span>
            </div>
          </template>
        </Suspense>
      </router-view>
    </main>
  </div>
</template>

<style scoped>
.sidebar {
  width: 200px;
  background-color: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  user-select: none;
}

.sidebar-brand {
  padding: 20px 16px;
  display: flex;
  align-items: center;
  gap: 10px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.brand-icon {
  width: 28px;
  height: 28px;
  background: var(--primary-light);
  color: var(--primary);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
}

.brand-title {
  font-size: 14px;
  font-weight: 700;
  color: #ffffff;
  letter-spacing: 0.5px;
}

.sidebar-nav {
  flex: 1;
  padding: 12px 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: left;
}

.nav-item i {
  font-size: 18px;
}

.nav-item:hover {
  background-color: var(--bg-card);
  color: var(--text-main);
}

.nav-item-active {
  background-color: var(--primary-light) !important;
  color: var(--primary) !important;
  font-weight: 600;
}

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
}

.footer-tag {
  font-size: 11px;
  color: var(--text-sub);
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 60vh;
  gap: 12px;
  color: var(--text-muted);
  font-size: 14px;
}

.loading-state i {
  font-size: 32px;
  color: var(--primary);
}
</style>
