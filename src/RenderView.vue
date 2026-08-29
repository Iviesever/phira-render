<i18n>
en:
  already-running: phira-render is already running

  prev-step: Previous
  next-step: Next
  steps:
    choose: '1. Choose Chart'
    config: '2. Chart Info'
    options: '3. Render Options'
    render: '4. Render'

  choose:
    archive: Archive (.zip, .pez)
    folder: Folder
    can-also-drop: You can also drag & drop the chart file into this window
    drop: Drop Chart File Here
    filter-name: Chart Archive

  chart-file: Chart file

  chart-name: Chart Name
  charter: Charter
  composer: Composer
  illustrator: Illustrator
  level: Level
  aspect: Aspect Ratio
  dim: Background Dim

  tip: Tip
  tip-placeholder: Leave empty to choose randomly

  width: Width
  height: Height

  file:
    title: File
    chart: Chart file (empty for default)
    music: Music (empty for default)
    illustration: Illustration (empty for default)

  preview: Preview
  render: Start Render
  export-preview: Export Preview
  export-preview-tips: Export using preview-identical settings (monitor refresh rate, high quality CRF encoding), ignoring configured FPS and bitrate

  render-started: Rendering has started!
  see-tasks: View Task Queue

  ffmpeg-not-found: You haven't installed ffmpeg yet. Please download FFmpeg.exe and put it in the specific folder.

zh-CN:
  already-running: phira-render 已经在运行

  prev-step: 上一步
  next-step: 下一步
  steps:
    choose: '1. 选择谱面'
    config: '2. 谱面信息'
    options: '3. 渲染配置'
    render: '4. 渲染任务'

  choose:
    archive: 压缩包 (.zip, .pez)
    folder: 文件夹
    can-also-drop: 也可以直接拖放谱面文件或文件夹至窗口内
    drop: 释放鼠标以导入谱面
    filter-name: 谱面压缩包

  chart-file: 谱面文件

  chart-name: 谱面名
  charter: 谱师
  composer: 曲师
  illustrator: 画师
  level: 难度
  aspect: 宽高比
  dim: 背景昏暗度

  tip: Tip 提示
  tip-placeholder: 留空则随机选择

  width: 宽
  height: 高

  preview: 谱面预览
  render: 开始渲染
  export-preview: 导出预览
  export-preview-tips: 以预览一致的行为导出（自动检测显示器刷新率作为帧率、CRF 高画质编码），忽略界面上设置的帧数和码率

  render-started: 视频已加入渲染任务队列！
  see-tasks: 查看任务列表

  ffmpeg-not-found: 您尚未安装 FFmpeg。请下载 FFmpeg.exe 并放置在指定文件夹内。

</i18n>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke, event, dialog, shell } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';

import { toastError, RULES, toast, anyFilter, isString } from './common';
import type { ChartInfo } from './model';

import ConfigView from './components/ConfigView.vue';
import TipTextField from './components/TipTextField.vue';
import CustomSlider from './components/CustomSlider.vue';

if (!(await invoke('is_the_only_instance'))) {
  await dialog.message(t('already-running'));
  await invoke('exit_program');
}

const router = useRouter();

const steps = ['choose', 'config', 'options', 'render'];
const stepIndex = ref(1),
  step = computed(() => steps[stepIndex.value - 1]);

const chartInfo = ref<ChartInfo>();
let chartPath = '';

const choosingChart = ref(false),
  parsingChart = ref(false);

async function chooseChart(folder?: boolean) {
  if (choosingChart.value) return;
  choosingChart.value = true;
  let file = folder
    ? await dialog.open({ directory: true })
    : await dialog.open({
        filters: [
          {
            name: t('choose.filter-name'),
            extensions: ['zip', 'pez'],
          },
          anyFilter(),
        ],
      });
  choosingChart.value = false;
  if (!file) return;

  await loadChart(file as string);
}

async function loadChart(file: string) {
  try {
    parsingChart.value = true;
    chartPath = file;
    chartInfo.value = (await invoke('parse_chart', { path: file })) as ChartInfo;
    stepIndex.value = 2;
    aspectWidth.value = String(chartInfo.value.aspectRatio);
    aspectHeight.value = '1.0';
    for (let asp of [
      [16, 9],
      [4, 3],
      [8, 5],
      [3, 2],
    ]) {
      if (Math.abs(asp[0] / asp[1] - chartInfo.value.aspectRatio) < 1e-4) {
        aspectWidth.value = String(asp[0]);
        aspectHeight.value = String(asp[1]);
        break;
      }
    }
  } catch (e) {
    toastError(e);
  } finally {
    parsingChart.value = false;
  }
}

const aspectWidth = ref('16'),
  aspectHeight = ref('9');

const fileHovering = ref(false);
event.listen('tauri://file-drop-hover', (_event) => (fileHovering.value = step.value === 'choose'));
event.listen('tauri://file-drop-cancelled', (_event) => (fileHovering.value = false));
event.listen('tauri://file-drop', async (event) => {
  if (step.value === 'choose') {
    fileHovering.value = false;
    await loadChart((event.payload as string[])[0]);
  }
});

const configView = ref<InstanceType<typeof ConfigView>>();

async function buildParams() {
  if (!configView.value) return null;
  let config = await configView.value.buildConfig();
  if (!config) return null;
  if (!chartInfo.value!.tip?.trim().length) chartInfo.value!.tip = null;
  return {
    path: chartPath,
    info: chartInfo.value,
    config,
  };
}

async function postRender() {
  try {
    if (!(await invoke('test_ffmpeg'))) {
      await dialog.message(t('ffmpeg-not-found'));
      await invoke('open_app_folder');
      await shell.open('https://mivik.moe/ffmpeg-windows/');
      return false;
    }
    let params = await buildParams();
    if (!params) return false;
    await invoke('post_render', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
  }
}

async function previewChart() {
  try {
    let params = await buildParams();
    if (!params) return false;
    await invoke('preview_chart', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
  }
}

async function exportPreview() {
  try {
    if (!(await invoke('test_ffmpeg'))) {
      await dialog.message(t('ffmpeg-not-found'));
      await invoke('open_app_folder');
      await shell.open('https://mivik.moe/ffmpeg-windows/');
      return false;
    }
    let params = await buildParams();
    if (!params) return false;
    const monitorFps = (await invoke('get_refresh_rate')) as number;
    params.config.fps = monitorFps;
    params.config.bitrate = '0';
    params.config.exportPreview = true;
    await invoke('post_render', { params });
    return true;
  } catch (e) {
    toastError(e);
    return false;
  }
}

function validateChartInfo(): boolean {
  if (!chartInfo.value?.name?.trim()) {
    toast('谱面名不能为空', 'error');
    return false;
  }
  if (!chartInfo.value?.level?.trim()) {
    toast('难度不能为空', 'error');
    return false;
  }
  if (!chartInfo.value?.charter?.trim()) {
    toast('谱师不能为空', 'error');
    return false;
  }
  return true;
}

async function moveNext() {
  if (step.value === 'config') {
    if (validateChartInfo()) {
      stepIndex.value = 3;
      if (configView.value) configView.value.onEnter();
    }
    return;
  }
  if (step.value === 'options') {
    if (await postRender()) {
      stepIndex.value = 4;
    }
    return;
  }
}

let chartInQuery = router.currentRoute.value.query.chart;
if (isString(chartInQuery)) {
  onMounted(() => loadChart(chartInQuery as string));
}

function tryParseAspect(): number | undefined {
  try {
    let width = parseFloat(aspectWidth.value);
    let height = parseFloat(aspectHeight.value);
    if (isNaN(width) || isNaN(height)) return undefined;
    return width / height;
  } catch (e) {
    return undefined;
  }
}
</script>

<template>
  <div class="page-wrapper">
    <!-- Clean Step Indicator Header -->
    <div class="steps-nav">
      <div
        v-for="(s, idx) in steps"
        :key="s"
        class="step-nav-item"
        :class="{
          'step-active': stepIndex === idx + 1,
          'step-done': stepIndex > idx + 1,
        }"
      >
        <div class="step-num">{{ idx + 1 }}</div>
        <div class="step-title">{{ t('steps.' + s) }}</div>
      </div>
    </div>

    <!-- Action Bar (for config & options steps) -->
    <div v-if="step === 'config' || step === 'options'" class="action-bar">
      <button class="btn btn-secondary" @click="stepIndex--">
        <i class="mdi mdi-arrow-left"></i> {{ t('prev-step') }}
      </button>
      <div class="flex-1"></div>
      <template v-if="step === 'options'">
        <button class="btn btn-secondary" @click="previewChart">
          <i class="mdi mdi-play-circle-outline"></i> {{ t('preview') }}
        </button>
        <button
          class="btn btn-secondary"
          @click="exportPreview().then(ok => ok && (stepIndex = 4))"
          :title="t('export-preview-tips')"
        >
          <i class="mdi mdi-video-check-outline text-success"></i> {{ t('export-preview') }}
        </button>
      </template>
      <button class="btn btn-primary" @click="moveNext">
        <span>{{ step === 'options' ? t('render') : t('next-step') }}</span>
        <i class="mdi" :class="step === 'options' ? 'mdi-filmstrip' : 'mdi-arrow-right'"></i>
      </button>
    </div>

    <!-- Step 1: Choose chart -->
    <div v-if="step === 'choose'" class="clean-card choose-card">
      <div
        class="drop-zone"
        :class="{ 'drop-zone-hover': fileHovering }"
      >
        <i class="mdi mdi-cloud-upload-outline drop-icon"></i>
        <h3>{{ fileHovering ? t('choose.drop') : t('choose.can-also-drop') }}</h3>
        <p class="drop-hint">支持 Phira 谱面压缩包 (.zip, .pez) 以及未解包文件夹</p>
        
        <div class="d-flex gap-4 mt-6">
          <button class="btn btn-primary btn-lg" @click="chooseChart(false)">
            <i class="mdi mdi-folder-zip-outline"></i>
            <span>{{ t('choose.archive') }}</span>
          </button>
          <button class="btn btn-secondary btn-lg" @click="chooseChart(true)">
            <i class="mdi mdi-folder-outline"></i>
            <span>{{ t('choose.folder') }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Step 2: Chart Info -->
    <div v-if="step === 'config' && chartInfo" class="clean-card">
      <div class="card-title">
        <i class="mdi mdi-information-outline text-primary"></i>
        <span>{{ t('steps.config') }}</span>
      </div>

      <div class="grid-2">
        <TipTextField
          :label="t('chart-name')"
          :rules="[RULES.non_empty]"
          v-model="chartInfo.name"
        />
        <TipTextField
          :label="t('level')"
          :rules="[RULES.non_empty]"
          v-model="chartInfo.level"
        />
      </div>

      <div class="grid-3 mt-4">
        <TipTextField
          :label="t('charter')"
          :rules="[RULES.non_empty]"
          v-model="chartInfo.charter"
        />
        <TipTextField
          :label="t('composer')"
          v-model="chartInfo.composer"
        />
        <TipTextField
          :label="t('illustrator')"
          v-model="chartInfo.illustrator"
        />
      </div>

      <div class="grid-2 mt-4 align-center">
        <div class="form-group mb-0">
          <label class="form-label">{{ t('aspect') }}</label>
          <div class="d-flex align-center gap-2">
            <input type="number" class="form-input" v-model="aspectWidth" placeholder="16" />
            <span>:</span>
            <input type="number" class="form-input" v-model="aspectHeight" placeholder="9" />
          </div>
        </div>

        <CustomSlider
          :label="t('dim')"
          :min="0"
          :max="1"
          :step="0.05"
          v-model="chartInfo.backgroundDim"
          :format-value="(v) => Math.round(v * 100) + '%'"
        />
      </div>

      <div class="mt-4">
        <TipTextField
          :label="t('tip')"
          :placeholder="t('tip-placeholder')"
          v-model="chartInfo.tip"
        />
      </div>
    </div>

    <!-- Step 3: Render Options -->
    <div v-if="step === 'options'">
      <ConfigView ref="configView" :init-aspect-ratio="tryParseAspect()" />
    </div>

    <!-- Step 4: Finished / Render Started -->
    <div v-if="step === 'render'" class="clean-card text-center py-12">
      <div class="success-icon mb-4">
        <i class="mdi mdi-check-circle-outline"></i>
      </div>
      <h2>{{ t('render-started') }}</h2>
      <p class="text-muted mt-2 mb-6">视频渲染任务已提交至后台队列，您可以在任务列表中查看实时进度。</p>
      <div class="d-flex justify-center gap-4">
        <button class="btn btn-secondary" @click="stepIndex = 1">
          <i class="mdi mdi-plus"></i> 继续渲染其他谱面
        </button>
        <button class="btn btn-primary" @click="router.push({ name: 'tasks' })">
          <i class="mdi mdi-server"></i> {{ t('see-tasks') }}
        </button>
      </div>
    </div>

    <!-- Parsing loader modal -->
    <div v-if="parsingChart" class="modal-overlay">
      <div class="modal-content text-center py-8 align-center" style="max-width: 320px">
        <i class="mdi mdi-loading mdi-spin text-primary" style="font-size: 40px"></i>
        <h4>正在解析谱面文件...</h4>
      </div>
    </div>
  </div>
</template>

<style scoped>
.steps-nav {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  padding: 8px 16px;
  gap: 12px;
}

.step-nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-sub);
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s;
}

.step-num {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
}

.step-active {
  color: var(--primary) !important;
  font-weight: 600;
}

.step-active .step-num {
  background: var(--primary);
  color: #ffffff;
  border-color: var(--primary);
}

.step-done {
  color: var(--text-muted);
}

.step-done .step-num {
  background: var(--success-light);
  color: var(--success);
  border-color: var(--success);
}

.action-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  padding: 10px 16px;
  border-radius: var(--radius-sm);
}

.choose-card {
  padding: 32px 20px;
}

.drop-zone {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  border: 2px dashed var(--border-color);
  border-radius: var(--radius-md);
  background: rgba(255, 255, 255, 0.01);
  transition: all 0.2s;
}

.drop-zone-hover {
  border-color: var(--primary);
  background: var(--primary-light);
}

.drop-icon {
  font-size: 56px;
  color: var(--primary);
  margin-bottom: 12px;
}

.drop-hint {
  font-size: 12px;
  color: var(--text-sub);
  margin-top: 6px;
}

.success-icon i {
  font-size: 64px;
  color: var(--success);
}

.d-flex { display: flex; }
.flex-1 { flex: 1; }
.justify-center { justify-content: center; }
.align-center { align-items: center; }
.gap-2 { gap: 8px; }
.gap-4 { gap: 16px; }
.mt-2 { margin-top: 8px; }
.mt-4 { margin-top: 16px; }
.mt-6 { margin-top: 24px; }
.mb-0 { margin-bottom: 0 !important; }
.mb-2 { margin-bottom: 8px; }
.mb-4 { margin-bottom: 16px; }
.mb-6 { margin-bottom: 24px; }
.py-8 { padding-top: 32px; padding-bottom: 32px; }
.py-12 { padding-top: 48px; padding-bottom: 48px; }
.text-center { text-align: center; }
.text-primary { color: var(--primary); }
.text-success { color: var(--success); }
.text-muted { color: var(--text-muted); }
</style>
