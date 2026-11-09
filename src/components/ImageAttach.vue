<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import Icon from "./Icon.vue";
import type { AttachmentDraft } from "../types";
import { prepareImage } from "../utils";
import { t } from "../i18n";
import { toast } from "../store";

const props = defineProps<{ modelValue: AttachmentDraft[]; limit?: number }>();
const emit = defineEmits<{ "update:modelValue": [AttachmentDraft[]] }>();

const cap = computed(() => props.limit ?? 12);
const busy = ref(false);
const viewer = ref<string | null>(null);
const fileEl = ref<HTMLInputElement | null>(null);

const active = computed(() =>
  props.modelValue.map((item, index) => ({ item, index })).filter((x) => !x.item.removed),
);

async function addFiles(files: File[]) {
  const images = Array.from(files).filter((f) => f.type.startsWith("image/"));
  if (!images.length) return;

  const next = [...props.modelValue];
  let count = next.filter((i) => !i.removed).length;
  if (count >= cap.value) {
    toast(t("dlg.attachTooMany", { n: cap.value }), "err");
    return;
  }

  busy.value = true;
  let failed = 0;
  try {
    for (const file of images) {
      if (count >= cap.value) {
        toast(t("dlg.attachTooMany", { n: cap.value }), "err");
        break;
      }
      try {
        next.push(await prepareImage(file));
        count += 1;
        // 一张一张往上贴，不用等整批压缩完
        emit("update:modelValue", [...next]);
      } catch {
        failed += 1;
      }
    }
  } finally {
    busy.value = false;
  }
  if (failed) toast(t("dlg.attachFailed"), "err");
}

function removeAt(index: number) {
  const next = props.modelValue.map((it, i) => (i === index ? { ...it, removed: true } : it));
  emit("update:modelValue", next);
}

function pick() {
  fileEl.value?.click();
}

function onPick(e: Event) {
  const input = e.target as HTMLInputElement;
  if (input.files?.length) void addFiles(Array.from(input.files));
  input.value = "";
}

function onDrop(e: DragEvent) {
  if (e.dataTransfer?.files?.length) void addFiles(Array.from(e.dataTransfer.files));
}

// 灯箱开着时 Esc 只关灯箱，不要再把底下的对话框一起关掉
function onKey(e: KeyboardEvent) {
  if (viewer.value && e.key === "Escape") {
    e.preventDefault();
    e.stopImmediatePropagation();
    viewer.value = null;
  }
}

onMounted(() => window.addEventListener("keydown", onKey, true));
onBeforeUnmount(() => window.removeEventListener("keydown", onKey, true));

defineExpose({ addFiles });
</script>

<template>
  <div class="attach">
    <div v-if="active.length || busy" class="strip" @dragover.prevent @drop.prevent="onDrop">
      <div
        v-for="entry in active"
        :key="entry.index"
        class="thumb"
        :title="t('dlg.attachOpen')"
        @click="viewer = entry.item.dataUrl"
      >
        <img :src="entry.item.dataUrl" alt="" />
        <button class="x" :title="t('dlg.attachRemove')" @click.stop="removeAt(entry.index)">
          <Icon name="close" :size="8" />
        </button>
      </div>
      <div v-if="busy" class="thumb working"><span class="spin" /></div>
      <button
        v-if="active.length < cap"
        class="thumb add"
        :title="t('dlg.attachAdd')"
        @click="pick"
        @dragover.prevent
        @drop.prevent="onDrop"
      >
        <Icon name="plus" :size="14" />
      </button>
    </div>

    <button v-else class="hint" type="button" @click="pick" @dragover.prevent @drop.prevent="onDrop">
      <Icon name="image" :size="13" />
      <span>{{ t("dlg.attachHint") }}</span>
      <em>{{ t("dlg.attachCap", { n: cap }) }}</em>
    </button>

    <input ref="fileEl" type="file" accept="image/*" multiple hidden @change="onPick" />

    <Teleport to="body">
      <div v-if="viewer" class="viewer" @click="viewer = null">
        <img :src="viewer" alt="" />
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.attach {
  margin-top: 6px;
}

.strip {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.thumb {
  position: relative;
  width: 54px;
  height: 54px;
  padding: 0;
  border: 0.5px solid var(--line);
  border-radius: 7px;
  background: var(--sunken);
  cursor: pointer;
  overflow: visible;
}

.thumb img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 6px;
}

.thumb .x {
  position: absolute;
  top: -5px;
  right: -5px;
  width: 15px;
  height: 15px;
  padding: 0;
  border-radius: 50%;
  background: var(--text);
  color: var(--surface);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.12s ease;
}

.thumb:hover .x {
  opacity: 0.92;
}

.thumb.add {
  display: flex;
  align-items: center;
  justify-content: center;
  border-style: dashed;
  border-color: var(--line-strong);
  color: var(--text-3);
}

.thumb.add:hover {
  color: var(--text);
  background: var(--sunken-2);
}

.thumb.working {
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: default;
}

.spin {
  width: 13px;
  height: 13px;
  border-radius: 50%;
  border: 1.5px solid var(--line-strong);
  border-top-color: var(--accent);
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.hint {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 24px;
  padding: 0 8px;
  border: 0.5px dashed var(--line-strong);
  border-radius: 6px;
  background: transparent;
  color: var(--text-3);
  font-size: 11.5px;
}

.hint em {
  font-style: normal;
  opacity: 0.7;
}

.hint:hover {
  color: var(--text-2);
  background: var(--sunken);
}

.viewer {
  position: fixed;
  inset: 0;
  z-index: 300;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
  background: rgba(8, 11, 16, 0.74);
  cursor: zoom-out;
  animation: viewer-in 0.12s ease-out;
}

.viewer img {
  max-width: 100%;
  max-height: 100%;
  border-radius: 8px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

@keyframes viewer-in {
  from {
    opacity: 0;
  }
}
</style>
