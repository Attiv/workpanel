<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Icon from "./Icon.vue";
import {
  openProjectModal,
  openTaskModal,
  setView,
  startTask,
  state,
} from "../store";
import { fmtDayShort, fmtDur, shiftISO, weekdayZh, parseISO } from "../utils";
import type { Project } from "../types";

const selectedId = ref<number | null>(state.snap?.projects[0]?.id ?? null);

watch(
  () => state.snap?.projects,
  (list) => {
    if (!list?.length) {
      selectedId.value = null;
      return;
    }
    if (!list.some((p) => p.id === selectedId.value)) selectedId.value = list[0].id;
  },
);

const project = computed(
  () => state.snap?.projects.find((p) => p.id === selectedId.value) ?? null,
);

const days = computed(() => {
  const out: { iso: string; label: string; weekday: string }[] = [];
  for (let i = 6; i >= 0; i--) {
    const iso = shiftISO(state.date, -i);
    out.push({ iso, label: fmtDayShort(iso), weekday: weekdayZh(parseISO(iso)) });
  }
  return out;
});

const bars = computed(() => {
  const series = state.snap?.series ?? [];
  const rows = days.value.map((d) => {
    const minutes = series
      .filter((s) => s.day === d.iso && s.projectId === selectedId.value)
      .reduce((sum, s) => sum + s.minutes, 0);
    return { ...d, minutes };
  });
  const max = Math.max(30, ...rows.map((r) => r.minutes));
  return rows.map((r) => ({ ...r, pct: Math.round((r.minutes / max) * 100) }));
});

const todayTasks = computed(() =>
  (state.snap?.tasks ?? []).filter(
    (t) => t.projectId === selectedId.value && t.plannedDate === state.date && t.status !== "done",
  ),
);

const openTasks = computed(() =>
  (state.snap?.tasks ?? []).filter(
    (t) =>
      t.projectId === selectedId.value &&
      t.status !== "done" &&
      !(t.plannedDate === state.date),
  ),
);

function pick(p: Project) {
  selectedId.value = p.id;
}
</script>

<template>
  <section class="view">
    <header class="head">
      <h1>项目</h1>
      <span class="muted num">{{ state.snap?.projects.length || 0 }}</span>
      <div style="flex: 1"></div>
      <button class="btn primary" @click="openProjectModal()">
        <Icon name="plus" :size="12" />项目
      </button>
    </header>

    <div class="split">
      <div class="list">
        <button
          v-for="p in state.snap?.projects || []"
          :key="p.id"
          class="row"
          :class="{ on: p.id === selectedId }"
          @click="pick(p)"
        >
          <span class="bar" :style="{ background: p.color }"></span>
          <span class="pname">{{ p.name }}</span>
          <span class="pnum num">{{ p.weekMinutes ? fmtDur(p.weekMinutes) : "—" }}</span>
        </button>
        <p v-if="!state.snap?.projects.length" class="empty">还没有项目，先建一个。</p>
      </div>

      <div v-if="project" class="detail">
        <div class="d-head">
          <span class="dot" :style="{ background: project.color }"></span>
          <h2>{{ project.name }}</h2>
          <button class="btn ghost icon" title="编辑项目" @click="openProjectModal(project)">
            <Icon name="edit" :size="13" />
          </button>
          <div style="flex: 1"></div>
          <button class="btn" @click="openTaskModal(undefined, 'today')">＋ 任务</button>
        </div>

        <div class="metrics">
          <div class="metric">
            <span>本周</span>
            <b class="num">{{ fmtDur(project.weekMinutes) }}</b>
          </div>
          <div class="metric">
            <span>今日</span>
            <b class="num">{{ fmtDur(project.todayMinutes) }}</b>
          </div>
          <div class="metric">
            <span>未完成</span>
            <b class="num">{{ project.openTasks }}</b>
          </div>
          <div class="metric">
            <span>累计</span>
            <b class="num">{{ fmtDur(project.totalMinutes) }}</b>
          </div>
        </div>

        <div class="block">
          <div class="block-title">近七天</div>
          <div class="bars">
            <div v-for="b in bars" :key="b.iso" class="bcol" :title="`${b.label} ${fmtDur(b.minutes)}`">
              <div class="bfill-wrap">
                <div class="bfill" :style="{ height: Math.max(2, b.pct) + '%' }"></div>
              </div>
              <span class="blabel">{{ b.weekday }}</span>
            </div>
          </div>
        </div>

        <div class="block">
          <div class="block-title">
            今天要做
            <span v-if="!todayTasks.length" class="muted">没有排进今天的任务</span>
          </div>
          <div v-if="todayTasks.length" class="tasks">
            <div v-for="t in todayTasks" :key="t.id" class="trow" @click="openTaskModal(t)">
              <span class="dot" :style="{ background: project.color }"></span>
              <span class="tname">{{ t.title }}</span>
              <span v-if="t.todayMinutes" class="num muted">{{ fmtDur(t.todayMinutes) }}</span>
              <button class="play" title="开始计时" @click.stop="startTask(t)">
                <Icon :name="state.snap?.timer?.taskId === t.id ? 'stop' : 'play'" :size="8" />
              </button>
            </div>
          </div>
        </div>

        <div class="block">
          <div class="block-title">
            其他未完成
            <span v-if="!openTasks.length" class="muted">都清空了</span>
          </div>
          <div v-if="openTasks.length" class="tasks">
            <div v-for="t in openTasks" :key="t.id" class="trow" @click="openTaskModal(t)">
              <span class="dot" :style="{ background: project.color }"></span>
              <span class="tname">{{ t.title }}</span>
              <span v-if="t.plannedDate" class="num muted">{{ fmtDayShort(t.plannedDate) }}</span>
            </div>
          </div>
        </div>

        <button class="all" @click="setView('tasks')">
          在任务列表里看全部<Icon name="arrow" :size="12" />
        </button>
      </div>

      <div v-else class="detail empty-detail">
        <p>选左边的项目，或者新建一个。</p>
      </div>
    </div>
  </section>
</template>

<style scoped>
.view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 14px 16px 16px;
  gap: 12px;
  min-height: 0;
}

.head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex: none;
}

.head h1 {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.split {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 218px minmax(0, 1fr);
  gap: 12px;
}

.list {
  background: var(--sunken);
  border-radius: var(--r);
  padding: 6px;
  overflow-y: auto;
}

.row {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  height: 32px;
  padding: 0 7px;
  border-radius: 6px;
  font-size: 12.5px;
  color: var(--text-2);
  border: 0.5px solid transparent;
}

.row:hover {
  background: color-mix(in srgb, var(--text) 5%, transparent);
}

.row.on {
  background: var(--surface);
  border-color: var(--line);
  color: var(--text);
  font-weight: 500;
}

.bar {
  width: 3px;
  height: 14px;
  border-radius: 2px;
  flex: none;
}

.pname {
  flex: 1;
  text-align: left;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pnum {
  font-size: 11px;
  color: var(--text-3);
}

.detail {
  background: var(--surface);
  border: 0.5px solid var(--line);
  border-radius: var(--r);
  padding: 14px 16px;
  overflow-y: auto;
  min-height: 0;
}

.d-head {
  display: flex;
  align-items: center;
  gap: 7px;
}

.d-head h2 {
  margin: 0;
  font-size: 15px;
  font-weight: 500;
}

.metrics {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
  margin: 14px 0 4px;
}

.metric {
  background: var(--sunken);
  border-radius: 8px;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.metric span {
  font-size: 11px;
  color: var(--text-3);
}

.metric b {
  font-size: 15px;
  font-weight: 500;
}

.block {
  margin-top: 16px;
}

.block-title {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 11.5px;
  color: var(--text-3);
  margin-bottom: 8px;
}

.bars {
  display: flex;
  gap: 6px;
  height: 64px;
}

.bcol {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.bfill-wrap {
  flex: 1;
  display: flex;
  align-items: flex-end;
  background: var(--sunken);
  border-radius: 4px;
  overflow: hidden;
}

.bfill {
  width: 100%;
  background: var(--accent);
  opacity: 0.75;
  border-radius: 3px;
  transition: height 0.25s ease;
}

.blabel {
  text-align: center;
  font-size: 10.5px;
  color: var(--text-3);
}

.tasks {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.trow {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 7px;
  border-radius: 6px;
  font-size: 12.5px;
  color: var(--text);
}

.trow:hover {
  background: var(--sunken);
}

.tname {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.play {
  width: 19px;
  height: 19px;
  border-radius: 50%;
  border: 0.5px solid var(--line-strong);
  display: grid;
  place-items: center;
  color: var(--text-3);
  opacity: 0;
}

.trow:hover .play {
  opacity: 1;
}

.play:hover {
  color: var(--accent);
  border-color: var(--accent);
}

.all {
  margin-top: 18px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  color: var(--accent);
}

.empty,
.empty-detail p {
  color: var(--text-3);
  font-size: 12.5px;
  padding: 18px 10px;
}

.empty-detail {
  display: grid;
  place-items: center;
}
</style>
