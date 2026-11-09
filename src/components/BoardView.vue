<script setup lang="ts">
import { computed, onBeforeUnmount, reactive, ref } from "vue";
import Icon from "./Icon.vue";
import TaskCard from "./TaskCard.vue";
import {
  moveTask,
  openEntryModal,
  openTaskModal,
  setDate,
  startTask,
  state,
  stepDate,
  type Column,
} from "../store";
import { fmtDay, fmtDur, todayISO } from "../utils";
import { t } from "../i18n";
import type { Task } from "../types";

const boardEl = ref<HTMLElement | null>(null);

const columns = computed(() => {
  const all = state.snap?.tasks ?? [];
  const date = state.date;

  const backlog = all
    .filter((t) => t.status === "todo" && t.plannedDate !== date)
    .sort((a, b) => {
      const da = a.dueDate || "9999-99-99";
      const db = b.dueDate || "9999-99-99";
      if (da !== db) return da < db ? -1 : 1;
      return a.id - b.id;
    });

  const today = all
    .filter((t) => t.status === "todo" && t.plannedDate === date)
    .sort((a, b) => a.id - b.id);

  const doing = all.filter((t) => t.status === "doing");
  const done = all
    .filter((t) => t.status === "done" && (t.doneAt || "").slice(0, 10) === date)
    .sort((a, b) => ((a.doneAt || "") < (b.doneAt || "") ? 1 : -1));

  const est = today.reduce((s, t) => s + (t.estimateMin || 0), 0);
  const rec = today.reduce((s, t) => s + (t.todayMinutes || 0), 0);

  return [
    {
      key: "backlog" as Column,
      title: t("board.backlog"),
      tasks: backlog,
      meta: `${backlog.length}`,
      hint: t("board.hintBacklog"),
    },
    {
      key: "today" as Column,
      title: t("board.today"),
      tasks: today,
      meta: `${today.length}${est ? " · " + t("board.est") + " " + fmtDur(est) : ""}${
        rec ? " · " + t("board.logged") + " " + fmtDur(rec) : ""
      }`,
      hint: t("board.hintToday"),
    },
    {
      key: "doing" as Column,
      title: t("board.doing"),
      tasks: doing,
      meta: `${doing.length}`,
      hint: t("board.hintDoing"),
    },
    {
      key: "done" as Column,
      title: t("board.done"),
      tasks: done,
      meta: done.length ? fmtDur(done.reduce((s, t) => s + t.todayMinutes, 0)) : "0",
      hint: t("board.hintDone"),
    },
  ];
});

const runningId = computed(() => state.snap?.timer?.taskId ?? null);

const elapsedMs = computed(() => {
  const started = state.snap?.timer?.startedAt;
  if (!started) return 0;
  return state.now - new Date(started).getTime();
});

const progress = computed(() => {
  const target = state.snap?.settings.workdayMinutes || 480;
  const done = state.snap?.stats.todayMinutes || 0;
  return Math.min(100, Math.round((done / target) * 100));
});

const drag = reactive({
  active: false,
  task: null as Task | null,
  from: "" as Column | "",
  over: "" as Column | "",
  x: 0,
  y: 0,
  ox: 0,
  oy: 0,
  w: 0,
});

let startX = 0;
let startY = 0;
let pending: { task: Task; from: Column } | null = null;
let rects: { key: Column; r: DOMRect }[] = [];

function onGrab(e: PointerEvent, task: Task, el: HTMLElement) {
  if (e.button !== 0) return;
  if ((e.target as HTMLElement).closest("button")) return;
  const colEl = el.closest("[data-col]") as HTMLElement | null;
  const from = colEl?.dataset.col as Column | undefined;
  if (!from) return;

  const r = el.getBoundingClientRect();
  pending = { task, from };
  startX = e.clientX;
  startY = e.clientY;
  drag.ox = e.clientX - r.left;
  drag.oy = e.clientY - r.top;
  drag.w = r.width;
  window.addEventListener("pointermove", onMove, { passive: false });
  window.addEventListener("pointerup", onUp);
  window.addEventListener("pointercancel", onUp);
}

function onMove(e: PointerEvent) {
  if (!pending) return;
  if (!drag.active) {
    if (Math.hypot(e.clientX - startX, e.clientY - startY) < 6) return;
    drag.active = true;
    drag.task = pending.task;
    drag.from = pending.from;
    rects = Array.from(boardEl.value?.querySelectorAll<HTMLElement>("[data-col]") || []).map(
      (el) => ({ key: el.dataset.col as Column, r: el.getBoundingClientRect() }),
    );
    document.body.classList.add("is-dragging");
  }
  e.preventDefault();
  drag.x = e.clientX;
  drag.y = e.clientY;
  const hit = rects.find(
    (c) => e.clientX >= c.r.left && e.clientX <= c.r.right && e.clientY >= c.r.top && e.clientY <= c.r.bottom,
  );
  drag.over = hit ? hit.key : "";
}

async function onUp() {
  window.removeEventListener("pointermove", onMove);
  window.removeEventListener("pointerup", onUp);
  window.removeEventListener("pointercancel", onUp);
  document.body.classList.remove("is-dragging");

  const task = drag.task;
  const to = drag.over;
  const from = drag.from;
  const active = drag.active;
  drag.active = false;
  drag.task = null;
  drag.over = "";
  drag.from = "";
  pending = null;

  if (active && task && to && to !== from) {
    await moveTask(task, to);
  }
}

onBeforeUnmount(() => {
  window.removeEventListener("pointermove", onMove);
  window.removeEventListener("pointerup", onUp);
  window.removeEventListener("pointercancel", onUp);
});

const ghostStyle = computed(() => ({
  width: `${drag.w}px`,
  transform: `translate3d(${drag.x - drag.ox}px, ${drag.y - drag.oy}px, 0) scale(1.015)`,
}));

const isToday = computed(() => state.date === todayISO());
</script>

<template>
  <section class="view">
    <header class="head">
      <div class="date">
        <button class="btn icon ghost" :title="t('board.prevDay')" @click="stepDate(-1)">
          <Icon name="left" :size="14" />
        </button>
        <h1>{{ fmtDay(state.date) }}</h1>
        <button class="btn icon ghost" :title="t('board.nextDay')" @click="stepDate(1)">
          <Icon name="right" :size="14" />
        </button>
        <button v-if="!isToday" class="btn ghost today-btn" @click="setDate(todayISO())">
          {{ t("board.backToToday") }}
        </button>
      </div>

      <div class="head-right">
        <div class="load">
          <div class="load-text num">
            <b>{{ fmtDur(state.snap?.stats.todayMinutes || 0) }}</b>
            <span>/ {{ fmtDur(state.snap?.settings.workdayMinutes || 480) }}</span>
          </div>
          <div class="bar"><i :style="{ width: progress + '%' }"></i></div>
        </div>
        <button class="btn" @click="openEntryModal()">{{ t("board.logTime") }}</button>
        <button class="btn primary" @click="openTaskModal(undefined, 'today')">
          <Icon name="plus" :size="12" />{{ t("board.newTask") }}
        </button>
      </div>
    </header>

    <div ref="boardEl" class="board">
      <div
        v-for="col in columns"
        :key="col.key"
        class="col"
        :data-col="col.key"
        :class="{ today: col.key === 'today', over: drag.over === col.key && drag.from !== col.key }"
      >
        <div class="col-head">
          <span class="col-title">{{ col.title }}</span>
          <span class="col-meta num">{{ col.meta }}</span>
        </div>

        <div class="col-body">
          <TaskCard
            v-for="task in col.tasks"
            :key="task.id"
            :task="task"
            :running="runningId === task.id"
            :elapsed-ms="elapsedMs"
            :dimmed="drag.active && drag.task?.id === task.id"
            @grab="onGrab"
            @open="openTaskModal($event)"
            @play="startTask($event)"
            @log="openEntryModal(undefined, undefined, $event)"
          />
          <div v-if="!col.tasks.length" class="hint">{{ col.hint }}</div>
          <button
            v-if="col.key === 'backlog' || col.key === 'today'"
            class="add"
            @click="openTaskModal(undefined, col.key)"
          >
            <Icon name="plus" :size="11" /> {{ t("board.addTask") }}
          </button>
        </div>
      </div>
    </div>

    <Teleport to="body">
      <div v-if="drag.active && drag.task" class="drag-layer" :style="ghostStyle">
        <TaskCard :task="drag.task" ghost />
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.view {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  padding: 14px 16px 16px;
  gap: 12px;
}

.head {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: none;
}

.date {
  display: flex;
  align-items: center;
  gap: 3px;
}

.date h1 {
  font-size: 16px;
  font-weight: 500;
  margin: 0 4px;
  min-width: 128px;
}

.today-btn {
  margin-left: 6px;
  height: 24px;
  font-size: 12px;
  color: var(--text-2);
}

.head-right {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 8px;
}

.load {
  display: flex;
  flex-direction: column;
  gap: 3px;
  width: 92px;
  margin-right: 4px;
}

.load-text {
  display: flex;
  align-items: baseline;
  gap: 3px;
  font-size: 11.5px;
  color: var(--text-3);
  justify-content: flex-end;
}

.load-text b {
  font-weight: 500;
  color: var(--text);
}

.bar {
  height: 3px;
  border-radius: 2px;
  background: var(--sunken-2);
  overflow: hidden;
}

.bar i {
  display: block;
  height: 100%;
  border-radius: 2px;
  background: var(--accent);
  transition: width 0.3s ease;
}

.board {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 9px;
}

.col {
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--sunken);
  border-radius: var(--r);
  padding: 7px 7px 6px;
  border: 1px solid transparent;
  transition: background-color 0.14s ease, border-color 0.14s ease;
}

.col.today {
  background: var(--accent-soft);
}

.col.over {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 9%, var(--sunken));
}

.col-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  padding: 1px 3px 7px;
  flex: none;
}

.col-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-2);
}

.col.today .col-title {
  color: var(--accent);
}

.col-meta {
  font-size: 11px;
  color: var(--text-3);
  white-space: nowrap;
}

.col-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding-right: 1px;
}

.hint {
  border: 1px dashed var(--line-strong);
  border-radius: var(--r-sm);
  padding: 14px 10px;
  text-align: center;
  font-size: 11.5px;
  color: var(--text-3);
  line-height: 1.5;
}

.col.today .hint {
  border-color: var(--accent-line);
  color: var(--accent);
}

.add {
  width: 100%;
  height: 27px;
  border-radius: var(--r-sm);
  color: var(--text-3);
  font-size: 11.5px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  border: 1px dashed transparent;
  transition: background-color 0.12s ease, color 0.12s ease;
}

.add:hover {
  background: color-mix(in srgb, var(--text) 5%, transparent);
  color: var(--text-2);
}

.drag-layer {
  position: fixed;
  left: 0;
  top: 0;
  z-index: 80;
  pointer-events: none;
  will-change: transform;
}
</style>

<style>
body.is-dragging {
  cursor: grabbing;
}

body.is-dragging * {
  cursor: grabbing !important;
}
</style>
