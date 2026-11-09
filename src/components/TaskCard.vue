<script setup lang="ts">
import { computed } from "vue";
import Icon from "./Icon.vue";
import type { Task } from "../types";
import { dueLabel, fmtClock, fmtDur, isOverdue } from "../utils";

const props = defineProps<{
  task: Task;
  running?: boolean;
  ghost?: boolean;
  elapsedMs?: number;
  dimmed?: boolean;
}>();

defineEmits<{
  (e: "open", task: Task): void;
  (e: "play", task: Task): void;
  (e: "log", task: Task): void;
  (e: "grab", ev: PointerEvent, task: Task, el: HTMLElement): void;
}>();

const overdue = computed(() => isOverdue(props.task.dueDate, new Date().toISOString().slice(0, 10)));
const due = computed(() => dueLabel(props.task.dueDate, new Date().toISOString().slice(0, 10)));
</script>

<template>
  <article
    class="card"
    :class="{ ghost, running, dimmed, done: task.status === 'done' }"
    @pointerdown="$emit('grab', $event, task, $event.currentTarget as HTMLElement)"
    @dblclick="$emit('open', task)"
  >
    <div class="card-top">
      <span class="dot" :style="{ background: task.color || 'var(--text-3)' }"></span>
      <span class="proj">{{ task.projectName || "未归类" }}</span>
      <span v-if="task.status === 'done'" class="done-mark"><Icon name="check" :size="11" /></span>
    </div>

    <div class="card-title">{{ task.title }}</div>

    <div class="card-meta">
      <span v-if="due && task.status !== 'done'" class="due" :class="{ late: overdue }">{{ due }}</span>
      <span v-if="running" class="num live">{{ fmtClock(elapsedMs || 0) }}</span>
      <span v-else-if="task.todayMinutes" class="num">{{ fmtDur(task.todayMinutes) }}</span>
      <span v-else-if="task.estimateMin" class="num faint">估 {{ fmtDur(task.estimateMin) }}</span>
      <div class="acts">
        <button class="act" title="手动补记时间" @click.stop="$emit('log', task)">
          <Icon name="timer" :size="10" />
        </button>
        <button
          class="act play"
          :class="{ on: running }"
          :title="running ? '停止计时' : '开始计时'"
          @click.stop="$emit('play', task)"
        >
          <Icon :name="running ? 'stop' : 'play'" :size="9" />
        </button>
      </div>
    </div>
  </article>
</template>

<style scoped>
.card {
  position: relative;
  background: var(--surface);
  border: 0.5px solid var(--line);
  border-radius: var(--r-sm);
  padding: 7px 9px 6px;
  margin-bottom: 5px;
  cursor: grab;
  transition: border-color 0.14s ease, background-color 0.14s ease;
  touch-action: none;
}

.card:hover {
  border-color: var(--line-strong);
}

.card.ghost {
  box-shadow: var(--shadow-lift);
  border-color: var(--line-strong);
  cursor: grabbing;
  margin: 0;
  width: 100%;
}

.card.dimmed {
  opacity: 0.32;
  border-style: dashed;
}

.card.running {
  border-color: var(--accent-line);
  background: color-mix(in srgb, var(--accent) 5%, var(--surface));
}

.card.done {
  opacity: 0.72;
}

.card-top {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  color: var(--text-3);
  line-height: 1.3;
}

.proj {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.done-mark {
  margin-left: auto;
  color: var(--ok);
  display: inline-flex;
}

.card-title {
  font-size: 12.5px;
  line-height: 1.45;
  margin: 4px 0 6px;
  font-weight: 500;
  color: var(--text);
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
}

.card.done .card-title {
  color: var(--text-2);
}

.card-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-3);
  min-height: 17px;
}

.due {
  background: var(--sunken);
  border-radius: 4px;
  padding: 0 5px;
  line-height: 16px;
}

.due.late {
  color: var(--danger);
  background: var(--danger-soft);
}

.live {
  color: var(--accent);
  font-weight: 500;
}

.faint {
  color: var(--text-3);
  opacity: 0.8;
}

.acts {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 4px;
}

.act {
  width: 17px;
  height: 17px;
  border-radius: 50%;
  border: 0.5px solid var(--line-strong);
  display: grid;
  place-items: center;
  color: var(--text-3);
  opacity: 0;
  transition: opacity 0.12s ease, color 0.12s ease, border-color 0.12s ease;
}

.card:hover .act,
.card.running .act {
  opacity: 1;
}

.act:hover {
  color: var(--accent);
  border-color: var(--accent);
}

.act.on {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}

@media (hover: none) {
  .act {
    opacity: 1;
  }
}
</style>
