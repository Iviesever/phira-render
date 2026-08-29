<i18n>
en:
  not-binded: You have not binded RPE folder yet
  not-binded-sub: Select your RPE install directory to load and render RPE charts.
  bind: Bind RPE Folder
  binded: Binded successfully
  unbind: Unbind RPE
  unbinded: Unbinded successfully
  rpe-folder: Please select RPE's folder

  render: Render Chart

zh-CN:
  not-binded: 尚未绑定 RPE 目录
  not-binded-sub: 绑定 RPE 游戏目录后，可直接在此处浏览并一键渲染已安装的 RPE 谱面。
  bind: 绑定 RPE 目录
  binded: 绑定成功
  unbind: 解除绑定
  unbinded: 解绑成功
  rpe-folder: 请选择 RPE 所在文件夹

  render: 渲染谱面

</i18n>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';

import { toast, toastError } from './common';
import type { RPEChart } from './model';
import router from './router';

async function getRPECharts() {
  return (await invoke('get_rpe_charts')) as RPEChart[] | null;
}
const charts = ref(await getRPECharts());

async function bindRPE() {
  let file = await open({ directory: true, title: t('rpe-folder') });
  if (!file) return;
  try {
    await invoke('set_rpe_dir', { path: file });
    toast(t('binded'), 'success');
    charts.value = await getRPECharts();
  } catch (e) {
    toastError(e);
  }
}
async function unbindRPE() {
  try {
    await invoke('unset_rpe_dir');
    toast(t('unbinded'), 'success');
    charts.value = null;
  } catch (e) {
    toastError(e);
  }
}
</script>

<template>
  <div class="page-wrapper">
    <div class="d-flex justify-between align-center mb-2">
      <h2>RPE</h2>
      <button v-if="charts" class="btn btn-secondary btn-sm" @click="unbindRPE">
        <i class="mdi mdi-link-off"></i> {{ t('unbind') }}
      </button>
    </div>

    <!-- Unbound State -->
    <div v-if="!charts" class="clean-card text-center py-16">
      <i class="mdi mdi-bookshelf empty-icon"></i>
      <h3 class="mt-4">{{ t('not-binded') }}</h3>
      <p class="text-muted mt-1 mb-6">{{ t('not-binded-sub') }}</p>
      <button class="btn btn-primary btn-lg" @click="bindRPE">
        <i class="mdi mdi-folder-open-outline"></i>
        <span>{{ t('bind') }}</span>
      </button>
    </div>

    <!-- Bound Charts List -->
    <div v-else class="charts-grid">
      <div v-for="chart in charts" :key="chart.id" class="clean-card chart-card">
        <div
          class="chart-cover"
          :style="{ 'background-image': 'url(' + convertFileSrc(chart.illustration) + ')' }"
        ></div>
        <div class="chart-info">
          <h3 class="chart-name">{{ chart.name }}</h3>
          <p class="chart-id">{{ chart.id }}</p>
          <div class="chart-actions">
            <button
              class="btn btn-primary btn-sm"
              @click="router.push({ name: 'render', query: { chart: chart.path } })"
            >
              <i class="mdi mdi-auto-fix"></i> {{ t('render') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.empty-icon {
  font-size: 56px;
  color: var(--text-sub);
}

.charts-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.chart-card {
  display: flex;
  padding: 0;
  overflow: hidden;
}

.chart-cover {
  width: 120px;
  min-height: 100px;
  background-position: center;
  background-repeat: no-repeat;
  background-size: cover;
  background-color: #12151c;
  flex-shrink: 0;
  border-right: 1px solid var(--border-color);
}

.chart-info {
  flex: 1;
  padding: 14px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 6px;
}

.chart-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-main);
  line-height: 1.3;
}

.chart-id {
  font-size: 11px;
  color: var(--text-sub);
}

.chart-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 4px;
}

.d-flex { display: flex; }
.justify-between { justify-content: space-between; }
.align-center { align-items: center; }
.mt-1 { margin-top: 4px; }
.mt-4 { margin-top: 16px; }
.mb-2 { margin-bottom: 8px; }
.mb-6 { margin-bottom: 24px; }
.py-16 { padding-top: 64px; padding-bottom: 64px; }
.text-center { text-align: center; }
.text-muted { color: var(--text-muted); }
</style>
