<template>
  <div class="preview-ctrl-container">
    <!-- Header -->
    <div class="ctrl-header">
      <div class="header-left">
        <div class="status-indicator" :class="{ paused: status.paused }">
          <i class="mdi" :class="status.paused ? 'mdi-pause' : 'mdi-play'"></i>
          <span>{{ status.paused ? '已暂停' : '播放中' }}</span>
        </div>
      </div>
      <div class="header-actions">
        <button class="btn-icon danger" title="退出预览 (Esc)" @click="sendCmd('exit')">
          <i class="mdi mdi-close"></i>
        </button>
      </div>
    </div>

    <!-- Main Controls -->
    <div class="ctrl-section play-actions">
      <button class="ctrl-btn retry-btn" title="重播选区 (R)" @click="sendCmd('replay')">
        <i class="mdi mdi-replay"></i>
        <span>重播选区</span>
      </button>
      <button
        class="ctrl-btn play-btn"
        :class="{ 'is-paused': status.paused }"
        title="暂停/播放 (Space)"
        @click="sendCmd('toggle_pause')"
      >
        <i class="mdi" :class="status.paused ? 'mdi-play' : 'mdi-pause'"></i>
        <span>{{ status.paused ? '继续播放' : '暂停' }}</span>
      </button>
    </div>

    <!-- Speed Adjustment -->
    <div class="ctrl-section">
      <div class="section-label">
        <span>播放速度</span>
        <span class="value-badge">{{ speedText }}x</span>
      </div>
      <div class="slider-row">
        <button class="step-btn" @click="adjustSpeed(-0.05)">
          <i class="mdi mdi-minus"></i>
        </button>
        <input
          type="range"
          class="custom-slider"
          min="0.5"
          max="2.0"
          step="0.05"
          :value="status.speed"
          @input="onSpeedSlider"
        />
        <button class="step-btn" @click="adjustSpeed(0.05)">
          <i class="mdi mdi-plus"></i>
        </button>
      </div>
      <div class="speed-chips">
        <button
          v-for="s in [0.5, 0.75, 1.0, 1.25, 1.5, 2.0]"
          :key="s"
          class="chip-btn"
          :class="{ active: Math.abs(status.speed - s) < 0.01 }"
          @click="setSpeed(s)"
        >
          {{ s }}x
        </button>
      </div>
    </div>

    <!-- Current Time -->
    <div class="ctrl-section">
      <div class="section-label">
        <span>当前时间戳</span>
        <span class="total-len">/ {{ fmtTime(status.length) }}</span>
      </div>
      <div class="time-box" @click="startEditTime">
        <input
          v-if="editingTime"
          ref="timeInputRef"
          v-model="editTimeStr"
          class="time-edit-input"
          @blur="finishEditTime"
          @keydown.enter="finishEditTime"
          @keydown.esc="cancelEditTime"
        />
        <span v-else class="time-display">{{ fmtTime(status.time) }}</span>
        <i class="mdi mdi-pencil-outline edit-icon"></i>
      </div>
      <div class="quick-jump-row">
        <button class="jump-btn" @click="seekDelta(-5)">-5s</button>
        <button class="jump-btn" @click="seekDelta(-1)">-1s</button>
        <button class="jump-btn" @click="seekDelta(1)">+1s</button>
        <button class="jump-btn" @click="seekDelta(5)">+5s</button>
      </div>
    </div>

    <!-- AB Loop Range -->
    <div class="ctrl-section">
      <div class="section-label">
        <span>AB 循环区间</span>
        <button class="link-btn" @click="resetRange">重置为全曲</button>
      </div>
      
      <!-- Range track visualization -->
      <div class="range-track-box">
        <div class="range-track-bg"></div>
        <div
          class="range-selected-bar"
          :style="{
            left: `${rangeStartPercent}%`,
            width: `${rangeWidthPercent}%`,
          }"
        ></div>
        <div class="range-cursor" :style="{ left: `${currentPercent}%` }"></div>
      </div>

      <div class="range-inputs-row">
        <div class="range-input-group">
          <label>起点 (A)</label>
          <div class="input-with-action">
            <input
              type="text"
              class="range-val-input"
              :value="fmtTime(status.start)"
              @change="onStartChange"
            />
            <button class="mini-btn" title="设为当前时间" @click="setStartToCurrent">
              <i class="mdi mdi-map-marker-down"></i>
            </button>
          </div>
        </div>

        <div class="range-divider">至</div>

        <div class="range-input-group">
          <label>终点 (B)</label>
          <div class="input-with-action">
            <input
              type="text"
              class="range-val-input"
              :value="fmtTime(status.end)"
              @change="onEndChange"
            />
            <button class="mini-btn" title="设为当前时间" @click="setEndToCurrent">
              <i class="mdi mdi-map-marker-down"></i>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Shortcuts Footer -->
    <div class="ctrl-footer">
      <div class="shortcut-tip">
        <kbd>Space</kbd> 暂停/继续 · <kbd>R</kbd> 重播 · <kbd>←</kbd><kbd>→</kbd> 步进
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

interface PreviewStatus {
  time: number;
  paused: boolean;
  speed: number;
  start: number;
  end: number;
  length: number;
}

const status = ref<PreviewStatus>({
  time: 0,
  paused: false,
  speed: 1.0,
  start: 0,
  end: 180,
  length: 180,
});

const editingTime = ref(false);
const editTimeStr = ref('');
const timeInputRef = ref<HTMLInputElement>();

let unlistenStatus: UnlistenFn | null = null;

const speedText = computed(() => status.value.speed.toFixed(2));

const currentPercent = computed(() => {
  const len = Math.max(status.value.length, 1);
  return Math.min(Math.max((status.value.time / len) * 100, 0), 100);
});

const rangeStartPercent = computed(() => {
  const len = Math.max(status.value.length, 1);
  return Math.min(Math.max((status.value.start / len) * 100, 0), 100);
});

const rangeWidthPercent = computed(() => {
  const len = Math.max(status.value.length, 1);
  const st = Math.max(status.value.start, 0);
  const en = Math.min(status.value.end, len);
  return Math.min(Math.max(((en - st) / len) * 100, 0), 100);
});

function fmtTime(sec: number): string {
  if (isNaN(sec) || sec < 0) sec = 0;
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  const ms = Math.floor((sec % 1) * 100);
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}.${String(ms).padStart(2, '0')}`;
}

function parseTime(str: string): number | null {
  const parts = str.trim().split(':');
  if (parts.length === 2) {
    const m = parseFloat(parts[0]);
    const s = parseFloat(parts[1]);
    if (!isNaN(m) && !isNaN(s)) return m * 60 + s;
  } else if (parts.length === 3) {
    const h = parseFloat(parts[0]);
    const m = parseFloat(parts[1]);
    const s = parseFloat(parts[2]);
    if (!isNaN(h) && !isNaN(m) && !isNaN(s)) return h * 3600 + m * 60 + s;
  } else {
    const s = parseFloat(str.trim());
    if (!isNaN(s)) return s;
  }
  return null;
}

async function sendCmd(action: string, payload: any = {}) {
  try {
    await invoke('send_preview_command', {
      cmd: JSON.stringify({ action, ...payload }),
    });
  } catch (err) {
    console.error('Failed to send command:', err);
  }
}

function setSpeed(val: number) {
  status.value.speed = val;
  sendCmd('set_speed', { speed: val });
}

function adjustSpeed(delta: number) {
  const val = Math.round((status.value.speed + delta) * 100) / 100;
  if (val >= 0.5 && val <= 2.0) {
    setSpeed(val);
  }
}

function onSpeedSlider(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  setSpeed(val);
}

function seekDelta(delta: number) {
  const t = Math.max(0, Math.min(status.value.time + delta, status.value.length));
  sendCmd('seek', { time: t });
}

function startEditTime() {
  editingTime.value = true;
  editTimeStr.value = fmtTime(status.value.time);
  nextTick(() => {
    timeInputRef.value?.focus();
    timeInputRef.value?.select();
  });
}

function finishEditTime() {
  if (!editingTime.value) return;
  editingTime.value = false;
  const t = parseTime(editTimeStr.value);
  if (t !== null && t >= 0 && t <= status.value.length) {
    sendCmd('seek', { time: t });
  }
}

function cancelEditTime() {
  editingTime.value = false;
}

function setStartToCurrent() {
  const st = status.value.time;
  const en = Math.max(st + 1, status.value.end);
  status.value.start = st;
  status.value.end = en;
  sendCmd('set_range', { start: st, end: en });
}

function setEndToCurrent() {
  const en = status.value.time;
  const st = Math.min(en - 1, status.value.start);
  status.value.start = st;
  status.value.end = en;
  sendCmd('set_range', { start: st, end: en });
}

function onStartChange(e: Event) {
  const t = parseTime((e.target as HTMLInputElement).value);
  if (t !== null && t >= 0 && t < status.value.end) {
    status.value.start = t;
    sendCmd('set_range', { start: t, end: status.value.end });
  }
}

function onEndChange(e: Event) {
  const t = parseTime((e.target as HTMLInputElement).value);
  if (t !== null && t > status.value.start && t <= status.value.length) {
    status.value.end = t;
    sendCmd('set_range', { start: status.value.start, end: t });
  }
}

function resetRange() {
  status.value.start = 0;
  status.value.end = status.value.length;
  sendCmd('set_range', { start: 0, end: status.value.length });
}

function onKeyDown(e: KeyboardEvent) {
  if (editingTime.value) return;
  if (e.code === 'Space') {
    e.preventDefault();
    sendCmd('toggle_pause');
  } else if (e.code === 'KeyR') {
    e.preventDefault();
    sendCmd('replay');
  } else if (e.code === 'ArrowLeft') {
    e.preventDefault();
    seekDelta(-1);
  } else if (e.code === 'ArrowRight') {
    e.preventDefault();
    seekDelta(1);
  } else if (e.code === 'Escape') {
    e.preventDefault();
    sendCmd('exit');
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown);
  unlistenStatus = await listen<string>('preview-status', (event) => {
    try {
      const data = JSON.parse(event.payload);
      status.value = { ...status.value, ...data };
    } catch (e) {
      // ignore
    }
  });
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown);
  if (unlistenStatus) unlistenStatus();
});
</script>

<style scoped>
.preview-ctrl-container {
  height: 100vh;
  width: 100%;
  background: #0d1117;
  color: #e6edf3;
  display: flex;
  flex-direction: column;
  padding: 16px;
  gap: 16px;
  overflow-y: auto;
  user-select: none;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  box-sizing: border-box;
}

/* Header */
.ctrl-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 600;
  color: #3fb950;
  background: rgba(63, 185, 80, 0.12);
  padding: 4px 10px;
  border-radius: 20px;
  border: 1px solid rgba(63, 185, 80, 0.25);
}

.status-indicator.paused {
  color: #d29922;
  background: rgba(210, 153, 34, 0.12);
  border-color: rgba(210, 153, 34, 0.25);
}

.btn-icon {
  width: 30px;
  height: 30px;
  border-radius: 6px;
  border: none;
  background: rgba(255, 255, 255, 0.06);
  color: #8b949e;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  transition: all 0.15s ease;
}

.btn-icon:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #f0f6fc;
}

.btn-icon.danger:hover {
  background: rgba(248, 81, 73, 0.2);
  color: #f85149;
}

/* Sections */
.ctrl-section {
  background: #161b22;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: #8b949e;
  font-weight: 500;
}

.value-badge {
  font-size: 12px;
  color: #58a6ff;
  font-weight: 600;
  font-family: monospace;
}

.total-len {
  font-size: 12px;
  color: #6e7681;
  font-family: monospace;
}

/* Action Buttons */
.play-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  background: transparent;
  border: none;
  padding: 0;
}

.ctrl-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px;
  border-radius: 8px;
  border: none;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.retry-btn {
  background: rgba(88, 166, 255, 0.12);
  color: #58a6ff;
  border: 1px solid rgba(88, 166, 255, 0.25);
}

.retry-btn:hover {
  background: rgba(88, 166, 255, 0.2);
  border-color: rgba(88, 166, 255, 0.4);
}

.play-btn {
  background: #238636;
  color: #ffffff;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.play-btn:hover {
  background: #2ea043;
}

.play-btn.is-paused {
  background: #1f6feb;
}

.play-btn.is-paused:hover {
  background: #388bfd;
}

/* Slider */
.slider-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.step-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: #21262d;
  color: #c9d1d9;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
}

.step-btn:hover {
  background: #30363d;
  color: #ffffff;
}

.custom-slider {
  flex: 1;
  accent-color: #58a6ff;
  height: 4px;
  background: #21262d;
  border-radius: 2px;
  cursor: pointer;
}

.speed-chips {
  display: flex;
  gap: 4px;
  justify-content: space-between;
}

.chip-btn {
  flex: 1;
  padding: 4px 0;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: #21262d;
  color: #8b949e;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.1s ease;
}

.chip-btn:hover {
  background: #30363d;
  color: #c9d1d9;
}

.chip-btn.active {
  background: rgba(88, 166, 255, 0.15);
  border-color: #58a6ff;
  color: #58a6ff;
  font-weight: 600;
}

/* Time Box */
.time-box {
  background: #0d1117;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  padding: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  cursor: pointer;
  transition: border-color 0.15s ease;
}

.time-box:hover {
  border-color: #58a6ff;
}

.time-display {
  font-size: 20px;
  font-weight: 700;
  font-family: 'SF Mono', Consolas, monospace;
  color: #58a6ff;
  letter-spacing: 1px;
}

.time-edit-input {
  font-size: 20px;
  font-weight: 700;
  font-family: 'SF Mono', Consolas, monospace;
  color: #58a6ff;
  background: transparent;
  border: none;
  outline: none;
  text-align: center;
  width: 100%;
}

.edit-icon {
  position: absolute;
  right: 10px;
  color: #6e7681;
  font-size: 14px;
}

.quick-jump-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 6px;
}

.jump-btn {
  padding: 6px 0;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: #21262d;
  color: #c9d1d9;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
}

.jump-btn:hover {
  background: #30363d;
  color: #ffffff;
}

/* Range Track */
.range-track-box {
  position: relative;
  height: 8px;
  margin: 6px 0;
}

.range-track-bg {
  position: absolute;
  inset: 0;
  background: #21262d;
  border-radius: 4px;
}

.range-selected-bar {
  position: absolute;
  top: 0;
  bottom: 0;
  background: rgba(88, 166, 255, 0.4);
  border-radius: 4px;
  border-left: 2px solid #58a6ff;
  border-right: 2px solid #f85149;
}

.range-cursor {
  position: absolute;
  top: -3px;
  width: 3px;
  height: 14px;
  background: #3fb950;
  border-radius: 1px;
  transform: translateX(-50%);
  box-shadow: 0 0 6px #3fb950;
}

.range-inputs-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.range-input-group {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.range-input-group label {
  font-size: 11px;
  color: #8b949e;
}

.input-with-action {
  display: flex;
  background: #0d1117;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  overflow: hidden;
}

.range-val-input {
  flex: 1;
  background: transparent;
  border: none;
  color: #c9d1d9;
  font-size: 12px;
  font-family: monospace;
  padding: 4px 6px;
  outline: none;
  width: 0;
}

.mini-btn {
  background: #21262d;
  border: none;
  border-left: 1px solid rgba(255, 255, 255, 0.1);
  color: #8b949e;
  padding: 0 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
}

.mini-btn:hover {
  background: #30363d;
  color: #58a6ff;
}

.range-divider {
  font-size: 11px;
  color: #6e7681;
  padding-top: 14px;
}

.link-btn {
  background: transparent;
  border: none;
  color: #58a6ff;
  font-size: 11px;
  cursor: pointer;
  padding: 0;
}

.link-btn:hover {
  text-decoration: underline;
}

/* Footer */
.ctrl-footer {
  margin-top: auto;
  text-align: center;
  padding-top: 8px;
}

.shortcut-tip {
  font-size: 11px;
  color: #6e7681;
}

.shortcut-tip kbd {
  background: #21262d;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  padding: 1px 4px;
  font-size: 10px;
  color: #8b949e;
}
</style>
