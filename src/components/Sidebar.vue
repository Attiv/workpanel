<script setup lang="ts">
import { computed } from "vue";
import Icon from "./Icon.vue";
import { setView, state } from "../store";
import { fmtDur } from "../utils";
import { zoomIn, zoomLevel, zoomOut, zoomReset } from "../zoom";
import { locale, setLocale, t } from "../i18n";

const nav = computed(() => {
  const s = state.snap;
  return [
    { key: "today", label: t("nav.today"), icon: "board", badge: s?.stats.todayTasks || 0 },
    { key: "projects", label: t("nav.projects"), icon: "folder", badge: s?.projects.length || 0 },
    { key: "tasks", label: t("nav.tasks"), icon: "list", badge: s?.stats.openTasks || 0 },
    { key: "report", label: t("nav.report"), icon: "doc", badge: 0 },
    { key: "settings", label: t("nav.settings"), icon: "gear", badge: 0 },
  ] as const;
});
</script>

<template>
  <aside class="side">
    <div class="brand">
      <span class="mark" aria-hidden="true"><i></i><i></i><i></i></span>
      <span class="name">WorkPanel</span>
      <div class="lang" role="group" :aria-label="t('side.language')">
        <button :class="{ on: locale === 'zh' }" :title="t('common.language')" @click="setLocale('zh')">
          中
        </button>
        <button :class="{ on: locale === 'en' }" :title="t('common.language')" @click="setLocale('en')">
          EN
        </button>
      </div>
    </div>

    <nav class="nav">
      <button
        v-for="item in nav"
        :key="item.key"
        class="nav-item"
        :class="{ on: state.view === item.key }"
        @click="setView(item.key)"
      >
        <Icon :name="item.icon" :size="15" />
        <span class="label">{{ item.label }}</span>
        <span v-if="item.badge" class="badge num">{{ item.badge }}</span>
      </button>
    </nav>

    <div class="foot">
      <div class="row">
        <span>{{ t("side.today") }}</span>
        <b class="num">{{ fmtDur(state.snap?.stats.todayMinutes || 0) }}</b>
      </div>
      <div class="row">
        <span>{{ t("side.week") }}</span>
        <b class="num">{{ fmtDur(state.snap?.stats.weekMinutes || 0) }}</b>
      </div>
      <div class="row muted">
        <span>{{ t("side.open") }}</span>
        <b class="num">{{ t("side.items", { n: state.snap?.stats.openTasks || 0 }) }}</b>
      </div>

      <div class="zoom">
        <button class="zbtn" :title="t('side.zoomOut')" @click="zoomOut">−</button>
        <button class="zbtn val num" :title="t('side.zoomReset')" @click="zoomReset">
          {{ Math.round(zoomLevel * 100) }}%
        </button>
        <button class="zbtn" :title="t('side.zoomIn')" @click="zoomIn">+</button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.side {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 16px 12px 14px;
  border-right: 0.5px solid var(--line);
  background: var(--bg);
  min-height: 0;
}

.brand {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 0 6px;
}

.mark {
  display: flex;
  align-items: flex-end;
  gap: 2.5px;
  height: 15px;
}

.mark i {
  width: 4px;
  border-radius: 1.5px;
  background: var(--accent);
}
.mark i:nth-child(1) {
  height: 7px;
  opacity: 0.5;
}
.mark i:nth-child(2) {
  height: 11px;
  opacity: 0.75;
}
.mark i:nth-child(3) {
  height: 15px;
}

.name {
  font-size: 13px;
  font-weight: 500;
}

.lang {
  margin-left: auto;
  display: flex;
  gap: 1px;
  background: var(--sunken);
  border-radius: 5px;
  padding: 1px;
}

.lang button {
  height: 17px;
  min-width: 19px;
  padding: 0 4px;
  border-radius: 4px;
  font-size: 10px;
  line-height: 1;
  color: var(--text-3);
}

.lang button:hover {
  color: var(--text-2);
}

.lang button.on {
  background: var(--surface);
  color: var(--text);
  box-shadow: 0 0 0 0.5px var(--line);
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 9px;
  height: 30px;
  padding: 0 9px;
  border-radius: var(--r-sm);
  color: var(--text-2);
  font-size: 13px;
  border: 0.5px solid transparent;
  transition: background-color 0.12s ease, color 0.12s ease;
}

.nav-item:hover {
  background: color-mix(in srgb, var(--text) 5%, transparent);
  color: var(--text);
}

.nav-item.on {
  background: var(--surface);
  border-color: var(--line);
  color: var(--text);
  font-weight: 500;
}

.label {
  flex: 1;
  text-align: left;
}

.badge {
  font-size: 11px;
  color: var(--text-3);
}

.foot {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 9px;
  border-top: 0.5px solid var(--line);
  font-size: 11.5px;
}

.row {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  color: var(--text-2);
}

.row b {
  font-weight: 500;
  color: var(--text);
}

.row.muted,
.row.muted b {
  color: var(--text-3);
  font-weight: 400;
}

.zoom {
  display: flex;
  align-items: center;
  gap: 1px;
  margin-top: 8px;
  background: var(--sunken);
  border-radius: 6px;
  padding: 2px;
}

.zbtn {
  flex: 1;
  height: 20px;
  border-radius: 4px;
  font-size: 12px;
  color: var(--text-3);
  display: grid;
  place-items: center;
  transition: background-color 0.12s ease, color 0.12s ease;
}

.zbtn:hover {
  background: var(--surface);
  color: var(--text);
}

.zbtn.val {
  flex: 1.6;
  font-size: 10.5px;
}
</style>
