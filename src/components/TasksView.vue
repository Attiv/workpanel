<script setup lang="ts">
import { computed, ref } from "vue";
import Icon from "./Icon.vue";
import { openTaskModal, startTask, state, toggleDone } from "../store";
import { dueLabel, fmtDur, isOverdue } from "../utils";
import type { Task } from "../types";

type Filter = "all" | "backlog" | "today" | "doing" | "done";

const filter = ref<Filter>("all");
const query = ref("");

const tabs = computed(() => [
  { key: "all" as Filter, label: "全部", n: state.snap?.tasks.filter((t) => t.status !== "done").length || 0 },
  { key: "today" as Filter, label: "今天", n: state.snap?.tasks.filter((t) => t.plannedDate === state.date && t.status !== "done").length || 0 },
  { key: "doing" as Filter, label: "进行中", n: state.snap?.tasks.filter((t) => t.status === "doing").length || 0 },
  { key: "done" as Filter, label: "已完成", n: state.snap?.tasks.filter((t) => t.status === "done").length || 0 },
]);

const list = computed(() => {
  let rows = state.snap?.tasks ?? [];
  const q = query.value.trim().toLowerCase();

  if (filter.value === "today") rows = rows.filter((t) => t.plannedDate === state.date);
  else if (filter.value === "doing") rows = rows.filter((t) => t.status === "doing");
  else if (filter.value === "done") rows = rows.filter((t) => t.status === "done");
  else rows = rows.filter((t) => t.status !== "done" || t.todayMinutes > 0);

  if (q) rows = rows.filter((t) => (t.title + " " + (t.projectName || "")).toLowerCase().includes(q));

  return rows.slice().sort((a, b) => {
    const sa = a.status === "doing" ? 0 : a.status === "todo" ? 1 : 2;
    const sb = b.status === "doing" ? 0 : b.status === "todo" ? 1 : 2;
    if (sa !== sb) return sa - sb;
    const da = a.dueDate || "9999-99-99";
    const db = b.dueDate || "9999-99-99";
    if (da !== db) return da < db ? -1 : 1;
    return b.id - a.id;
  });
});

function overdue(t: Task) {
  return t.status !== "done" && isOverdue(t.dueDate, state.date);
}
</script>

<template>
  <section class="view">
    <header class="head">
      <h1>任务</h1>
      <div style="flex: 1"></div>
      <button class="btn primary" @click="openTaskModal(undefined, 'today')">
        <Icon name="plus" :size="12" />新任务
      </button>
    </header>

    <div class="toolbar">
      <div class="tabs">
        <button
          v-for="t in tabs"
          :key="t.key"
          class="tab"
          :class="{ on: filter === t.key }"
          @click="filter = t.key"
        >
          {{ t.label }}<span class="tn num">{{ t.n }}</span>
        </button>
      </div>
      <div class="search">
        <Icon name="search" :size="13" />
        <input v-model="query" class="sinput" placeholder="搜任务名或项目" />
      </div>
    </div>

    <div class="table">
      <div v-for="t in list" :key="t.id" class="trow" @dblclick="openTaskModal(t)">
        <button
          class="check"
          :class="{ on: t.status === 'done' }"
          :title="t.status === 'done' ? '标记为未完成' : '标记完成'"
          @click.stop="toggleDone(t)"
        >
          <Icon v-if="t.status === 'done'" name="check" :size="9" />
        </button>

        <div class="main" @click="openTaskModal(t)">
          <span class="title" :class="{ done: t.status === 'done' }">{{ t.title }}</span>
          <span v-if="t.note" class="note">{{ t.note }}</span>
        </div>

        <span class="proj">
          <i class="dot" :style="{ background: t.color || 'var(--text-3)' }"></i>
          <span>{{ t.projectName || "未归类" }}</span>
        </span>

        <span class="tag">
          <em v-if="t.plannedDate === state.date" class="tchip">今天</em>
          <em v-else-if="t.plannedDate" class="tdate">{{ t.plannedDate.slice(5) }}</em>
        </span>

        <span class="due" :class="{ late: overdue(t) }">
          {{ t.dueDate ? dueLabel(t.dueDate, state.date) : "" }}
        </span>

        <span class="mins num">{{ t.todayMinutes ? fmtDur(t.todayMinutes) : "" }}</span>

        <button
          class="play"
          :class="{ on: state.snap?.timer?.taskId === t.id }"
          :title="state.snap?.timer?.taskId === t.id ? '停止' : '开始计时'"
          @click.stop="startTask(t)"
        >
          <Icon :name="state.snap?.timer?.taskId === t.id ? 'stop' : 'play'" :size="8" />
        </button>
      </div>

      <p v-if="!list.length" class="empty">这里没有任务。</p>
    </div>
  </section>
</template>

<style scoped>
.view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 14px 16px 16px;
  gap: 10px;
  min-height: 0;
}

.head {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: none;
}

.head h1 {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: none;
}

.tabs {
  display: flex;
  gap: 2px;
  background: var(--sunken);
  padding: 2px;
  border-radius: 7px;
}

.tab {
  height: 24px;
  padding: 0 9px;
  border-radius: 5px;
  font-size: 12px;
  color: var(--text-3);
  display: flex;
  align-items: center;
  gap: 5px;
}

.tab:hover {
  color: var(--text-2);
}

.tab.on {
  background: var(--surface);
  color: var(--text);
  font-weight: 500;
  border: 0.5px solid var(--line);
}

.tn {
  font-size: 10.5px;
  color: var(--text-3);
}

.search {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 9px;
  border: 0.5px solid var(--line-strong);
  border-radius: 7px;
  background: var(--surface);
  color: var(--text-3);
  width: 220px;
}

.sinput {
  border: none;
  outline: none;
  background: transparent;
  font-size: 12.5px;
  width: 100%;
  color: var(--text);
}

.table {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  background: var(--surface);
  border: 0.5px solid var(--line);
  border-radius: var(--r);
  padding: 4px;
}

.trow {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 9px;
  height: 37px;
  border-radius: 0;
  font-size: 12.5px;
}

.trow + .trow {
  border-top: 0.5px solid var(--line);
}

.trow:hover {
  background: var(--sunken);
}

.check {
  width: 16px;
  height: 16px;
  flex: none;
  border-radius: 50%;
  border: 1.2px solid var(--line-strong);
  display: grid;
  place-items: center;
  color: #fff;
}

.check:hover {
  border-color: var(--ok);
}

.check.on {
  background: var(--ok);
  border-color: var(--ok);
}

.main {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: 8px;
  overflow: hidden;
}

.title {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.title.done {
  color: var(--text-3);
  text-decoration: line-through;
}

.note {
  font-size: 11.5px;
  color: var(--text-3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 0 1 auto;
  min-width: 0;
}

.proj {
  width: 120px;
  flex: none;
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 11.5px;
  color: var(--text-2);
  overflow: hidden;
  white-space: nowrap;
}

.proj i {
  flex: none;
}

.proj span {
  overflow: hidden;
  text-overflow: ellipsis;
}

.tag {
  width: 44px;
  flex: none;
  display: flex;
  align-items: center;
  font-size: 11px;
  color: var(--text-3);
  white-space: nowrap;
}

.tchip {
  font-style: normal;
  font-size: 10.5px;
  color: var(--accent);
  background: var(--accent-soft);
  border-radius: 4px;
  padding: 2px 5px;
  line-height: 1;
}

.tdate {
  font-style: normal;
  color: var(--text-3);
}

.due {
  width: 76px;
  flex: none;
  font-size: 11px;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.due.late {
  color: var(--danger);
}

.mins {
  width: 44px;
  flex: none;
  font-size: 11px;
  color: var(--text-2);
  text-align: right;
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

.play.on {
  opacity: 1;
  color: var(--accent);
  border-color: var(--accent);
}

.empty {
  color: var(--text-3);
  font-size: 12.5px;
  text-align: center;
  padding: 30px 0;
}
</style>
