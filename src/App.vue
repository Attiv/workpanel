<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from "vue";
import Sidebar from "./components/Sidebar.vue";
import BoardView from "./components/BoardView.vue";
import ProjectsView from "./components/ProjectsView.vue";
import TasksView from "./components/TasksView.vue";
import ReportView from "./components/ReportView.vue";
import SettingsView from "./components/SettingsView.vue";
import TaskDialog from "./components/TaskDialog.vue";
import EntryDialog from "./components/EntryDialog.vue";
import ProjectDialog from "./components/ProjectDialog.vue";
import Icon from "./components/Icon.vue";
import {
  closeModal,
  openEntryModal,
  openProjectModal,
  openTaskModal,
  refresh,
  setView,
  state,
  stopTimerFlow,
  type View,
} from "./store";
import * as api from "./api";
import { fmtClock } from "./utils";
import { setLocale, t } from "./i18n";
import { applyZoom, zoomIn, zoomOut, zoomReset } from "./zoom";

const views = {
  today: BoardView,
  projects: ProjectsView,
  tasks: TasksView,
  report: ReportView,
  settings: SettingsView,
};

const current = computed(() => views[state.view]);

const timer = computed(() => state.snap?.timer ?? null);

const elapsed = computed(() => {
  if (!timer.value) return 0;
  return Math.max(0, state.now - new Date(timer.value.startedAt).getTime());
});

let tick = 0;

function onKey(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey;
  if (!mod) return;
  if (e.key === "=" || e.key === "+") {
    e.preventDefault();
    zoomIn();
  } else if (e.key === "-" || e.key === "_") {
    e.preventDefault();
    zoomOut();
  } else if (e.key === "0") {
    e.preventDefault();
    zoomReset();
  }
}

onMounted(async () => {
  await applyZoom();
  window.addEventListener("keydown", onKey);
  await refresh();
  try {
    const info = await api.appInfo();
    if (info.view && info.view in views) setView(info.view as View);
    if (info.zoom) await applyZoom(info.zoom, false);
    if (info.lang === "zh" || info.lang === "en") setLocale(info.lang);
    const [kind, arg] = info.modal.split(":");
    const id = Number(arg);
    if (kind === "task") {
      openTaskModal(id ? state.snap?.tasks.find((t) => t.id === id) : undefined);
    } else if (kind === "entry") {
      openEntryModal(id ? state.snap?.entries.find((e) => e.id === id) : undefined);
    } else if (kind === "project") {
      openProjectModal();
    }
  } catch {
    /* ignore */
  }
  tick = window.setInterval(() => {
    state.now = Date.now();
  }, 1000);
});

onBeforeUnmount(() => {
  window.clearInterval(tick);
  window.removeEventListener("keydown", onKey);
});
</script>

<template>
  <div class="app">
    <Sidebar />

    <main class="main">
      <div v-if="state.loading" class="loading">{{ t("app.loading") }}</div>
      <div v-else-if="state.error" class="failure">
        <p class="failure-title">{{ t("app.errorTitle") }}</p>
        <pre class="failure-msg">{{ state.error }}</pre>
        <button class="btn" @click="refresh">{{ t("app.retry") }}</button>
      </div>
      <component :is="current" v-else />
    </main>

    <Transition name="fade">
      <div v-if="timer" class="timer">
        <span class="live-dot"></span>
        <div class="t-text">
          <b>{{ timer.projectName || t("common.unassigned") }}</b>
          <span>{{ timer.taskTitle || timer.note || t("app.tracking") }}</span>
        </div>
        <span class="clock num">{{ fmtClock(elapsed) }}</span>
        <button class="stop" :title="t('app.stopAndLog')" @click="stopTimerFlow()">
          <Icon name="stop" :size="10" />
        </button>
      </div>
    </Transition>

    <div class="toasts">
      <TransitionGroup name="toast">
        <div v-for="item in state.toasts" :key="item.id" class="toast" :class="item.kind">
          {{ item.text }}
        </div>
      </TransitionGroup>
    </div>

    <TaskDialog
      v-if="state.modal?.type === 'task'"
      :task="state.modal.task"
      :column="state.modal.column"
      @close="closeModal"
    />
    <EntryDialog
      v-else-if="state.modal?.type === 'entry'"
      :entry="state.modal.entry"
      :day="state.modal.day"
      :task="state.modal.task"
      @close="closeModal"
    />
    <ProjectDialog
      v-else-if="state.modal?.type === 'project'"
      :project="state.modal.project"
      @close="closeModal"
    />
  </div>
</template>

<style scoped>
.app {
  display: grid;
  grid-template-columns: 208px minmax(0, 1fr);
  height: 100%;
  overflow: hidden;
}

.main {
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.loading {
  height: 100%;
  display: grid;
  place-items: center;
  color: var(--text-3);
  font-size: 12.5px;
}

.failure {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
}

.failure-title {
  margin: 0;
  font-size: 13px;
  font-weight: 500;
  color: var(--danger);
}

.failure-msg {
  margin: 0;
  max-width: 620px;
  white-space: pre-wrap;
  font-size: 12px;
  color: var(--text-2);
  background: var(--sunken);
  padding: 10px 12px;
  border-radius: 8px;
}

.timer {
  position: fixed;
  right: 20px;
  bottom: 18px;
  z-index: 70;
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 7px 9px 7px 11px;
  border-radius: 10px;
  background: var(--surface);
  border: 0.5px solid var(--line-strong);
  box-shadow: 0 10px 26px rgba(16, 22, 32, 0.14);
  max-width: 360px;
}

.live-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--ok);
  flex: none;
}

.t-text {
  display: flex;
  flex-direction: column;
  line-height: 1.25;
  min-width: 0;
}

.t-text b {
  font-size: 10.5px;
  font-weight: 500;
  color: var(--text-3);
}

.t-text span {
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 190px;
}

.clock {
  font-size: 13.5px;
  font-weight: 500;
  letter-spacing: 0.01em;
}

.stop {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  display: grid;
  place-items: center;
  background: var(--accent-soft);
  color: var(--accent);
}

.stop:hover {
  background: var(--accent);
  color: #fff;
}

.toasts {
  position: fixed;
  left: 220px;
  bottom: 18px;
  z-index: 100;
  display: flex;
  flex-direction: column;
  gap: 6px;
  pointer-events: none;
}

.toast {
  background: var(--text);
  color: var(--bg);
  font-size: 12px;
  padding: 6px 11px;
  border-radius: 7px;
  max-width: 380px;
  box-shadow: 0 8px 20px rgba(10, 14, 20, 0.18);
}

.toast.err {
  background: var(--danger);
  color: #fff;
}

.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.16s ease, transform 0.16s cubic-bezier(0.22, 1, 0.36, 1);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(4px);
}
</style>
