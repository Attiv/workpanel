<script setup lang="ts">
import { onMounted, ref } from "vue";
import * as api from "../api";
import Icon from "./Icon.vue";
import { refresh, saveSettingsFlow, state, toast } from "../store";
import type { Settings } from "../types";

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
  try {
    form.value.workdayMinutes = Math.round(hours.value * 60);
    await api.saveSettings({ ...form.value });
    await refresh();
    const out = await api.testAi();
    testResult.value = `通了：${out}`;
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
    importMsg.value = `从 ${r.boards} 个看板读到 ${r.cards} 张卡片，新增 ${r.createdTasks} 个任务、${r.createdProjects} 个项目，跳过 ${r.skipped} 个重复。`;
    toast("导入完成");
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
      <h1>设置</h1>
      <div style="flex: 1"></div>
      <button class="btn primary" @click="save">保存</button>
    </header>

    <div class="body">
      <section class="card">
        <div class="card-head">
          <h2>AI</h2>
          <span class="muted">可选。只在你点「改写」或「AI 翻译」时调用。</span>
        </div>

        <div class="field">
          <span class="lab">接口地址</span>
          <input v-model="form.aiBase" class="input" placeholder="https://api.deepseek.com/v1" />
        </div>
        <div class="field">
          <span class="lab">模型</span>
          <input v-model="form.aiModel" class="input" placeholder="deepseek-chat" />
        </div>
        <div class="field">
          <span class="lab">API Key</span>
          <input v-model="form.aiKey" class="input" type="password" placeholder="sk-..." />
        </div>

        <div class="row">
          <button class="btn" :disabled="testing" @click="test">
            {{ testing ? "测试中" : "测试连接" }}
          </button>
          <span v-if="testResult" class="test-out" :class="{ bad: !testResult.startsWith('通了') }">
            {{ testResult }}
          </span>
        </div>
      </section>

      <section class="card">
        <div class="card-head"><h2>工作节奏</h2></div>
        <div class="field">
          <span class="lab">每天工时</span>
          <div class="inline">
            <input v-model.number="hours" class="input" type="number" min="1" max="16" step="0.5" style="width: 76px" />
            <span class="muted">小时，看板顶部按它算进度</span>
          </div>
        </div>
        <div class="field">
          <span class="lab">日报默认</span>
          <div class="seg">
            <button :class="{ on: form.reportLang === 'zh' }" @click="form.reportLang = 'zh'">中文</button>
            <button :class="{ on: form.reportLang === 'en' }" @click="form.reportLang = 'en'">English</button>
            <button :class="{ on: form.reportLang === 'both' }" @click="form.reportLang = 'both'">双语</button>
          </div>
        </div>
      </section>

      <section class="card">
        <div class="card-head">
          <h2>数据</h2>
        </div>

        <div class="field">
          <span class="lab">数据库</span>
          <code class="path">{{ dbPath }}</code>
        </div>

        <div class="row">
          <button class="btn" :disabled="importing" @click="importKanri">
            {{ importing ? "导入中" : "从 Kanri 导入任务" }}
          </button>
          <span class="muted">只读 Kanri 的数据文件，不会改动它；重复任务会跳过。</span>
        </div>
        <p v-if="importMsg" class="import-msg">{{ importMsg }}</p>
      </section>

      <section class="card">
        <div class="card-head"><h2>关于</h2></div>
        <p class="about">
          WorkPanel {{ version }} · 本地优先，数据只存在这台机器上，不上传任何服务器。
        </p>
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
