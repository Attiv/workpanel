<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import Icon from "./Icon.vue";
import Modal from "./Modal.vue";
import { closeModal, deleteTaskFlow, saveTaskFlow, state } from "../store";
import { fmtDur, parseDuration, shiftISO } from "../utils";
import type { Status, Task } from "../types";
import { t } from "../i18n";

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
    error.value = t("dlg.errTitle");
    return;
  }
  const minutes = estimate.value.trim() ? parseDuration(estimate.value) : null;
  if (estimate.value.trim() && !minutes) {
    error.value = t("dlg.errDuration");
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
  <Modal :title="isEdit ? t('dlg.taskEdit') : t('dlg.taskNew')" :width="470" @close="closeModal">
    <label class="field">
      <span class="lab">{{ t("dlg.task") }}</span>
      <input
        ref="titleEl"
        v-model="title"
        class="input"
        :placeholder="t('dlg.taskPlaceholder')"
        @keydown.enter.prevent="submit"
        @input="error = ''"
      />
    </label>

    <div class="field">
      <span class="lab">{{ t("dlg.project") }}</span>
      <select v-model="projectId" class="input">
        <option :value="null">{{ t("common.unassigned") }}</option>
        <option v-for="p in projects" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
    </div>

    <div class="field">
      <span class="lab">{{ t("dlg.status") }}</span>
      <select v-model="status" class="input">
        <option value="todo">{{ t("dlg.todo") }}</option>
        <option value="doing">{{ t("dlg.doing") }}</option>
        <option value="done">{{ t("dlg.done") }}</option>
      </select>
    </div>

    <div class="field">
      <span class="lab">{{ t("dlg.planned") }}</span>
      <div class="dates">
        <input v-model="planned" type="date" class="input" />
        <button class="chip-btn" @click="setPlan(0)">{{ t("common.today") }}</button>
        <button class="chip-btn" @click="setPlan(1)">{{ t("common.tomorrow") }}</button>
        <button v-if="planned" class="chip-btn" @click="setPlan(null)">
          {{ t("dlg.clear") }}
        </button>
      </div>
    </div>

    <div class="field">
      <span class="lab">{{ t("dlg.due") }}</span>
      <input v-model="due" type="date" class="input" />
    </div>

    <div class="field">
      <span class="lab">{{ t("dlg.estimate") }}</span>
      <input v-model="estimate" class="input" placeholder="1h30m" />
    </div>

    <label class="field">
      <span class="lab">{{ t("dlg.note") }}</span>
      <textarea v-model="note" class="input" rows="3" :placeholder="t('dlg.notePlaceholder')" />
    </label>

    <p v-if="error" class="err">{{ error }}</p>

    <template #footer>
      <button v-if="isEdit" class="btn danger" @click="remove">
        {{ confirmDelete ? t("common.deleteAgain") : t("common.delete") }}
      </button>
      <div style="flex: 1"></div>
      <button class="btn" @click="closeModal">{{ t("common.cancel") }}</button>
      <button class="btn primary" @click="submit">{{ t("common.save") }}</button>
    </template>
  </Modal>
</template>

<style scoped>
.field {
  display: grid;
  grid-template-columns: 58px 1fr;
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
  margin: 2px 0 0 68px;
  font-size: 11.5px;
  color: var(--danger);
}
</style>
