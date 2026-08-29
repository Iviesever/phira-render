<i18n>
en:
  title:
    output: Video Output
    player: Player Info
    graphics: Graphics & Mods
    audio: Audio & Result
    presets: Presets

  resolution: Resolution
  fps: FPS
  get-monitor-fps: Screen FPS

  hw-accel: Hardware Acceleration
  hw-accel-tips: If render fails, try to turn it off

  fxaa: FXAA
  fxaa-tips: FXAA, as a low-cost anti-aliasing method, will cause the picture to be blurred, and it is not recommended to turn it on

  sample-count: Sample Count (MSAA)
  sample-count-tips: Must be a power of 2. A non-1 sample count enables MSAA, which can improve the quality of the picture while increasing the performance cost

  bitrate: Bitrate
  bitrate-tips: A higher bitrate will result in higher quality and larger file size

  export-path: Custom Export Path
  export-path-title: Choose Export Directory
  export-path-tips: Custom video export path (folder or file path). If empty, default app directory will be used. Supports pasted path with quotes.
  choose-folder: Browse

  player-avatar: Player Avatar
  player-name: Player Name
  player-rks: Player Rks.

  image-filter: Image

  challenge-color: Challenge Mode Color
  challenge-colors: White,Green,Blue,Red,Golden,Rainbow

  challenge-rank: Challenge Mode Rank

  respack: Resource Pack
  respack-default: '[Default]'
  respack-refresh: Refresh
  respack-open: Open Folder

  note-scale: Note Scale

  double-hint: Double Hit Hint

  aggressive: Aggressive Optimization
  aggressive-tips: Improve rendering speed, but may cause some notes to disappear

  disable-particle: Disable Particle
  disable-effect: Disable Effect

  volume-music: Music Volume
  volume-sfx: SFX Volume

  ending-length: Result Screen Duration (s)

  show-player: Show Player & Result
  show-player-tips: When off, player info is not needed and the ending score screen is skipped

  preset-refresh: Refresh
  preset-create: New
  preset-create-title: Preset name
  preset-created: Preset created
  preset-delete: Delete
  preset-deleted: Preset deleted
  preset-replace: Save
  preset-replaced: Preset saved
  preset-cannot-use-default: Cannot use 'default' as preset name
  default-preset: Default

zh-CN:
  title:
    output: 视频输出
    player: 玩家信息
    graphics: 图像与特效
    audio: 音频与结算
    presets: 预设管理

  resolution: 分辨率
  fps: 帧率 (FPS)
  get-monitor-fps: 获取刷新率

  hw-accel: 硬件加速
  hw-accel-tips: 如果渲染失败，请尝试关闭此选项

  fxaa: FXAA 抗锯齿
  fxaa-tips: FXAA 以低成本实现抗锯齿，但会导致画面模糊，不建议开启

  sample-count: 采样数 (MSAA)
  sample-count-tips: 必须为 2 的幂。非 1 的采样数会启用 MSAA，提高画面质量的同时会在一定程度上增加性能开销

  bitrate: 码率
  bitrate-tips: 码率越高，画面质量越高，文件大小也越大

  export-path: 视频导出路径
  export-path-title: 选择导出文件夹
  export-path-tips: 自定义视频导出路径（文件夹或文件路径）。留空则使用默认。支持直接粘贴带双引号的路径。
  choose-folder: 浏览

  player-avatar: 玩家头像
  player-name: 玩家名
  player-rks: 玩家 RKS

  image-filter: 图像

  challenge-color: 课题模式颜色
  challenge-colors: 白,绿,蓝,红,金,彩

  challenge-rank: 课题模式等级

  respack: 资源包
  respack-default: '[默认]'
  respack-refresh: 刷新
  respack-open: 打开文件夹

  note-scale: 音符缩放

  double-hint: 双押提示

  aggressive: 激进优化
  aggressive-tips: 提升渲染速度，但可能会导致部分音符消失

  disable-particle: 禁用粒子
  disable-effect: 禁用特效

  volume-music: 音乐音量
  volume-sfx: 音效音量

  ending-length: 结算画面时长 (秒)

  show-player: 显示玩家与结算画面
  show-player-tips: 关闭后无需填写玩家信息，且视频末尾不会有结算画面

  preset-refresh: 刷新
  preset-create: 新建
  preset-create-title: 预设配置名
  preset-created: 预设配置已创建
  preset-delete: 删除
  preset-deleted: 预设配置已删除
  preset-replace: 保存覆盖
  preset-replaced: 预设配置已保存
  preset-cannot-use-default: 不能使用 'default' 作为配置名
  default-preset: 默认

</i18n>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import { invoke } from '@tauri-apps/api';
import { open } from '@tauri-apps/api/dialog';

import { RULES, isNumeric, toast, anyFilter, toastError } from '../common';
import type { RenderConfig } from '../model';

import TipSwitch from './TipSwitch.vue';
import TipTextField from './TipTextField.vue';
import CustomSlider from './CustomSlider.vue';
import TooltipIcon from './TooltipIcon.vue';

const props = defineProps<{ initAspectRatio?: number }>();

const RESOLUTIONS = ['1920x1080', '2560x1440', '3840x2160', '1280x720', '960x540', '800x600'];

function parseResolution(resolution: string): [number, number] | null {
  let parts = resolution.split(/[xX]/g);
  if (parts.length !== 2) return null;
  let ws = parts[0].trim(),
    hs = parts[1].trim();
  if (!isNumeric(ws) || !isNumeric(hs)) return null;
  let w = parseInt(ws),
    h = parseInt(hs);
  if (w <= 0 || h <= 0) return null;
  return [w, h];
}
const resolutionRule = (value: string) => parseResolution(value) !== null || t('rules.resolution');
const sampleCountRule = (value: string) => (isNumeric(value) && Math.log2(Number(value)) % 1 === 0) || t('rules.sample-count');

const resolution = ref('1920x1080'),
  fps = ref('60'),
  hwAccel = ref(true);

const fxaa = ref(false),
  sampleCount = ref('4'),
  bitrate = ref('7M'),
  exportPath = ref<string | null>(localStorage.getItem('lastExportPath') || null);

watch(exportPath, (newVal) => {
  if (newVal) {
    let cleaned = newVal.trim().replace(/^['"]|['"]$/g, '').trim();
    if (cleaned !== newVal) {
      exportPath.value = cleaned;
      return;
    }
    if (cleaned.length > 0) {
      localStorage.setItem('lastExportPath', cleaned);
    } else {
      localStorage.removeItem('lastExportPath');
    }
  } else {
    localStorage.removeItem('lastExportPath');
  }
});

async function chooseExportPath() {
  let dir = await open({
    directory: true,
    title: t('export-path-title'),
    defaultPath: exportPath.value || undefined,
  });
  if (dir) {
    exportPath.value = dir as string;
  }
}

async function fillMonitorFps() {
  try {
    const rate = (await invoke('get_refresh_rate')) as number;
    if (rate > 0) {
      fps.value = String(rate);
      toast(`已填入屏幕刷新率: ${rate} FPS`, 'info');
    }
  } catch (e) {
    toastError(e);
  }
}

const playerAvatar = ref<string>(),
  playerName = ref(''),
  playerRks = ref('15.0');

async function chooseAvatar() {
  let file = await open({
    filters: [
      {
        name: t('image-filter'),
        extensions: ['jpg', 'jpeg', 'png', 'webp', 'bmp'],
      },
      anyFilter(),
    ],
  });
  if (file) {
    playerAvatar.value = file as string;
  }
}

const challengeColor = ref(t('challenge-colors').split(',')[4]),
  challengeRank = ref('45');

interface Respack {
  name: string;
  path: string | null;
  index: number;
}
const DEFAULT_RESPACK: Respack = {
  name: t('respack-default'),
  path: null,
  index: 0,
};
async function getRespacks() {
  return [DEFAULT_RESPACK, ...((await invoke('get_respacks')) as { name: string; path: string }[])].map((obj, index) => ({
    name: obj.name,
    path: obj.path,
    index: index + 1,
  }));
}
const respacks = ref([DEFAULT_RESPACK]);
const respack = ref(DEFAULT_RESPACK);
async function updateRespacks() {
  respacks.value = await getRespacks();
  const lastRespackPath = localStorage.getItem('lastRespackPath');
  if (lastRespackPath) {
    respack.value = respacks.value.find((x) => x.path === lastRespackPath) || respacks.value[0];
  } else {
    respack.value = respacks.value.find((x) => x.name === respack.value.name) || respacks.value[0];
  }
}
updateRespacks();
watch(respack, (val) => {
  localStorage.setItem('lastRespackPath', val.path || '');
});

const noteScale = ref(1);

const doubleHint = ref(true),
  aggressive = ref(false),
  disableParticle = ref(false),
  disableEffect = ref(false);

const volumeMusic = ref(1),
  volumeSfx = ref(1);

const endingLength = ref('25.5');

const showPlayer = ref(false);

const STD_CHALLENGE_COLORS = ['white', 'green', 'blue', 'red', 'golden', 'rainbow'];

function validateForm(): boolean {
  if (parseResolution(resolution.value) === null) {
    toast(t('rules.resolution'), 'error');
    return false;
  }
  if (!RULES.positiveInt(fps.value) || typeof RULES.positiveInt(fps.value) === 'string') {
    toast(t('rules.positive-int'), 'error');
    return false;
  }
  if (!bitrate.value.trim().length) {
    toast(t('rules.non-empty'), 'error');
    return false;
  }
  if (showPlayer.value && !playerName.value.trim().length) {
    toast(t('rules.non-empty'), 'error');
    return false;
  }
  return true;
}

async function buildConfig(): Promise<RenderConfig | null> {
  if (!validateForm()) return null;
  return {
    resolution: (() => {
      let parts = resolution.value.split('x');
      return [parseInt(parts[0]), parseInt(parts[1])];
    })(),
    endingLength: parseFloat(endingLength.value) || 25.5,
    fps: parseInt(fps.value) || 60,
    hardwareAccel: hwAccel.value,
    bitrate: bitrate.value,

    aggressive: aggressive.value,
    challengeColor: STD_CHALLENGE_COLORS[t('challenge-colors').split(',').indexOf(challengeColor.value)] || 'golden',
    challengeRank: parseInt(challengeRank.value) || 45,
    disableEffect: disableEffect.value,
    doubleHint: doubleHint.value,
    fxaa: fxaa.value,
    noteScale: noteScale.value,
    particle: !disableParticle.value,
    playerAvatar: playerAvatar.value ? (playerAvatar.value.length ? playerAvatar.value : null) : null,
    playerName: playerName.value,
    playerRks: parseFloat(playerRks.value) || 15,
    sampleCount: parseInt(sampleCount.value) || 4,
    resPackPath: respack.value.path,
    speed: 1,
    volumeMusic: volumeMusic.value,
    volumeSfx: volumeSfx.value,
    showPlayer: showPlayer.value,
    exportPath: exportPath.value ? (exportPath.value.trim().length ? exportPath.value : null) : null,
  };
}

function onEnter() {
  if (preset.value.key !== 'default') return;
  resolution.value = RESOLUTIONS[0];
  if (props.initAspectRatio) {
    for (let res of RESOLUTIONS) {
      let [w, h] = parseResolution(res)!;
      if (Math.abs(w / h - props.initAspectRatio) < 0.01) {
        resolution.value = res;
        break;
      }
    }
  }
}

defineExpose({ buildConfig, onEnter });

function applyConfig(config: RenderConfig) {
  resolution.value = config.resolution.join('x');
  endingLength.value = String(config.endingLength);
  fps.value = String(config.fps);
  hwAccel.value = config.hardwareAccel;
  bitrate.value = config.bitrate;

  aggressive.value = config.aggressive;
  challengeColor.value = t('challenge-colors').split(',')[STD_CHALLENGE_COLORS.indexOf(config.challengeColor)] || challengeColor.value;
  challengeRank.value = String(config.challengeRank);
  disableEffect.value = config.disableEffect;
  doubleHint.value = config.doubleHint;
  fxaa.value = config.fxaa;
  noteScale.value = config.noteScale;
  disableParticle.value = !config.particle;
  playerAvatar.value = config.playerAvatar || undefined;
  playerName.value = config.playerName;
  playerRks.value = String(config.playerRks);
  sampleCount.value = String(config.sampleCount);
  respack.value = respacks.value.find((x) => x.path === config.resPackPath) || respacks.value[0];
  volumeMusic.value = config.volumeMusic;
  volumeSfx.value = config.volumeSfx;
  showPlayer.value = config.showPlayer ?? false;
  exportPath.value = config.exportPath || localStorage.getItem('lastExportPath') || null;
  localStorage.setItem('lastRespackPath', respack.value.path || '');
}

const DEFAULT_CONFIG: RenderConfig = {
  resolution: [1920, 1080],
  endingLength: 25.5,
  fps: 60,
  hardwareAccel: true,
  bitrate: '7M',

  aggressive: false,
  challengeColor: 'golden',
  challengeRank: 45,
  disableEffect: false,
  doubleHint: true,
  fxaa: false,
  noteScale: 1,
  particle: true,
  playerAvatar: null,
  playerName: '',
  playerRks: 15,
  sampleCount: 4,
  resPackPath: null,
  speed: 1,
  volumeMusic: 1,
  volumeSfx: 1,
  showPlayer: false,
  exportPath: null,
};
interface Preset {
  name: string;
  key: string;
  config: RenderConfig;
}
const DEFAULT_PRESET: Preset = {
  name: t('default-preset'),
  key: 'default',
  config: DEFAULT_CONFIG,
};

async function getPresets() {
  let result = [DEFAULT_PRESET];
  let pairs = (await invoke('get_presets')) as Record<string, RenderConfig>;
  for (let key of Object.keys(pairs).sort()) {
    result.push({
      name: key,
      key,
      config: pairs[key],
    });
  }
  return result;
}
const presets = ref([DEFAULT_PRESET]);
const preset = ref(DEFAULT_PRESET);
async function updatePresets() {
  presets.value = await getPresets();
  preset.value = presets.value.find((x) => x.key === preset.value.key) || presets.value[0];
}
updatePresets();

async function openRespackFolder() {
  try {
    await invoke('open_respack_folder');
  } catch (e) {
    toastError(e);
  }
}

async function createPreset() {
  let config = await buildConfig();
  if (!config) return;
  let name = prompt(t('preset-create-title'));
  if (!name || !name.length) return;
  if (name === 'default') {
    toast(t('preset-cannot-use-default'), 'error');
    return;
  }
  try {
    await invoke('add_preset', { name, config });
    await updatePresets();
    preset.value = presets.value.find((x) => x.key === name) || presets.value[0];
    toast(t('preset-created'), 'success');
  } catch (e) {
    toastError(e);
  }
}
async function deletePreset() {
  try {
    await invoke('remove_preset', { name: preset.value.key });
    await updatePresets();
    toast(t('preset-deleted'), 'success');
  } catch (e) {
    toastError(e);
  }
}
async function replacePreset() {
  let config = await buildConfig();
  if (!config) return;
  try {
    await invoke('remove_preset', { name: preset.value.key });
    await invoke('add_preset', { name: preset.value.key, config });
    await updatePresets();
    toast(t('preset-replaced'), 'success');
  } catch (e) {
    toastError(e);
  }
}
</script>

<template>
  <div class="config-container">
    <!-- Presets bar -->
    <div class="preset-bar">
      <div class="preset-left">
        <label class="form-label mb-0">{{ t('title.presets') }}:</label>
        <select
          class="form-select preset-select"
          :value="preset.key"
          @change="(e: any) => {
            const found = presets.find(x => x.key === e.target.value);
            if (found) { preset = found; applyConfig(found.config); }
          }"
        >
          <option v-for="p in presets" :key="p.key" :value="p.key">{{ p.name }}</option>
        </select>
      </div>
      <div class="preset-actions">
        <button class="btn btn-secondary btn-sm" @click="updatePresets" :title="t('preset-refresh')">
          <i class="mdi mdi-refresh"></i> {{ t('preset-refresh') }}
        </button>
        <button class="btn btn-secondary btn-sm" @click="createPreset" :title="t('preset-create')">
          <i class="mdi mdi-plus"></i> {{ t('preset-create') }}
        </button>
        <button class="btn btn-secondary btn-sm" :disabled="preset.key === 'default'" @click="replacePreset" :title="t('preset-replace')">
          <i class="mdi mdi-content-save"></i> {{ t('preset-replace') }}
        </button>
        <button class="btn btn-danger btn-sm" :disabled="preset.key === 'default'" @click="deletePreset" :title="t('preset-delete')">
          <i class="mdi mdi-delete"></i> {{ t('preset-delete') }}
        </button>
      </div>
    </div>

    <!-- Output settings card -->
    <div class="clean-card">
      <div class="card-title">
        <i class="mdi mdi-video-outline text-primary"></i>
        <span>{{ t('title.output') }}</span>
      </div>
      
      <div class="grid-3">
        <div class="form-group">
          <label class="form-label">{{ t('resolution') }}</label>
          <input
            list="resolution-list"
            class="form-input"
            v-model="resolution"
            :placeholder="t('resolution')"
          />
          <datalist id="resolution-list">
            <option v-for="res in RESOLUTIONS" :key="res" :value="res"></option>
          </datalist>
        </div>

        <div class="form-group">
          <div class="d-flex justify-between align-center">
            <label class="form-label">{{ t('fps') }}</label>
            <button class="btn btn-ghost btn-sm py-0 px-1 text-primary" @click="fillMonitorFps" :title="t('get-monitor-fps')">
              <i class="mdi mdi-monitor"></i> {{ t('get-monitor-fps') }}
            </button>
          </div>
          <input type="number" class="form-input" v-model="fps" />
        </div>

        <div class="form-group d-flex justify-center align-start pt-4">
          <TipSwitch :label="t('hw-accel')" :tooltip="t('hw-accel-tips')" v-model="hwAccel" />
        </div>
      </div>

      <div class="grid-3 mt-2">
        <TipTextField
          :label="t('sample-count')"
          type="number"
          :rules="[sampleCountRule]"
          v-model="sampleCount"
          :tooltip="t('sample-count-tips')"
        />
        <TipTextField
          :label="t('bitrate')"
          :rules="[RULES.non_empty]"
          v-model="bitrate"
          :tooltip="t('bitrate-tips')"
        />
        <div class="form-group d-flex justify-center align-start pt-4">
          <TipSwitch :label="t('fxaa')" :tooltip="t('fxaa-tips')" v-model="fxaa" />
        </div>
      </div>

      <div class="form-group mt-2">
        <label class="form-label">
          {{ t('export-path') }}
          <TooltipIcon :tooltip="t('export-path-tips')" />
        </label>
        <div class="d-flex gap-2">
          <input
            class="form-input flex-1"
            v-model="exportPath"
            placeholder="留空则保存至默认输出目录"
          />
          <button class="btn btn-secondary" @click="chooseExportPath">
            <i class="mdi mdi-folder-open"></i> {{ t('choose-folder') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Graphics & Mods settings card -->
    <div class="clean-card">
      <div class="card-title">
        <i class="mdi mdi-palette-outline text-primary"></i>
        <span>{{ t('title.graphics') }}</span>
      </div>

      <div class="grid-2 align-end">
        <div class="form-group mb-0">
          <label class="form-label">{{ t('respack') }}</label>
          <select
            class="form-select"
            :value="respack.name"
            @change="(e: any) => {
              const found = respacks.find(r => r.name === e.target.value);
              if (found) respack = found;
            }"
          >
            <option v-for="r in respacks" :key="r.name" :value="r.name">{{ r.name }}</option>
          </select>
        </div>
        <div class="d-flex gap-2 mb-0">
          <button class="btn btn-secondary" @click="updateRespacks">
            <i class="mdi mdi-refresh"></i> {{ t('respack-refresh') }}
          </button>
          <button class="btn btn-secondary" @click="openRespackFolder">
            <i class="mdi mdi-folder-outline"></i> {{ t('respack-open') }}
          </button>
        </div>
      </div>

      <div class="mt-4">
        <CustomSlider
          :label="t('note-scale')"
          :min="0.3"
          :max="1.5"
          :step="0.05"
          v-model="noteScale"
          :format-value="(v) => v.toFixed(2) + 'x'"
        />
      </div>

      <div class="grid-4 mt-4">
        <TipSwitch :label="t('double-hint')" v-model="doubleHint" />
        <TipSwitch :label="t('aggressive')" :tooltip="t('aggressive-tips')" v-model="aggressive" />
        <TipSwitch :label="t('disable-particle')" v-model="disableParticle" />
        <TipSwitch :label="t('disable-effect')" v-model="disableEffect" />
      </div>
    </div>

    <!-- Audio & Result settings card -->
    <div class="clean-card">
      <div class="card-title">
        <i class="mdi mdi-volume-high text-primary"></i>
        <span>{{ t('title.audio') }}</span>
      </div>

      <div class="grid-2">
        <CustomSlider
          :label="t('volume-music')"
          :min="0"
          :max="2"
          :step="0.05"
          v-model="volumeMusic"
          :format-value="(v) => Math.round(v * 100) + '%'"
        />
        <CustomSlider
          :label="t('volume-sfx')"
          :min="0"
          :max="2"
          :step="0.05"
          v-model="volumeSfx"
          :format-value="(v) => Math.round(v * 100) + '%'"
        />
      </div>

      <div class="grid-2 mt-4 align-center">
        <TipTextField
          :label="t('ending-length')"
          v-model="endingLength"
          type="number"
          :rules="[RULES.positive]"
        />
        <div class="pt-4">
          <TipSwitch
            :label="t('show-player')"
            :tooltip="t('show-player-tips')"
            v-model="showPlayer"
          />
        </div>
      </div>

      <!-- Player section (collapsible when showPlayer is active) -->
      <div v-if="showPlayer" class="player-box mt-4">
        <div class="card-title mb-2 text-muted" style="font-size: 13px">
          <i class="mdi mdi-account-circle-outline"></i>
          <span>{{ t('title.player') }}</span>
        </div>

        <div class="grid-3">
          <div class="form-group">
            <label class="form-label">{{ t('player-avatar') }}</label>
            <div class="d-flex gap-2">
              <input
                readonly
                class="form-input flex-1"
                :value="playerAvatar ? playerAvatar.split('\\').pop()!.split('/').pop() : '未选择'"
                @click="chooseAvatar"
              />
              <button class="btn btn-secondary btn-sm" @click="chooseAvatar">选择</button>
            </div>
          </div>

          <TipTextField
            :label="t('player-name')"
            :rules="[RULES.non_empty]"
            v-model="playerName"
          />

          <TipTextField
            :label="t('player-rks')"
            :rules="[RULES.positive]"
            type="number"
            v-model="playerRks"
          />
        </div>

        <div class="grid-2 mt-2">
          <div class="form-group">
            <label class="form-label">{{ t('challenge-color') }}</label>
            <select class="form-select" v-model="challengeColor">
              <option v-for="c in t('challenge-colors').split(',')" :key="c" :value="c">{{ c }}</option>
            </select>
          </div>

          <TipTextField
            :label="t('challenge-rank')"
            :rules="[RULES.positiveInt]"
            type="number"
            v-model="challengeRank"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-height: 60vh;
  overflow-y: auto;
  padding-right: 4px;
}

.preset-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  padding: 10px 16px;
  border-radius: var(--radius-sm);
}

.preset-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.preset-select {
  width: 160px;
  padding: 6px 12px;
  padding-right: 28px;
  font-size: 13px;
}

.preset-actions {
  display: flex;
  gap: 6px;
}

.player-box {
  background: var(--bg-input);
  border: 1px dashed var(--border-color);
  border-radius: var(--radius-sm);
  padding: 14px;
}

.d-flex { display: flex; }
.justify-between { justify-content: space-between; }
.justify-center { justify-content: center; }
.align-center { align-items: center; }
.align-start { align-items: flex-start; }
.align-end { align-items: flex-end; }
.flex-1 { flex: 1; }
.gap-2 { gap: 8px; }
.mt-2 { margin-top: 8px; }
.mt-4 { margin-top: 16px; }
.mb-0 { margin-bottom: 0 !important; }
.mb-2 { margin-bottom: 8px; }
.pt-4 { padding-top: 16px; }
.text-primary { color: var(--primary); }
</style>
