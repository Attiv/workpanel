<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import * as api from "../api";
import Icon from "./Icon.vue";
import Modal from "./Modal.vue";
import { closeModal, deleteEntryFlow, saveEntryFlow, state, toast } from "../store";
import { parseDuration, shiftISO } from "../utils";
import type { Entry, Task } from "../types";

const props = defineProps<{ entry?: Entry; day?: string; task?: Task }>();

const projectId = ref<number | null>(
  props.entry?.projectId ?? props.task?.projectId ?? state.snap?.projects[0]?.id ?? null,
);
const taskId = ref<number | null>(props.entry?.taskId ?? props.task?.id ?? null);
const day = ref(props.entry?.day ?? props.day ?? state.date);
const minutesText = ref(props.entry ? String(props.entry.minutes) : "");
const note = ref(props.entry?.note ?? "");
const error = ref("");
const busy = ref(false);
const confirmDelete = ref(false);
const noteEl = ref<HTMLTextAreaElement | null>(null);

const projects = computed(() => state.snap?.projects ?? []);
const tasks = computed(() =>
  (state.snap?.tasks ?? []).filter((t) => t.projectId === projectId.value),
);
const isEdit = computed(() => !!props.entry);

watch(projectId, () => {
  if (!tasks.value.some((t) => t.id === taskId.value)) taskId.value = null;
});

onMounted(() => noteEl.value?.focus());

function quick(min: number) {
  minutesText.value = String(min);
  error.value = "";
}

async function polish() {
  if (!note.value.trim()) return;
  busy.value = true;
  try {
    note.value = await api.polishText(note.value.trim());
  } catch (e) {
    toast(String(e), "err");
  } finally {
    busy.value = false;
  }
}

async function submit() {
  const minutes = parseDuration(minutesText.value);
  if (!projectId.value) {
    error.value = "选一个项目";
    return;
  }
  if (!minutes || minutes <= 0) {
    error.value = "填个时长，比如 45m 或 1h30m";
    return;
  }
  const ok = await saveEntryFlow({
    id: props.entry?.id,
    projectId: projectId.value,
    taskId: taskId.value,
    day: day.value,
    minutes,
    note: note.value.trim(),
  });
  if (ok) closeModal();
}

async function remove() {
  if (!props.entry) return;
  if (!confirmDelete.value) {
    confirmDelete.value = true;
    window.setTimeout(() => (confirmDelete.value = false), 3000);
    return;
  }
  await deleteEntryFlow(props.entry.id);
  closeModal();
}
</script>

<template>
  <Modal :title="isEdit ? '编辑记录' : '补记时间'" :width="470" @close="closeModal">
    <div class="field">
      <span class="lab">项目</span>
      <select v-model="projectId" class="input">
        <option v-for="p in projects" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
    </div>

    <div class="field">
      <span class="lab">任务</span>
      <select v-model="taskId" class="input">
        <option :value="null">不指定任务</option>
        <option v-for="t in tasks" :key="t.id" :value="t.id">{{ t.title }}</option>
      </select>
    </div>

    <div class="field">
      <span class="lab">日期</span>
      <div class="dates">
        <input v-model="day" type="date" class="input" />
        <button class="chip-btn" @click="day = state.date">今天</button>
        <button class="chip-btn" @click="day = shiftISO(state.date, -1)">昨天</button>
      </div>
    </div>

    <div class="field">
      <span class="lab">时长</span>
      <div class="dates">
        <input
          v-model="minutesText"
          class="input"
          style="max-width: 96px"
          placeholder="45m"
          @input="error = ''"
        />
        <button v-for="m in [15, 30, 45, 60, 90]" :key="m" class="chip-btn" @click="quick(m)">
          {{ m }}m
        </button>
      </div>
    </div>

    <label class="field">
      <span class="lab">说明</span>
      <div class="note-wrap">
        <textarea
          ref="noteEl"
          v-model="note"
          class="input"
          rows="3"
          placeholder="这件事具体做了什么，日报会直接用它"
        />
        <button class="ai" :disabled="busy || !note.trim()" title="用 AI 改写成日报口吻" @click.prevent="polish">
          <Icon name="sparkle" :size="12" />
          {{ busy ? "处理中" : "改写" }}
        </button>
      </div>
    </label>

    <p v-if="error" class="err">{{ error }}</p>

    <template #footer>
      <button v-if="isEdit" class="btn danger" @click="remove">
        {{ confirmDelete ? "再点一次删除" : "删除" }}
      </button>
      <div style="flex: 1"></div>
      <button class="btn" @click="closeModal">取消</button>
      <button class="btn primary" @click="submit">保存</button>
    </template>
  </Modal>
</template>

<style scoped>
.field {
  display: grid;
  grid-template-columns: 46px 1fr;
  align-items: center;
  gap: 10px;
  margin-bottom: 9px;
}

.lab {
  font-size: 12px;
  color: var(--text-3);
}

.field:has(textarea) {
  align-items: start;
}

.field:has(textarea) .lab {
  padding-top: 7px;
}

.dates {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.dates .input {
  flex: 1;
  min-width: 90px;
}

.chip-btn {
  height: 26px;
  padding: 0 8px;
  border-radius: 6px;
  border: 0.5px solid var(--line-strong);
  font-size: 11.5px;
  color: var(--text-2);
  background: var(--surface);
  white-space: nowrap;
}

.chip-btn:hover {
  background: var(--sunken);
  color: var(--text);
}

.note-wrap {
  position: relative;
}

.note-wrap .input {
  width: 100%;
  padding-bottom: 30px;
}

.ai {
  position: absolute;
  right: 6px;
  bottom: 6px;
  height: 21px;
  padding: 0 7px;
  border-radius: 5px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 11px;
  display: flex;
  align-items: center;
  gap: 3px;
}

.ai:disabled {
  opacity: 0.4;
}

.err {
  margin: 2px 0 0 56px;
  font-size: 11.5px;
  color: var(--danger);
}
</style>
