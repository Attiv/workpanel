<script setup lang="ts">
import { onMounted, ref } from "vue";
import * as api from "../api";
import Icon from "./Icon.vue";
import { refresh, saveSettingsFlow, state, toast } from "../store";
import type { Settings } from "../types";
import { locale, setLocale, t } from "../i18n";

const form = ref<Settings>({
  aiBase: "",
  aiModel: "",
  aiKey: "",
  workdayMinutes: 480,
  reportLang: "both",
});

const dbPath = ref("");
const version = ref("");
const testing = ref(false);
const testResult = ref("");
const testOk = ref(false);
const importing = ref(false);
const importMsg = ref("");
const hours = ref(8);

onMounted(async () => {
  const s = state.snap?.settings;
  if (s) {
    form.value = { ...s };
    hours.value = Math.round((s.workdayMinutes / 60) * 10) / 10;
  }
  try {
    const info = await api.appInfo();
    dbPath.value = info.dbPath;
    version.value = info.version;
  } catch {
    /* ignore */
  }
});

async function save() {
  form.value.workdayMinutes = Math.round(hours.value * 60);
  await saveSettingsFlow({ ...form.value });
}

async function test() {
  testing.value = true;
  testResult.value = "";
  testOk.value = false;
  try {
    form.value.workdayMinutes = Math.round(hours.value * 60);
    await api.saveSettings({ ...form.value });
    await refresh();
    const out = await api.testAi();
    testResult.value = t("set.testOk", { msg: out });
    testOk.value = true;
  } catch (e) {
    testResult.value = String(e);
  } finally {
    testing.value = false;
  }
}

async function importKanri() {
  importing.value = true;
  importMsg.value = "";
  try {
    const r = await api.importKanri();
    await refresh();
    importMsg.value = t("set.importResult", {
      boards: r.boards,
      cards: r.cards,
      tasks: r.createdTasks,
      projects: r.createdProjects,
      skipped: r.skipped,
    });
    toast(t("set.importDone"));
  } catch (e) {
    importMsg.value = String(e);
  } finally {
    importing.value = false;
  }
}
</script>

<template>
  <section class="view">
    <header class="head">
      <h1>{{ t("set.title") }}</h1>
      <div style="flex: 1"></div>
      <button class="btn primary" @click="save">{{ t("common.save") }}</button>
    </header>

    <div class="body">
      <section class="card">
        <div class="card-head"><h2>{{ t("set.ui") }}</h2></div>
        <div class="field">
          <span class="lab">{{ t("set.uiLang") }}</span>
          <div class="seg">
            <button :class="{ on: locale === 'zh' }" @click="setLocale('zh')">中文</button>
            <button :class="{ on: locale === 'en' }" @click="setLocale('en')">English</button>
          </div>
        </div>
      </section>

      <section class="card">
        <div class="card-head">
          <h2>AI</h2>
          <span class="muted">{{ t("set.aiHint") }}</span>
        </div>

        <div class="field">
          <span class="lab">{{ t("set.endpoint") }}</span>
          <input v-model="form.aiBase" class="input" placeholder="https://api.deepseek.com/v1" />
        </div>
        <div class="field">
          <span class="lab">{{ t("set.model") }}</span>
          <input v-model="form.aiModel" class="input" placeholder="deepseek-chat" />
        </div>
        <div class="field">
          <span class="lab">API Key</span>
          <input v-model="form.aiKey" class="input" type="password" placeholder="sk-..." />
        </div>

        <div class="row">
          <button class="btn" :disabled="testing" @click="test">
            {{ testing ? t("set.testing") : t("set.test") }}
          </button>
          <span v-if="testResult" class="test-out" :class="{ bad: !testOk }">
            {{ testResult }}
          </span>
        </div>
      </section>

      <section class="card">
        <div class="card-head"><h2>{{ t("set.rhythm") }}</h2></div>
        <div class="field">
          <span class="lab">{{ t("set.workday") }}</span>
          <div class="inline">
            <input v-model.number="hours" class="input" type="number" min="1" max="16" step="0.5" style="width: 76px" />
            <span class="muted">{{ t("set.workdayHint") }}</span>
          </div>
        </div>
        <div class="field">
          <span class="lab">{{ t("set.reportLang") }}</span>
          <div class="seg">
            <button :class="{ on: form.reportLang === 'zh' }" @click="form.reportLang = 'zh'">
              {{ t("report.langZh") }}
            </button>
            <button :class="{ on: form.reportLang === 'en' }" @click="form.reportLang = 'en'">
              {{ t("report.langEn") }}
            </button>
            <button :class="{ on: form.reportLang === 'both' }" @click="form.reportLang = 'both'">
              {{ t("report.langBoth") }}
            </button>
          </div>
        </div>
      </section>

      <section class="card">
        <div class="card-head">
          <h2>{{ t("set.data") }}</h2>
        </div>

        <div class="field">
          <span class="lab">{{ t("set.db") }}</span>
          <code class="path">{{ dbPath }}</code>
        </div>

        <div class="row">
          <button class="btn" :disabled="importing" @click="importKanri">
            {{ importing ? t("set.importing") : t("set.import") }}
          </button>
          <span class="muted">{{ t("set.importHint") }}</span>
        </div>
        <p v-if="importMsg" class="import-msg">{{ importMsg }}</p>
      </section>

      <section class="card">
        <div class="card-head"><h2>{{ t("set.about") }}</h2></div>
        <p class="about">WorkPanel {{ version }} · {{ t("set.aboutText") }}</p>
      </section>
    </div>
  </section>
</template>

<style scoped>
.view {
  display: flex;
  flex-direction: column;
  align-items: center;
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
  width: 100%;
  max-width: 760px;
}

.head h1 {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-bottom: 10px;
  width: 100%;
  max-width: 760px;
}

.card {
  background: var(--surface);
  border: 0.5px solid var(--line);
  border-radius: var(--r);
  padding: 14px 16px 16px;
}

.card-head {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 12px;
}

.card-head h2 {
  margin: 0;
  font-size: 13px;
  font-weight: 500;
}

.card-head .muted,
.row .muted {
  font-size: 11.5px;
  color: var(--text-3);
}

.field {
  display: grid;
  grid-template-columns: 68px 1fr;
  align-items: center;
  gap: 10px;
  margin-bottom: 9px;
}

.lab {
  font-size: 12px;
  color: var(--text-3);
}

.inline {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11.5px;
}

.row {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-top: 10px;
  flex-wrap: wrap;
}

.test-out {
  font-size: 11.5px;
  color: var(--ok);
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.test-out.bad {
  color: var(--danger);
  white-space: normal;
}

.seg {
  display: inline-flex;
  gap: 2px;
  background: var(--sunken);
  padding: 2px;
  border-radius: 7px;
  justify-self: start;
}

.seg button {
  height: 24px;
  padding: 0 10px;
  border-radius: 5px;
  font-size: 12px;
  color: var(--text-3);
}

.seg button.on {
  background: var(--surface);
  color: var(--text);
  font-weight: 500;
  border: 0.5px solid var(--line);
}

.path {
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  font-size: 11.5px;
  color: var(--text-2);
  background: var(--sunken);
  padding: 4px 7px;
  border-radius: 5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  user-select: text;
}

.import-msg {
  margin: 8px 0 0;
  font-size: 11.5px;
  color: var(--text-2);
  line-height: 1.5;
}

.about {
  margin: 0;
  font-size: 12px;
  color: var(--text-2);
  line-height: 1.6;
}
</style>
