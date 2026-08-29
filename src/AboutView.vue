<i18n>
en:
  app: Phira Render
  desc: A fast, clean, offline chart video rendering & preview tool for Phira
  version: Version
  github: Source Code on GitHub
  license: Licensed under Apache 2.0 / GPLv3

zh-CN:
  app: Phira Render 谱面渲染器
  desc: 高性能、清爽简洁的 Phira 谱面离线视频渲染与练习预览工具
  version: 当前版本
  github: GitHub 开源仓库
  license: 遵循 Apache 2.0 / GPLv3 开源协议

</i18n>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { getVersion } from '@tauri-apps/api/app';
import { open } from '@tauri-apps/api/shell';

const appVersion = await getVersion();
</script>

<template>
  <div class="page-wrapper">
    <div class="clean-card text-center py-12 align-center">
      <div class="app-avatar mb-4">
        <i class="mdi mdi-play-circle-outline"></i>
      </div>
      <h2>{{ t('app') }}</h2>
      <p class="text-muted mt-2">{{ t('desc') }}</p>

      <div class="version-tag mt-4">
        <span>{{ t('version') }}: v{{ appVersion }}</span>
      </div>

      <div class="d-flex gap-4 mt-6">
        <button class="btn btn-secondary" @click="open('https://github.com/TeamFlos/phira-render')">
          <i class="mdi mdi-github"></i> {{ t('github') }}
        </button>
      </div>

      <p class="license-text mt-8">{{ t('license') }}</p>
    </div>
  </div>
</template>

<style scoped>
.app-avatar {
  width: 64px;
  height: 64px;
  border-radius: var(--radius-md);
  background: var(--primary-light);
  color: var(--primary);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 36px;
  margin: 0 auto;
}

.version-tag {
  display: inline-block;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  padding: 4px 12px;
  border-radius: 9999px;
  font-size: 12px;
  color: var(--text-muted);
}

.license-text {
  font-size: 11px;
  color: var(--text-sub);
}

.d-flex { display: flex; }
.gap-4 { gap: 16px; }
.mt-2 { margin-top: 8px; }
.mt-4 { margin-top: 16px; }
.mt-6 { margin-top: 24px; }
.mt-8 { margin-top: 32px; }
.mb-4 { margin-bottom: 16px; }
.py-12 { padding-top: 48px; padding-bottom: 48px; }
.text-center { text-align: center; }
.text-muted { color: var(--text-muted); }
</style>
