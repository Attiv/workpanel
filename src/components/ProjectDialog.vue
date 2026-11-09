<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import Modal from "./Modal.vue";
import { closeModal, deleteProjectFlow, saveProjectFlow, state } from "../store";
import type { Project } from "../types";
import { t } from "../i18n";

const props = defineProps<{ project?: Project }>();

const PALETTE = [
  "#6C5CE7",
  "#12967A",
  "#D2603A",
  "#2F7BD6",
  "#B07714",
  "#C94F7C",
  "#5E8C1F",
  "#6B7A90",
];

const name = ref(props.project?.name ?? "");
const color = ref(props.project?.color ?? PALETTE[0]);
const aliases = ref((props.project?.aliases ?? []).join("、"));
const error = ref("");
const confirmDelete = ref(false);
const nameEl = ref<HTMLInputElement | null>(null);

const isEdit = computed(() => !!props.project);

onMounted(() => nameEl.value?.focus());

async function submit() {
  if (!name.value.trim()) {
    error.value = t("dlg.errName");
    return;
  }
  const list = aliases.value
    .split(/[、,，\s]+/)
    .map((s) => s.trim())
    .filter(Boolean);
  const id = await saveProjectFlow({
    id: props.project?.id,
    name: name.value.trim(),
    color: color.value,
    aliases: list,
  });
  if (id) closeModal();
}

async function remove() {
  if (!props.project) return;
  if (!confirmDelete.value) {
    confirmDelete.value = true;
    window.setTimeout(() => (confirmDelete.value = false), 3000);
    return;
  }
  await deleteProjectFlow(props.project.id);
  closeModal();
}
</script>

<template>
  <Modal :title="isEdit ? t('dlg.projEdit') : t('dlg.projNew')" :width="420" @close="closeModal">
    <div class="field">
      <span class="lab">{{ t("dlg.name") }}</span>
      <input
        ref="nameEl"
        v-model="name"
        class="input"
        :placeholder="t('dlg.namePlaceholder')"
        @keydown.enter.prevent="submit"
        @input="error = ''"
      />
    </div>

    <div class="field">
      <span class="lab">{{ t("dlg.color") }}</span>
      <div class="swatches">
        <button
          v-for="c in PALETTE"
          :key="c"
          class="sw"
          :class="{ on: color === c }"
          :style="{ background: c }"
          @click="color = c"
        ></button>
      </div>
    </div>

    <div class="field">
      <span class="lab">{{ t("dlg.aliases") }}</span>
      <input v-model="aliases" class="input" :placeholder="t('dlg.aliasesPlaceholder')" />
    </div>

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
  margin-bottom: 11px;
}

.lab {
  font-size: 12px;
  color: var(--text-3);
}

.swatches {
  display: flex;
  gap: 7px;
}

.sw {
  width: 19px;
  height: 19px;
  border-radius: 50%;
  border: 2px solid transparent;
  box-shadow: 0 0 0 0.5px var(--line-strong);
}

.sw.on {
  border-color: var(--surface);
  box-shadow: 0 0 0 1.5px var(--text);
}

.err {
  margin: 2px 0 0 68px;
  font-size: 11.5px;
  color: var(--danger);
}
</style>
