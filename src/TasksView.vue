<i18n>
en:
  empty: No active tasks
  empty-sub: Newly started render tasks will appear here.

  status:
    pending: Pending…
    loading: Loading…
    mixing: Mixing…
    rendering: Rendering ({ progress }%), { fps } FPS, estimated to end { estimate }
    done: Done, took { duration }
    canceled: Canceled
    failed: Failed

  cancel: Cancel Task
  confirm: Close

  details: View Error Details
  error: Error Output
  output: Render Output

  show-output: Output Logs
  show-in-folder: Show in Folder

zh-CN:
  empty: 暂无渲染任务
  empty-sub: 发起的视频渲染任务将实时在此处显示进度与状态。

  status:
    pending: 等待中…
    loading: 加载中…
    mixing: 混音中…
    rendering: 渲染中（{ progress }%），{ fps } FPS，预计 { estimate } 结束
    done: 已完成，耗时 { duration }
    canceled: 已取消
    failed: 失败

  cancel: 取消任务
  confirm: 关闭

  details: 查看错误详情
  error: 错误日志
  output: 渲染日志

  show-output: 查看输出日志
  show-in-folder: 在文件夹中显示

</i18n>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();

import type { Task, TaskStatus } from './model';
import { invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import moment from 'moment';
import { toastError } from './common';

const tasks = ref<Task[]>();

async function updateList() {
  tasks.value = await invoke<Task[]>('get_tasks');
}

await updateList();

const updateTask = setInterval(updateList, 700);
onUnmounted(() => clearInterval(updateTask));

function describeStatus(status: TaskStatus): string {
  switch (status.type) {
    case 'pending':
      return t('status.pending');
    case 'loading':
      return t('status.loading');
    case 'mixing':
      return t('status.mixing');
    case 'rendering':
      return t('status.rendering', {
        progress: (status.progress * 100).toFixed(2),
        fps: status.fps,
        estimate: status.estimate ? moment.duration(Math.ceil(status.estimate), 'seconds').humanize(true, { ss: 0, s: 60, m: 60 }) : '',
      });
    case 'done':
      return t('status.done', {
        duration: moment.duration(Math.ceil(status.duration), 'seconds').humanize(false, { ss: 0, s: 60, m: 60 }),
      });
    case 'canceled':
      return t('status.canceled');
    case 'failed':
      return t('status.failed');
  }
}

function getStatusBadge(type: string) {
  switch (type) {
    case 'done': return 'badge-success';
    case 'rendering':
    case 'loading':
    case 'mixing': return 'badge-primary';
    case 'failed': return 'badge-danger';
    case 'canceled': return 'badge-warning';
    default: return 'badge-muted';
  }
}

const errorDialog = ref(false),
  errorDialogMessage = ref('');

const outputDialog = ref(false),
  outputDialogMessage = ref('');

async function showInFolder(path: string) {
  try {
    await invoke('show_in_folder', { path });
  } catch (e) {
    toastError(e);
  }
}
</script>

<template>
  <div class="page-wrapper">
    <div class="d-flex justify-between align-center mb-2">
      <h2>{{ t('tasks') }}</h2>
      <button class="btn btn-secondary btn-sm" @click="updateList">
        <i class="mdi mdi-refresh"></i> 刷新
      </button>
    </div>

    <!-- Empty state -->
    <div v-if="!tasks || !tasks.length" class="clean-card empty-card text-center py-16">
      <i class="mdi mdi-clipboard-text-outline empty-icon"></i>
      <h3 class="mt-4">{{ t('empty') }}</h3>
      <p class="text-muted mt-1">{{ t('empty-sub') }}</p>
    </div>

    <!-- Task cards -->
    <div v-else class="tasks-list">
      <div v-for="task in tasks" :key="task.id" class="clean-card task-card">
        <div class="task-cover" :style="{ 'background-image': 'url(' + convertFileSrc(task.cover) + ')' }">
          <div v-if="!task.cover" class="cover-placeholder">
            <i class="mdi mdi-music-note"></i>
          </div>
        </div>

        <div class="task-body">
          <div class="task-header">
            <div class="task-title-group">
              <h3 class="task-name">{{ task.name }}</h3>
              <p class="task-path" :title="task.path">{{ task.path }}</p>
            </div>
            <span class="badge" :class="getStatusBadge(task.status.type)">
              {{ task.status.type.toUpperCase() }}
            </span>
          </div>

          <div class="task-status-line">
            <span class="status-desc">{{ describeStatus(task.status) }}</span>
          </div>

          <!-- Progress bar -->
          <div v-if="['loading', 'mixing', 'rendering'].includes(task.status.type)" class="progress-wrap">
            <div
              class="progress-fill"
              :class="{ 'progress-indeterminate': task.status.type !== 'rendering' }"
              :style="{ width: (task.status.type === 'rendering' ? (task.status.progress * 100) : 100) + '%' }"
            ></div>
          </div>

          <!-- Task Action buttons -->
          <div class="task-footer">
            <div class="flex-1"></div>
            <button
              v-if="['loading', 'mixing', 'rendering', 'pending'].includes(task.status.type)"
              class="btn btn-danger btn-sm"
              @click="invoke('cancel_task', { id: task.id })"
            >
              <i class="mdi mdi-close"></i> {{ t('cancel') }}
            </button>

            <button
              v-if="task.status.type === 'failed'"
              class="btn btn-secondary btn-sm"
              @click="() => {
                if (task.status.type === 'failed') {
                  errorDialogMessage = task.status.error;
                  errorDialog = true;
                }
              }"
            >
              <i class="mdi mdi-alert-circle-outline text-danger"></i> {{ t('details') }}
            </button>

            <template v-if="task.status.type === 'done'">
              <button
                class="btn btn-secondary btn-sm"
                @click="() => {
                  if (task.status.type === 'done') {
                    outputDialogMessage = task.status.output;
                    outputDialog = true;
                  }
                }"
              >
                <i class="mdi mdi-text-box-outline"></i> {{ t('show-output') }}
              </button>
              <button class="btn btn-primary btn-sm" @click="showInFolder(task.output)">
                <i class="mdi mdi-folder-open-outline"></i> {{ t('show-in-folder') }}
              </button>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- Error Modal -->
    <div v-if="errorDialog" class="modal-overlay" @click.self="errorDialog = false">
      <div class="modal-content">
        <div class="card-title text-danger">
          <i class="mdi mdi-alert-circle"></i>
          <span>{{ t('error') }}</span>
        </div>
        <pre class="log-box">{{ errorDialogMessage }}</pre>
        <div class="d-flex justify-end">
          <button class="btn btn-secondary" @click="errorDialog = false">{{ t('confirm') }}</button>
        </div>
      </div>
    </div>

    <!-- Output Logs Modal -->
    <div v-if="outputDialog" class="modal-overlay" @click.self="outputDialog = false">
      <div class="modal-content">
        <div class="card-title">
          <i class="mdi mdi-text-box-outline text-primary"></i>
          <span>{{ t('output') }}</span>
        </div>
        <pre class="log-box">{{ outputDialogMessage }}</pre>
        <div class="d-flex justify-end">
          <button class="btn btn-primary" @click="outputDialog = false">{{ t('confirm') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.empty-card {
  border: 1px dashed var(--border-color);
  background: transparent;
}

.empty-icon {
  font-size: 56px;
  color: var(--text-sub);
}

.tasks-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.task-card {
  display: flex;
  padding: 0;
  overflow: hidden;
  gap: 0;
}

.task-cover {
  width: 180px;
  min-height: 120px;
  background-position: center;
  background-repeat: no-repeat;
  background-size: cover;
  background-color: #12151c;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-right: 1px solid var(--border-color);
}

.cover-placeholder {
  font-size: 36px;
  color: var(--text-sub);
}

.task-body {
  flex: 1;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.task-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-main);
}

.task-path {
  font-size: 11px;
  color: var(--text-sub);
  margin-top: 2px;
  max-width: 500px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-status-line {
  font-size: 13px;
  color: var(--text-muted);
}

.progress-wrap {
  width: 100%;
  height: 6px;
  background: var(--bg-input);
  border-radius: 3px;
  overflow: hidden;
  margin: 6px 0;
}

.progress-fill {
  height: 100%;
  background: var(--primary);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-indeterminate {
  width: 50% !important;
  animation: indeterminate 1.5s infinite linear;
}

@keyframes indeterminate {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(250%); }
}

.task-footer {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: auto;
  padding-top: 6px;
}

.log-box {
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  padding: 12px;
  font-family: monospace;
  font-size: 12px;
  color: #cbd5e1;
  max-height: 50vh;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.d-flex { display: flex; }
.justify-between { justify-content: space-between; }
.justify-end { justify-content: flex-end; }
.align-center { align-items: center; }
.flex-1 { flex: 1; }
.mt-1 { margin-top: 4px; }
.mt-4 { margin-top: 16px; }
.mb-2 { margin-bottom: 8px; }
.py-16 { padding-top: 64px; padding-bottom: 64px; }
.text-center { text-align: center; }
.text-danger { color: var(--danger); }
.text-muted { color: var(--text-muted); }
</style>
