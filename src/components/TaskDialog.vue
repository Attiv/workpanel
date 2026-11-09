<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import Icon from "./Icon.vue";
import Modal from "./Modal.vue";
import { closeModal, deleteTaskFlow, saveTaskFlow, state } from "../store";
import { fmtDur, parseDuration, shiftISO } from "../utils";
import type { Status, Task } from "../types";

const props = defineProps<{ task?: Task; column?: string }>();

const title = ref(props.task?.title ?? "");
const projectId = ref<number | null>(
  props.task?.projectId ?? state.snap?.projects[0]?.id ?? null,
);
const status = ref<Status>(props.task?.status ?? "todo");
const planned = ref<string>(props.task?.plannedDate ?? (props.column === "today" ? state.date : ""));
const due = ref(props.task?.dueDate ?? "");
const estimate = ref(props.task?.estimateMin ? fmtDur(props.task.estimateMin) : "");
const note = ref(props.task?.note ?? "");
const error = ref("");
const confirmDelete = ref(false);
const titleEl = ref<HTMLInputElement | null>(null);

const projects = computed(() => state.snap?.projects ?? []);
const isEdit = computed(() => !!props.task);

onMounted(async () => {
  await nextTick();
  titleEl.value?.focus();
});

function setPlan(offset: number | null) {
  planned.value = offset === null ? "" : shiftISO(state.date, offset);
}

async function submit() {
  if (!title.value.trim()) {
    error.value = "先写一句任务名";
    return;
  }
  const minutes = estimate.value.trim() ? parseDuration(estimate.value) : null;
  if (estimate.value.trim() && !minutes) {
    error.value = "时长格式看不懂，试试 1h30m 或 45m";
    return;
  }
  const ok = await saveTaskFlow({
    id: props.task?.id,
    projectId: projectId.value,
    title: title.value.trim(),
    note: note.value,
    status: status.value,
    plannedDate: planned.value || null,
    dueDate: due.value || null,
    estimateMin: minutes,
  });
  if (ok) closeModal();
}

async function remove() {
  if (!props.task) return;
  if (!confirmDelete.value) {
    confirmDelete.value = true;
    window.setTimeout(() => (confirmDelete.value = false), 3000);
    return;
  }
  await deleteTaskFlow(props.task.id);
  closeModal();
}
</script>

<template>
  <Modal :title="isEdit ? '编辑任务' : '新任务'" :width="470" @close="closeModal">
    <label class="field">
      <span class="lab">任务</span>
      <input
        ref="titleEl"
        v-model="title"
        class="input"
        placeholder="要做什么"
        @keydown.enter.prevent="submit"
        @input="error = ''"
      />
    </label>

    <div class="field">
      <span class="lab">项目</span>
      <select v-model="projectId" class="input">
        <option :value="null">未归类</option>
        <option v-for="p in projects" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
    </div>

    <div class="field">
      <span class="lab">状态</span>
      <select v-model="status" class="input">
        <option value="todo">待办</option>
        <option value="doing">进行中</option>
        <option value="done">已完成</option>
      </select>
    </div>

    <div class="field">
      <span class="lab">排到</span>
      <div class="dates">
        <input v-model="planned" type="date" class="input" />
        <button class="chip-btn" @click="setPlan(0)">今天</button>
        <button class="chip-btn" @click="setPlan(1)">明天</button>
        <button v-if="planned" class="chip-btn" @click="setPlan(null)">清除</button>
      </div>
    </div>

    <div class="field">
      <span class="lab">截止</span>
      <input v-model="due" type="date" class="input" />
    </div>

    <div class="field">
      <span class="lab">预估</span>
      <input v-model="estimate" class="input" placeholder="1h30m" />
    </div>

    <label class="field">
      <span class="lab">备注</span>
      <textarea v-model="note" class="input" rows="3" placeholder="补充说明，会出现在日报里" />
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
  align-self: center;
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
}

.dates .input {
  flex: 1;
  min-width: 0;
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

.err {
  margin: 2px 0 0 56px;
  font-size: 11.5px;
  color: var(--danger);
}
</style>
