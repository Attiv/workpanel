<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import * as api from "../api";
import Icon from "./Icon.vue";
import ImageAttach from "./ImageAttach.vue";
import Modal from "./Modal.vue";
import {
  closeModal,
  deleteEntryFlow,
  saveEntryFlow,
  state,
  syncAttachments,
  toast,
} from "../store";
import { imageFilesFromClipboard, parseDuration, shiftISO } from "../utils";
import type { AttachmentDraft, Entry, Task } from "../types";
import { t } from "../i18n";

const props = defineProps<{ entry?: Entry; day?: string; task?: Task }>();

const projectId = ref<number | null>(
  props.entry?.projectId ?? props.task?.projectId ?? state.snap?.projects[0]?.id ?? null,
);
const taskId = ref<number | null>(props.entry?.taskId ?? props.task?.id ?? null);
const day = ref(props.entry?.day ?? props.day ?? state.date);
const minutesText = ref(props.entry ? String(props.entry.minutes) : "");
const note = ref(props.entry?.note ?? "");
const images = ref<AttachmentDraft[]>([]);
const attachEl = ref<InstanceType<typeof ImageAttach> | null>(null);
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

onMounted(async () => {
  window.addEventListener("paste", onPaste);
  noteEl.value?.focus();
  if (props.entry) {
    try {
      const list = await api.listAttachments("entry", props.entry.id);
      images.value = list.map((a) => ({
        id: a.id,
        dataUrl: a.dataUrl,
        mime: a.mime,
        width: a.width,
        height: a.height,
        bytes: a.bytes,
      }));
    } catch {
      // 图片加载失败不该挡住改文字
    }
  }
});

onBeforeUnmount(() => window.removeEventListener("paste", onPaste));

function onPaste(e: ClipboardEvent) {
  const files = imageFilesFromClipboard(e);
  if (!files.length) return;
  e.preventDefault();
  void attachEl.value?.addFiles(files);
}

function onDrop(e: DragEvent) {
  if (!e.dataTransfer?.files.length) return;
  e.preventDefault();
  void attachEl.value?.addFiles(Array.from(e.dataTransfer.files));
}

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
    error.value = t("dlg.errProject");
    return;
  }
  if (!minutes || minutes <= 0) {
    error.value = t("dlg.errMinutes");
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
  if (!ok) return;
  if (images.value.some((d) => d.removed || !d.id)) {
    await syncAttachments("entry", ok, images.value);
  }
  closeModal();
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
  <Modal :title="isEdit ? t('dlg.entryEdit') : t('dlg.entryNew')" :width="470" @close="closeModal">
    <div class="dlg-body" @dragover.prevent @drop="onDrop">
      <div class="field">
        <span class="lab">{{ t("dlg.project") }}</span>
        <select v-model="projectId" class="input">
          <option v-for="p in projects" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
      </div>

      <div class="field">
        <span class="lab">{{ t("dlg.task") }}</span>
        <select v-model="taskId" class="input">
          <option :value="null">{{ t("dlg.noTask") }}</option>
          <option v-for="task in tasks" :key="task.id" :value="task.id">
            {{ task.title }}
          </option>
        </select>
      </div>

      <div class="field">
        <span class="lab">{{ t("dlg.date") }}</span>
        <div class="dates">
          <input v-model="day" type="date" class="input" />
          <button class="chip-btn" @click="day = state.date">{{ t("common.today") }}</button>
          <button class="chip-btn" @click="day = shiftISO(state.date, -1)">
            {{ t("common.yesterday") }}
          </button>
        </div>
      </div>

      <div class="field">
        <span class="lab">{{ t("dlg.duration") }}</span>
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
        <span class="lab">{{ t("dlg.desc") }}</span>
        <div class="note-wrap">
          <textarea
            ref="noteEl"
            v-model="note"
            class="input"
            rows="3"
            :placeholder="t('dlg.descPlaceholder')"
          />
          <button
            class="ai"
            :disabled="busy || !note.trim()"
            :title="t('dlg.polishTitle')"
            @click.prevent="polish"
          >
            <Icon name="sparkle" :size="12" />
            {{ busy ? t("dlg.polishing") : t("dlg.polish") }}
          </button>
        </div>
      </label>

      <div class="attach-row">
        <ImageAttach ref="attachEl" v-model="images" />
      </div>

      <p v-if="error" class="err">{{ error }}</p>
    </div>

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
  margin: 2px 0 0 68px;
  font-size: 11.5px;
  color: var(--danger);
}

.attach-row {
  margin: -2px 0 9px 68px;
}
</style>
