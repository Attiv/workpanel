<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import * as api from "../api";
import Icon from "./Icon.vue";
import { state, toast } from "../store";
import { escapeHtml, fmtDayShort, fmtDur, parseISO, weekdayName } from "../utils";
import { locale, t } from "../i18n";
import type { Report, ReportLine } from "../types";

type Lang = "zh" | "en" | "both";
type Range = "day" | "week" | "custom";

const range = ref<Range>("day");
const lang = ref<Lang>((state.snap?.settings.reportLang as Lang) || "both");
const d0 = ref(state.date);
const d1 = ref(state.date);
const report = ref<Report | null>(null);
const loading = ref(false);
const summarizing = ref(false);

function weekStart(iso: string): string {
  const d = parseISO(iso);
  const offset = (d.getDay() + 6) % 7;
  d.setDate(d.getDate() - offset);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function applyRange() {
  if (range.value === "day") {
    d0.value = state.date;
    d1.value = state.date;
  } else if (range.value === "week") {
    d0.value = weekStart(state.date);
    d1.value = state.date;
  }
}

async function load() {
  loading.value = true;
  try {
    report.value = await api.buildReport(d0.value, d1.value, lang.value);
  } catch (e) {
    toast(String(e), "err");
  } finally {
    loading.value = false;
  }
}

function setRange(r: Range) {
  range.value = r;
  applyRange();
  load();
}

watch([d0, d1], () => {
  if (range.value === "custom") load();
});

watch(lang, () => load());

watch(
  () => state.date,
  () => {
    if (range.value !== "custom") {
      applyRange();
      load();
    }
  },
);

onMounted(() => {
  applyRange();
  load();
});

/** 报告正文的语言跟「语言」选择走，界面语言只影响按钮和标签 */
const docLang = computed<"zh" | "en">(() => (lang.value === "en" ? "en" : "zh"));

const docTitle = computed(() => {
  if (lang.value === "zh") return t("report.docZh");
  if (lang.value === "en") return t("report.docEn");
  return locale.value === "en" ? t("report.docEn") : t("report.docZh");
});

const heading = computed(() => {
  if (d0.value === d1.value) return `${d0.value} ${weekdayName(parseISO(d0.value))}`;
  const sep = docLang.value === "en" ? " – " : " 至 ";
  return `${fmtDayShort(d0.value)}${sep}${fmtDayShort(d1.value)}`;
});

const lineCount = computed(() =>
  (report.value?.groups || []).reduce((n, g) => n + g.lines.length, 0),
);

const aiReady = computed(() => !!state.snap?.settings.aiKey);

async function summarizeEn(force = false) {
  const r = report.value;
  if (!r) return;
  // force 用于「重写英文总结」：把整份日报的英文都过一遍，覆盖旧结果
  const texts = force
    ? Array.from(new Set(r.groups.flatMap((g) => g.lines.map((l) => l.text))))
    : r.missingEn;
  if (!texts.length) return;
  summarizing.value = true;
  try {
    await api.translateMissing(texts, force);
    await load();
    toast(t(force ? "report.repolished" : "report.translated"));
  } catch (e) {
    toast(String(e), "err");
  } finally {
    summarizing.value = false;
  }
}

async function copyLang(which: "zh" | "en") {
  if (!report.value) return;
  const text = which === "zh" ? report.value.markdownZh : report.value.markdownEn;
  try {
    await navigator.clipboard.writeText(text);
    const missing = which === "en" ? report.value.missingEn.length : 0;
    if (missing) toast(t("report.copyEnMissing", { n: missing }));
    else toast(t(which === "zh" ? "report.copiedZh" : "report.copiedEn"));
  } catch {
    toast(t("report.copyFail"), "err");
  }
}

/** 点某一条的中文 / 英文行，只复制那一条的文字 */
async function copyLine(line: ReportLine, which: "zh" | "en") {
  if (which === "en" && !line.en) {
    toast(t("report.lineNoEn"));
    return;
  }
  // 刚拖选过文字就不要抢剪贴板
  if (window.getSelection()?.toString().trim()) return;
  const text = (which === "zh" ? line.text : line.en) ?? "";
  try {
    await navigator.clipboard.writeText(text);
    const brief = text.length > 24 ? `${text.slice(0, 24)}…` : text;
    toast(t("report.lineCopied", { text: brief }));
  } catch {
    toast(t("report.copyFail"), "err");
  }
}

function buildHtml(): string {
  const r = report.value;
  if (!r) return "";
  const sections = r.groups
    .map((g) => {
      const lines = g.lines
        .map((l) => {
          if (lang.value === "en") {
            const text = escapeHtml(l.en || l.text);
            return `<div class="line"><div><div class="zh">${text}</div></div><div class="m">${fmtDur(l.minutes)}</div></div>`;
          }
          const en = lang.value === "both" && l.en ? `<div class="en">${escapeHtml(l.en)}</div>` : "";
          return `<div class="line"><div><div class="zh">${escapeHtml(l.text)}</div>${en}</div><div class="m">${fmtDur(l.minutes)}</div></div>`;
        })
        .join("");
      return `<section><h2><i style="background:${g.color}"></i>${escapeHtml(g.name)}<span>${fmtDur(g.minutes)}</span></h2>${lines}</section>`;
    })
    .join("");

  const en = docLang.value === "en";
  const title = en ? "Daily report" : "工作日报";
  const sub = en
    ? `${heading.value} · ${lineCount.value} lines · ${fmtDur(r.totalMinutes)} total`
    : `${heading.value} · ${lineCount.value} 条 · 合计 ${fmtDur(r.totalMinutes)}`;
  const foot = en ? `Total ${fmtDur(r.totalMinutes)}` : `合计 ${fmtDur(r.totalMinutes)}`;

  return `<!doctype html><html lang="${en ? "en" : "zh-CN"}"><head><meta charset="utf-8"><title>${title} ${d0.value}</title>
<style>
body{font-family:-apple-system,"PingFang SC","Microsoft YaHei",sans-serif;max-width:720px;margin:48px auto;padding:0 24px;color:#131922;line-height:1.6}
h1{font-size:20px;font-weight:600;margin:0 0 4px}
.sub{color:#8b94a2;font-size:13px;margin-bottom:28px}
section{margin-bottom:22px}
h2{font-size:13px;font-weight:600;margin:0 0 8px;padding-bottom:6px;border-bottom:1px solid #e1e6ed;display:flex;align-items:center;gap:6px}
h2 i{width:7px;height:7px;border-radius:50%}
h2 span{margin-left:auto;font-weight:500;color:#5b6573}
.line{display:flex;gap:12px;padding:3px 0;font-size:13.5px}
.line>div:first-child{flex:1}
.en{color:#8b94a2;font-size:12.5px;margin-top:1px}
.m{color:#5b6573;font-size:12px;white-space:nowrap;padding-top:2px}
footer{margin-top:26px;padding-top:10px;border-top:1px solid #e1e6ed;font-size:13px;color:#5b6573}
</style></head><body>
<h1>${title}</h1>
<div class="sub">${sub}</div>
${sections}
<footer>${foot}</footer>
</body></html>`;
}

async function exportMd() {
  if (!report.value) return;
  const path = await save({
    defaultPath: `${t("report.filePrefix")}-${d0.value}.md`,
    filters: [{ name: "Markdown", extensions: ["md"] }],
  });
  if (!path) return;
  await api.saveTextFile(path, report.value.markdown);
  toast(t("report.exportedMd"));
}

async function exportHtml() {
  const path = await save({
    defaultPath: `${t("report.filePrefix")}-${d0.value}.html`,
    filters: [{ name: "HTML", extensions: ["html"] }],
  });
  if (!path) return;
  await api.saveTextFile(path, buildHtml());
  toast(t("report.exportedHtml"));
}

async function print() {
  try {
    const path = await api.writeTempHtml(buildHtml());
    await openPath(path);
  } catch (e) {
    toast(String(e), "err");
  }
}
</script>

<template>
  <section class="view">
    <header class="head">
      <h1>{{ t("report.title") }}</h1>
      <div style="flex: 1"></div>
      <div class="seg">
        <button :class="{ on: range === 'day' }" @click="setRange('day')">
          {{ t("report.day") }}
        </button>
        <button :class="{ on: range === 'week' }" @click="setRange('week')">
          {{ t("report.week") }}
        </button>
        <button :class="{ on: range === 'custom' }" @click="setRange('custom')">
          {{ t("report.custom") }}
        </button>
      </div>
      <div v-if="range === 'custom'" class="dates">
        <input v-model="d0" type="date" class="input" />
        <span class="muted">—</span>
        <input v-model="d1" type="date" class="input" />
      </div>
    </header>

    <div class="toolbar">
      <div class="seg">
        <button :class="{ on: lang === 'zh' }" @click="lang = 'zh'">
          {{ t("report.langZh") }}
        </button>
        <button :class="{ on: lang === 'en' }" @click="lang = 'en'">
          {{ t("report.langEn") }}
        </button>
        <button :class="{ on: lang === 'both' }" @click="lang = 'both'">
          {{ t("report.langBoth") }}
        </button>
      </div>

      <span class="meta num">
        {{ t("report.meta", { n: lineCount, total: fmtDur(report?.totalMinutes || 0) }) }}
      </span>

      <div style="flex: 1"></div>

      <button
        v-if="report?.missingEn.length"
        class="btn"
        :class="{ primary: aiReady }"
        :disabled="summarizing || !aiReady"
        :title="aiReady ? t('report.aiTitle') : t('report.aiNeedKey')"
        @click="summarizeEn()"
      >
        <Icon name="sparkle" :size="12" />
        {{
          summarizing
            ? t("report.translating")
            : t("report.aiTranslate", { n: report.missingEn.length })
        }}
      </button>
      <button
        v-else-if="aiReady && lineCount && lang !== 'zh'"
        class="btn"
        :disabled="summarizing"
        :title="t('report.repolishTitle')"
        @click="summarizeEn(true)"
      >
        <Icon name="sparkle" :size="12" />
        {{ summarizing ? t("report.translating") : t("report.repolish") }}
      </button>
      <button class="btn" :title="t('report.copyZhTitle')" @click="copyLang('zh')">
        {{ t("report.copyZh") }}
      </button>
      <button class="btn" :title="t('report.copyEnTitle')" @click="copyLang('en')">
        {{ t("report.copyEn") }}
      </button>
      <button class="btn" @click="exportMd">{{ t("report.exportMd") }}</button>
      <button class="btn" @click="exportHtml">{{ t("report.exportHtml") }}</button>
      <button class="btn" @click="print">
        <Icon name="printer" :size="12" />{{ t("report.print") }}
      </button>
    </div>

    <div class="scroll">
      <article class="paper">
        <div class="p-head">
          <h2>{{ docTitle }}</h2>
          <div class="sub">{{ heading }}</div>
        </div>

        <p v-if="!loading && !lineCount" class="empty">{{ t("report.empty") }}</p>

        <section v-for="g in report?.groups || []" :key="g.name" class="group">
          <div class="g-head">
            <i class="dot" :style="{ background: g.color }"></i>
            <h3>{{ g.name }}</h3>
            <span class="g-min num">{{ fmtDur(g.minutes) }}</span>
          </div>
          <div v-for="(l, i) in g.lines" :key="i" class="line">
            <div class="l-body">
              <div
                v-if="lang !== 'en'"
                class="l-zh copyable"
                :title="t('report.lineCopyHint')"
                @click="copyLine(l, 'zh')"
              >
                {{ l.text }}
              </div>
              <div
                v-if="lang !== 'zh'"
                class="l-en copyable"
                :class="{ missing: lang === 'both' && !l.en }"
                :title="t('report.lineCopyHint')"
                @click="copyLine(l, 'en')"
              >
                {{ l.en || (lang === "en" ? l.text : t("report.missingEn")) }}
              </div>
            </div>
            <span class="l-min num">{{ fmtDur(l.minutes) }}</span>
          </div>
        </section>

        <div class="p-foot">
          <span>{{ t("report.total") }}</span>
          <b class="num">{{ fmtDur(report?.totalMinutes || 0) }}</b>
        </div>
      </article>
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
  flex-wrap: wrap;
}

.head h1 {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.seg {
  display: flex;
  gap: 2px;
  background: var(--sunken);
  padding: 2px;
  border-radius: 7px;
}

.seg button {
  height: 24px;
  padding: 0 9px;
  border-radius: 5px;
  font-size: 12px;
  color: var(--text-3);
}

.seg button:hover {
  color: var(--text-2);
}

.seg button.on {
  background: var(--surface);
  color: var(--text);
  font-weight: 500;
  border: 0.5px solid var(--line);
}

.dates {
  display: flex;
  align-items: center;
  gap: 5px;
}

.dates .input {
  height: 26px;
  font-size: 12px;
  width: 132px;
}

.toolbar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 7px;
  flex: none;
}

.meta {
  font-size: 11.5px;
  color: var(--text-3);
  margin-left: 4px;
}

.scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  justify-content: center;
  padding: 2px 0 10px;
}

.paper {
  width: 100%;
  max-width: 720px;
  background: var(--surface);
  border: 0.5px solid var(--line);
  border-radius: var(--r);
  padding: 26px 30px 20px;
  align-self: flex-start;
  user-select: text;
}

.p-head {
  margin-bottom: 20px;
}

.p-head h2 {
  margin: 0;
  font-size: 17px;
  font-weight: 500;
}

.sub {
  font-size: 12px;
  color: var(--text-3);
  margin-top: 3px;
}

.group {
  margin-bottom: 18px;
}

.g-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding-bottom: 6px;
  border-bottom: 0.5px solid var(--line);
  margin-bottom: 6px;
}

.g-head h3 {
  margin: 0;
  font-size: 12.5px;
  font-weight: 500;
}

.g-min {
  margin-left: auto;
  font-size: 11.5px;
  color: var(--text-3);
}

.line {
  display: flex;
  gap: 14px;
  padding: 4px 0 4px 12px;
  position: relative;
}

.line::before {
  content: "";
  position: absolute;
  left: 2px;
  top: 10px;
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: var(--line-strong);
}

.l-body {
  flex: 1;
  min-width: 0;
}

.copyable {
  cursor: copy;
  border-radius: 3px;
  transition:
    background 0.12s ease,
    box-shadow 0.12s ease;
}

.copyable:hover {
  background: var(--sunken);
  box-shadow: 0 0 0 3px var(--sunken);
}

.l-zh {
  font-size: 13px;
  line-height: 1.55;
}

.l-en {
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-3);
  margin-top: 1px;
}

.l-en.missing {
  color: var(--text-3);
  opacity: 0.65;
  font-style: italic;
}

.l-min {
  font-size: 12px;
  color: var(--text-3);
  white-space: nowrap;
  padding-top: 2px;
}

.p-foot {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  border-top: 0.5px solid var(--line);
  padding-top: 10px;
  margin-top: 6px;
  font-size: 12.5px;
  color: var(--text-2);
}

.p-foot b {
  font-weight: 500;
  color: var(--text);
  font-size: 13.5px;
}

.empty {
  color: var(--text-3);
  font-size: 12.5px;
  text-align: center;
  padding: 40px 0;
}
</style>
