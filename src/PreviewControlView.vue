<template>
  <div class="preview-ctrl-container">
    <!-- Top Action & Status Bar -->
    <div class="top-bar">
      <button
        class="play-toggle-btn"
        :class="{ 'is-paused': status.paused }"
        title="暂停 / 继续播放 (Space)"
        @click="sendCmd('toggle_pause')"
      >
        <i class="mdi" :class="status.paused ? 'mdi-play' : 'mdi-pause'"></i>
        <span>{{ status.paused ? '继续播放' : '正在播放' }}</span>
        <kbd>Space</kbd>
      </button>

      <div class="top-actions">
        <button
          class="action-btn retry-btn"
          title="重播当前循环选区 (R)"
          @click="sendCmd('replay')"
        >
          <i class="mdi mdi-replay"></i>
          <span>重播 (R)</span>
        </button>
        <button
          class="action-btn close-btn"
          title="退出预览 (Esc)"
          @click="sendCmd('exit')"
        >
          <i class="mdi mdi-close"></i>
        </button>
      </div>
    </div>

    <!-- Playback Speed Control -->
    <div class="card speed-card">
      <div class="card-header">
        <span class="label">播放速度</span>
        <span class="value-highlight">{{ speedText }}x</span>
      </div>
      <div class="speed-slider-row">
        <button class="step-btn" title="减小速度" @click="adjustSpeed(-0.05)">
          <i class="mdi mdi-minus"></i>
        </button>
        <input
          type="range"
          class="speed-slider"
          min="0.5"
          max="2.0"
          step="0.05"
          :value="status.speed"
          @input="onSpeedSlider"
        />
        <button class="step-btn" title="增加速度" @click="adjustSpeed(0.05)">
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

    <!-- Unified Interactive Timeline & AB Loop Scrubbing Station -->
    <div class="card timeline-card">
      <!-- Time Display & Jump Stepper Header -->
      <div class="time-header-row">
        <div class="jump-group left">
          <button class="jump-chip" @click="seekDelta(-5)">-5s</button>
          <button class="jump-chip" @click="seekDelta(-1)">-1s</button>
        </div>

        <div class="digital-time-box" title="点击就地编辑时间戳" @click="startEditTime">
          <input
            v-if="editingTime"
            ref="timeInputRef"
            v-model="editTimeStr"
            class="digital-input"
            @blur="finishEditTime"
            @keydown.enter="finishEditTime"
            @keydown.esc="cancelEditTime"
          />
          <template v-else>
            <span class="cur-time">{{ fmtTime(displayTime) }}</span>
            <span class="time-sep">/</span>
            <span class="total-time">{{ fmtTime(status.length) }}</span>
            <i class="mdi mdi-pencil-outline edit-pen"></i>
          </template>
        </div>

        <div class="jump-group right">
          <button class="jump-chip" @click="seekDelta(1)">+1s</button>
          <button class="jump-chip" @click="seekDelta(5)">+5s</button>
        </div>
      </div>

      <!-- Main Interactive Timeline Track (Clickable & Draggable) -->
      <div
        ref="trackContainerRef"
        class="timeline-track-wrap"
        :class="{ dragging: dragTarget !== null }"
        @mousedown="onTrackMouseDown"
        @touchstart.passive="onTrackTouchStart"
      >
        <!-- Background Track -->
        <div class="track-bg-line"></div>

        <!-- Loop Range Selected Bar -->
        <div
          class="track-loop-bar"
          :style="{
            left: `${startPercent}%`,
            width: `${rangeWidthPercent}%`,
          }"
        ></div>

        <!-- Loop Start Pin (A - Blue) -->
        <div
          class="track-pin pin-start"
          :style="{ left: `${startPercent}%` }"
          title="拖拽调整循环起点 (A)"
          @mousedown.stop="startPinDrag('start', $event)"
          @touchstart.stop="startPinTouchDrag('start', $event)"
        >
          <div class="pin-badge">A</div>
          <div class="pin-stem"></div>
        </div>

        <!-- Loop End Pin (B - Red) -->
        <div
          class="track-pin pin-end"
          :style="{ left: `${endPercent}%` }"
          title="拖拽调整循环终点 (B)"
          @mousedown.stop="startPinDrag('end', $event)"
          @touchstart.stop="startPinTouchDrag('end', $event)"
        >
          <div class="pin-badge">B</div>
          <div class="pin-stem"></div>
        </div>

        <!-- Playback Cursor (Current Time - Green) -->
        <div
          class="track-cursor"
          :style="{ left: `${curPercent}%` }"
          title="拖拽即时跳转时间"
          @mousedown.stop="startPinDrag('cursor', $event)"
          @touchstart.stop="startPinTouchDrag('cursor', $event)"
        >
          <div class="cursor-head"></div>
          <div class="cursor-line"></div>
        </div>
      </div>

      <!-- AB Loop Range Values & Quick Setters -->
      <div class="loop-control-footer">
        <!-- Start A Input -->
        <div class="loop-point-box">
          <div class="point-header">
            <span class="point-tag tag-a">起点 A</span>
            <button class="set-cur-btn" title="将当前播放时间设为起点" @click="setStartToCurrent">
              <i class="mdi mdi-map-marker-down"></i>
              <span>设为当前</span>
            </button>
          </div>
          <input
            type="text"
            class="point-input"
            :value="fmtTime(status.start)"
            @change="onStartChange"
          />
        </div>

        <!-- Reset All Button in center -->
        <button class="reset-range-btn" title="重置循环区间为全曲" @click="resetRange">
          <i class="mdi mdi-restore"></i>
          <span>全曲</span>
        </button>

        <!-- End B Input -->
        <div class="loop-point-box right">
          <div class="point-header right">
            <button class="set-cur-btn" title="将当前播放时间设为终点" @click="setEndToCurrent">
              <i class="mdi mdi-map-marker-down"></i>
              <span>设为当前</span>
            </button>
            <span class="point-tag tag-b">终点 B</span>
          </div>
          <input
            type="text"
            class="point-input right"
            :value="fmtTime(status.end)"
            @change="onEndChange"
          />
        </div>
      </div>
    </div>

    <!-- Shortcut Hint Footer -->
    <div class="footer-tips">
      <span>快捷键：<kbd>Space</kbd> 暂停/继续 · <kbd>R</kbd> 重播 · <kbd>←</kbd><kbd>→</kbd> 步进 1s</span>
      <div v-if="lastSentCmd" style="font-size: 11px; color: #6ee7b7; margin-top: 3px; font-family: monospace;">
        📡 {{ lastSentCmd }}
      </div>
      <div v-if="cmdError" style="font-size: 11px; color: #f87171; margin-top: 3px; font-weight: bold;">
        ❌ 下发异常: {{ cmdError }}
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

// Drag state
type DragTarget = 'cursor' | 'start' | 'end' | null;
const dragTarget = ref<DragTarget>(null);
const draggingTime = ref<number | null>(null);
const trackContainerRef = ref<HTMLElement>();

// Inline edit time
const editingTime = ref(false);
const editTimeStr = ref('');
const timeInputRef = ref<HTMLInputElement>();

let unlistenStatus: UnlistenFn | null = null;
let throttleTimer: any = null;
let ignoreTimeUntil = 0;
let ignoreRangeUntil = 0;

const speedText = computed(() => status.value.speed.toFixed(2));

const displayTime = computed(() => {
  if (dragTarget.value === 'cursor' && draggingTime.value !== null) {
    return draggingTime.value;
  }
  return status.value.time;
});

const curPercent = computed(() => {
  const len = Math.max(status.value.length, 1);
  return Math.min(Math.max((displayTime.value / len) * 100, 0), 100);
});

const startPercent = computed(() => {
  const len = Math.max(status.value.length, 1);
  const st = dragTarget.value === 'start' && draggingTime.value !== null
    ? draggingTime.value
    : status.value.start;
  return Math.min(Math.max((st / len) * 100, 0), 100);
});

const endPercent = computed(() => {
  const len = Math.max(status.value.length, 1);
  const en = dragTarget.value === 'end' && draggingTime.value !== null
    ? draggingTime.value
    : status.value.end;
  return Math.min(Math.max((en / len) * 100, 0), 100);
});

const rangeWidthPercent = computed(() => {
  return Math.max(0, endPercent.value - startPercent.value);
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

const lastSentCmd = ref('');
const cmdError = ref('');

async function sendCmd(action: string, payload: any = {}) {
  const cmdStr = JSON.stringify({ action, ...payload });
  lastSentCmd.value = cmdStr;
  cmdError.value = '';
  try {
    await invoke('send_preview_command', { payload: cmdStr });
  } catch (err: any) {
    cmdError.value = String(err?.message || err);
    console.error('Failed to send command:', err);
  }
}

function seekTo(t: number) {
  const clamped = Math.max(0, Math.min(t, status.value.length));
  status.value.time = clamped;
  ignoreTimeUntil = Date.now() + 1000;
  sendCmd('seek', { time: clamped });
}

function setRangeTo(start: number, end: number) {
  status.value.start = start;
  status.value.end = end;
  ignoreRangeUntil = Date.now() + 1000;
  sendCmd('set_range', { start, end });
}

function throttledSeek(time: number) {
  if (throttleTimer) return;
  throttleTimer = setTimeout(() => {
    throttleTimer = null;
    sendCmd('seek', { time });
  }, 35);
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
  seekTo(status.value.time + delta);
}

// Inline edit time
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
    seekTo(t);
  }
}

function cancelEditTime() {
  editingTime.value = false;
}

// Track Dragging & Scrubbing
function getTimeFromEvent(clientX: number): number {
  if (!trackContainerRef.value) return 0;
  const rect = trackContainerRef.value.getBoundingClientRect();
  const ratio = Math.min(Math.max((clientX - rect.left) / rect.width, 0), 1);
  return ratio * status.value.length;
}

function onTrackMouseDown(e: MouseEvent) {
  const t = getTimeFromEvent(e.clientX);
  const len = Math.max(status.value.length, 1);
  const clickRatio = t / len;
  const startRatio = status.value.start / len;
  const endRatio = status.value.end / len;

  if (Math.abs(clickRatio - startRatio) < 0.03) {
    startPinDrag('start', e);
  } else if (Math.abs(clickRatio - endRatio) < 0.03) {
    startPinDrag('end', e);
  } else {
    dragTarget.value = 'cursor';
    draggingTime.value = t;
    status.value.time = t;
    ignoreTimeUntil = Date.now() + 1000;
    sendCmd('seek', { time: t });
    window.addEventListener('mousemove', onWindowMouseMove);
    window.addEventListener('mouseup', onWindowMouseUp);
  }
}

function onTrackTouchStart(e: TouchEvent) {
  if (e.touches.length === 0) return;
  const clientX = e.touches[0].clientX;
  const t = getTimeFromEvent(clientX);
  dragTarget.value = 'cursor';
  draggingTime.value = t;
  status.value.time = t;
  ignoreTimeUntil = Date.now() + 1000;
  sendCmd('seek', { time: t });
  window.addEventListener('touchmove', onWindowTouchMove, { passive: false });
  window.addEventListener('touchend', onWindowTouchEnd);
}

function startPinDrag(target: DragTarget, e: MouseEvent) {
  e.preventDefault();
  dragTarget.value = target;
  draggingTime.value = target === 'start' ? status.value.start : target === 'end' ? status.value.end : status.value.time;
  if (target === 'cursor') ignoreTimeUntil = Date.now() + 1000;
  if (target === 'start' || target === 'end') ignoreRangeUntil = Date.now() + 1000;
  window.addEventListener('mousemove', onWindowMouseMove);
  window.addEventListener('mouseup', onWindowMouseUp);
}

function startPinTouchDrag(target: DragTarget, e: TouchEvent) {
  e.preventDefault();
  dragTarget.value = target;
  draggingTime.value = target === 'start' ? status.value.start : target === 'end' ? status.value.end : status.value.time;
  if (target === 'cursor') ignoreTimeUntil = Date.now() + 1000;
  if (target === 'start' || target === 'end') ignoreRangeUntil = Date.now() + 1000;
  window.addEventListener('touchmove', onWindowTouchMove, { passive: false });
  window.addEventListener('touchend', onWindowTouchEnd);
}

function onWindowMouseMove(e: MouseEvent) {
  if (!dragTarget.value) return;
  const t = getTimeFromEvent(e.clientX);
  applyDragTime(t);
}

function onWindowTouchMove(e: TouchEvent) {
  if (!dragTarget.value || e.touches.length === 0) return;
  e.preventDefault();
  const t = getTimeFromEvent(e.touches[0].clientX);
  applyDragTime(t);
}

function applyDragTime(t: number) {
  draggingTime.value = t;
  if (dragTarget.value === 'cursor') {
    status.value.time = t;
    ignoreTimeUntil = Date.now() + 1000;
    throttledSeek(t);
  } else if (dragTarget.value === 'start') {
    const st = Math.max(0, Math.min(t, status.value.end - 0.5));
    status.value.start = st;
    ignoreRangeUntil = Date.now() + 1000;
  } else if (dragTarget.value === 'end') {
    const en = Math.min(status.value.length, Math.max(t, status.value.start + 0.5));
    status.value.end = en;
    ignoreRangeUntil = Date.now() + 1000;
  }
}

function onWindowMouseUp(e: MouseEvent) {
  finishDrag(e.clientX);
  window.removeEventListener('mousemove', onWindowMouseMove);
  window.removeEventListener('mouseup', onWindowMouseUp);
}

function onWindowTouchEnd(e: TouchEvent) {
  const clientX = e.changedTouches.length > 0 ? e.changedTouches[0].clientX : 0;
  finishDrag(clientX);
  window.removeEventListener('touchmove', onWindowTouchMove);
  window.removeEventListener('touchend', onWindowTouchEnd);
}

function finishDrag(clientX: number) {
  if (!dragTarget.value) return;
  if (throttleTimer) {
    clearTimeout(throttleTimer);
    throttleTimer = null;
  }
  const currentTarget = dragTarget.value;
  const t = clientX ? getTimeFromEvent(clientX) : (draggingTime.value ?? status.value.time);
  if (currentTarget === 'cursor') {
    const clamped = Math.max(0, Math.min(t, status.value.length));
    status.value.time = clamped;
    ignoreTimeUntil = Date.now() + 1000;
    sendCmd('seek', { time: clamped });
  } else if (currentTarget === 'start') {
    const st = Math.max(0, Math.min(t, status.value.end - 0.5));
    status.value.start = st;
    ignoreRangeUntil = Date.now() + 1000;
    sendCmd('set_range', { start: st, end: status.value.end });
  } else if (currentTarget === 'end') {
    const en = Math.min(status.value.length, Math.max(t, status.value.start + 0.5));
    status.value.end = en;
    ignoreRangeUntil = Date.now() + 1000;
    sendCmd('set_range', { start: status.value.start, end: en });
  }
  dragTarget.value = null;
  draggingTime.value = null;
}

// Loop Point Setting
function setStartToCurrent() {
  const st = status.value.time;
  const en = Math.max(st + 0.5, status.value.end);
  setRangeTo(st, en);
}

function setEndToCurrent() {
  const en = status.value.time;
  const st = Math.min(en - 0.5, status.value.start);
  setRangeTo(st, en);
}

function onStartChange(e: Event) {
  const t = parseTime((e.target as HTMLInputElement).value);
  if (t !== null && t >= 0 && t < status.value.end) {
    setRangeTo(t, status.value.end);
  }
}

function onEndChange(e: Event) {
  const t = parseTime((e.target as HTMLInputElement).value);
  if (t !== null && t > status.value.start && t <= status.value.length) {
    setRangeTo(status.value.start, t);
  }
}

function resetRange() {
  setRangeTo(0, status.value.length);
}

// Key shortcuts
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
      const data: Partial<PreviewStatus> = JSON.parse(event.payload);
      const now = Date.now();

      if (dragTarget.value === 'cursor') {
        delete data.time;
      } else if (now < ignoreTimeUntil) {
        if (data.time !== undefined && Math.abs(data.time - status.value.time) < 0.15) {
          ignoreTimeUntil = 0;
        } else {
          delete data.time;
        }
      }

      if (dragTarget.value === 'start') {
        delete data.start;
      } else if (now < ignoreRangeUntil) {
        if (data.start !== undefined && Math.abs(data.start - status.value.start) < 0.05) {
          // confirmed
        } else {
          delete data.start;
        }
      }

      if (dragTarget.value === 'end') {
        delete data.end;
      } else if (now < ignoreRangeUntil) {
        if (data.end !== undefined && Math.abs(data.end - status.value.end) < 0.05) {
          // confirmed
        } else {
          delete data.end;
        }
      }

      status.value = { ...status.value, ...data };
    } catch (e) {
      // ignore
    }
  });
});

onUnmounted(() => {
  if (throttleTimer) clearTimeout(throttleTimer);
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
  padding: 14px;
  gap: 12px;
  overflow-y: auto;
  user-select: none;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  box-sizing: border-box;
}

/* Cards Base */
.card {
  background: #161b22;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.label {
  font-size: 12px;
  color: #8b949e;
  font-weight: 500;
}

.value-highlight {
  font-size: 13px;
  color: #58a6ff;
  font-weight: 700;
  font-family: 'SF Mono', Consolas, monospace;
}

/* Top Bar */
.top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.play-toggle-btn {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid rgba(63, 185, 80, 0.3);
  background: rgba(35, 134, 54, 0.25);
  color: #3fb950;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s ease;
}

.play-toggle-btn:hover {
  background: rgba(35, 134, 54, 0.4);
  border-color: #3fb950;
}

.play-toggle-btn.is-paused {
  background: rgba(31, 111, 235, 0.25);
  border-color: rgba(31, 111, 235, 0.4);
  color: #58a6ff;
}

.play-toggle-btn.is-paused:hover {
  background: rgba(31, 111, 235, 0.4);
  border-color: #58a6ff;
}

.play-toggle-btn kbd {
  font-size: 10px;
  padding: 1px 4px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.1);
  color: inherit;
  opacity: 0.8;
}

.top-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 3px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: #21262d;
  color: #c9d1d9;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: #30363d;
  color: #ffffff;
}

.retry-btn {
  color: #58a6ff;
  border-color: rgba(88, 166, 255, 0.2);
  background: rgba(88, 166, 255, 0.1);
}

.retry-btn:hover {
  background: rgba(88, 166, 255, 0.2);
  color: #79c0ff;
}

.close-btn {
  padding: 8px 9px;
}

.close-btn:hover {
  background: rgba(248, 81, 73, 0.2);
  border-color: rgba(248, 81, 73, 0.4);
  color: #f85149;
}

/* Speed Slider */
.speed-slider-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.step-btn {
  width: 26px;
  height: 26px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: #21262d;
  color: #c9d1d9;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  flex-shrink: 0;
  transition: all 0.1s ease;
}

.step-btn:hover {
  background: #30363d;
  color: #ffffff;
}

.speed-slider {
  flex: 1;
  accent-color: #58a6ff;
  height: 4px;
  background: #21262d;
  border-radius: 2px;
  cursor: pointer;
}

.speed-chips {
  display: flex;
  gap: 3px;
  justify-content: space-between;
}

.chip-btn {
  flex: 1;
  min-width: 0;
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
  font-weight: 700;
}

/* Timeline Card */
.timeline-card {
  gap: 12px;
}

.time-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 4px;
}

.jump-group {
  display: flex;
  gap: 3px;
  flex-shrink: 0;
}

.jump-chip {
  padding: 3px 5px;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: #21262d;
  color: #8b949e;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.1s ease;
}

.jump-chip:hover {
  background: #30363d;
  color: #58a6ff;
}

.digital-time-box {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: baseline;
  justify-content: center;
  gap: 3px;
  background: #0d1117;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 4px 6px;
  cursor: pointer;
  position: relative;
  transition: border-color 0.15s ease;
}

.digital-time-box:hover {
  border-color: #58a6ff;
}

.cur-time {
  font-size: 15px;
  font-weight: 700;
  font-family: 'SF Mono', Consolas, monospace;
  color: #3fb950;
  letter-spacing: 0.5px;
}

.time-sep {
  font-size: 11px;
  color: #484f58;
}

.total-time {
  font-size: 11px;
  color: #8b949e;
  font-family: 'SF Mono', Consolas, monospace;
}

.edit-pen {
  font-size: 11px;
  color: #484f58;
  margin-left: 2px;
}

.digital-input {
  font-size: 15px;
  font-weight: 700;
  font-family: 'SF Mono', Consolas, monospace;
  color: #3fb950;
  background: transparent;
  border: none;
  outline: none;
  text-align: center;
  width: 100%;
}

/* Main Interactive Timeline Track */
.timeline-track-wrap {
  position: relative;
  height: 48px;
  background: #0d1117;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  cursor: pointer;
  overflow: visible;
}

.timeline-track-wrap.dragging {
  cursor: ew-resize;
}

.track-bg-line {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  height: 6px;
  transform: translateY(-50%);
  background: #21262d;
  border-radius: 3px;
}

.track-loop-bar {
  position: absolute;
  top: 50%;
  height: 8px;
  transform: translateY(-50%);
  background: linear-gradient(90deg, rgba(88, 166, 255, 0.4), rgba(248, 81, 73, 0.4));
  border-left: 2px solid #58a6ff;
  border-right: 2px solid #f85149;
  border-radius: 4px;
}

/* Pins (A & B) */
.track-pin {
  position: absolute;
  top: 2px;
  bottom: 2px;
  width: 20px;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: ew-resize;
  z-index: 10;
}

.pin-badge {
  font-size: 9px;
  font-weight: 800;
  line-height: 14px;
  width: 16px;
  height: 14px;
  text-align: center;
  border-radius: 3px;
  color: #ffffff;
}

.pin-stem {
  flex: 1;
  width: 2px;
  margin-top: 1px;
}

.pin-start .pin-badge {
  background: #1f6feb;
  box-shadow: 0 0 6px rgba(31, 111, 235, 0.6);
}
.pin-start .pin-stem {
  background: #58a6ff;
}

.pin-end .pin-badge {
  background: #da3633;
  box-shadow: 0 0 6px rgba(218, 54, 51, 0.6);
}
.pin-end .pin-stem {
  background: #f85149;
}

/* Playback Cursor (Green) */
.track-cursor {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 14px;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: ew-resize;
  z-index: 20;
}

.cursor-head {
  width: 10px;
  height: 10px;
  background: #3fb950;
  border: 2px solid #ffffff;
  border-radius: 50%;
  box-shadow: 0 0 8px #3fb950;
  margin-top: 2px;
}

.cursor-line {
  flex: 1;
  width: 2px;
  background: #3fb950;
  box-shadow: 0 0 4px #3fb950;
}

/* Loop Control Footer */
.loop-control-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.loop-point-box {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.loop-point-box.right {
  align-items: flex-end;
}

.point-header {
  display: flex;
  align-items: center;
  gap: 4px;
}

.point-header.right {
  justify-content: flex-end;
}

.point-tag {
  font-size: 9px;
  font-weight: 700;
  padding: 1px 4px;
  border-radius: 3px;
}

.tag-a {
  background: rgba(31, 111, 235, 0.2);
  color: #58a6ff;
  border: 1px solid rgba(31, 111, 235, 0.3);
}

.tag-b {
  background: rgba(218, 54, 51, 0.2);
  color: #f85149;
  border: 1px solid rgba(218, 54, 51, 0.3);
}

.set-cur-btn {
  display: flex;
  align-items: center;
  gap: 2px;
  background: transparent;
  border: none;
  color: #8b949e;
  font-size: 9px;
  cursor: pointer;
  padding: 0;
  white-space: nowrap;
  transition: color 0.1s ease;
}

.set-cur-btn:hover {
  color: #58a6ff;
}

.point-input {
  background: #0d1117;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  color: #c9d1d9;
  font-size: 11px;
  font-family: 'SF Mono', Consolas, monospace;
  padding: 3px 6px;
  outline: none;
  width: 78px;
  box-sizing: border-box;
  transition: border-color 0.15s ease;
}

.point-input:focus {
  border-color: #58a6ff;
}

.point-input.right {
  text-align: right;
}

.reset-range-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  color: #8b949e;
  font-size: 9px;
  font-weight: 600;
  padding: 4px 6px;
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.1s ease;
}

.reset-range-btn i {
  font-size: 13px;
}

.reset-range-btn:hover {
  background: rgba(88, 166, 255, 0.12);
  border-color: rgba(88, 166, 255, 0.3);
  color: #58a6ff;
}

/* Footer Tips */
.footer-tips {
  margin-top: auto;
  text-align: center;
  font-size: 11px;
  color: #6e7681;
  padding-top: 4px;
}

.footer-tips kbd {
  background: #21262d;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  padding: 1px 4px;
  font-size: 10px;
  color: #8b949e;
}
</style>
