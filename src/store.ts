import { reactive } from "vue";
import * as api from "./api";
import type {
  Entry,
  EntryInput,
  Project,
  ProjectInput,
  Settings,
  Snapshot,
  Task,
  TaskInput,
} from "./types";
import { shiftISO, todayISO } from "./utils";

export type View = "today" | "projects" | "tasks" | "report" | "settings";
export type Column = "backlog" | "today" | "doing" | "done";

export type Modal =
  | { type: "task"; task?: Task; column?: Column }
  | { type: "entry"; entry?: Entry; day?: string; task?: Task }
  | { type: "project"; project?: Project }
  | null;

interface ToastItem {
  id: number;
  text: string;
  kind: "ok" | "err";
}

export const state = reactive({
  view: "today" as View,
  date: todayISO(),
  snap: null as Snapshot | null,
  loading: true,
  now: Date.now(),
  toasts: [] as ToastItem[],
  modal: null as Modal,
  toastSeq: 0,
  error: "",
});

export function toast(text: string, kind: "ok" | "err" = "ok") {
  const id = ++state.toastSeq;
  state.toasts.push({ id, text, kind });
  window.setTimeout(() => {
    const i = state.toasts.findIndex((t) => t.id === id);
    if (i >= 0) state.toasts.splice(i, 1);
  }, kind === "err" ? 4200 : 2200);
}

export async function refresh() {
  try {
    state.snap = await api.snapshot(state.date);
    state.error = "";
  } catch (e) {
    state.error = String(e);
    toast(String(e), "err");
  } finally {
    state.loading = false;
  }
}

export function setView(view: View) {
  state.view = view;
}

export async function setDate(date: string) {
  state.date = date;
  await refresh();
}

export async function stepDate(days: number) {
  await setDate(shiftISO(state.date, days));
}

export async function moveTask(task: Task, column: Column) {
  try {
    if (column === "backlog") {
      await api.setTaskStatus(task.id, "todo");
      await api.setTaskPlan(task.id, null);
    } else if (column === "today") {
      // 从「进行中」拖回来 = 今天还要做，但先不算在做了：结算掉计时
      if (task.status === "doing") {
        if (state.snap?.timer?.taskId === task.id) await api.stopTimer();
        await api.setTaskStatus(task.id, "todo");
      }
      await api.setTaskPlan(task.id, state.date);
      if (task.status === "done") await api.setTaskStatus(task.id, "todo");
    } else if (column === "doing") {
      await api.setTaskPlan(task.id, state.date);
      if (task.status === "done") await api.setTaskStatus(task.id, "todo");
      if (state.snap?.timer?.taskId === task.id) {
        await api.stopTimer();
      } else {
        await api.startTimer(task.id, "");
      }
    } else if (column === "done") {
      if (state.snap?.timer?.taskId === task.id) await api.stopTimer();
      await api.setTaskPlan(task.id, state.date);
      await api.setTaskStatus(task.id, "done");
    }
    await refresh();
  } catch (e) {
    toast(String(e), "err");
  }
}

export async function toggleDone(task: Task) {
  try {
    await api.setTaskStatus(task.id, task.status === "done" ? "todo" : "done");
    await refresh();
  } catch (e) {
    toast(String(e), "err");
  }
}

export async function startTask(task: Task) {
  try {
    if (state.snap?.timer?.taskId === task.id) {
      await api.stopTimer();
      await refresh();
      return;
    }
    await api.setTaskPlan(task.id, state.date);
    await api.startTimer(task.id, "");
    await refresh();
  } catch (e) {
    toast(String(e), "err");
  }
}

export async function stopTimerFlow(note?: string) {
  try {
    const entry = await api.stopTimer(note);
    await refresh();
    if (entry) toast(`已记录 ${entry.minutes} 分钟`);
  } catch (e) {
    toast(String(e), "err");
  }
}

export async function saveTaskFlow(input: TaskInput) {
  try {
    await api.saveTask(input);
    await refresh();
    toast(input.id ? "任务已更新" : "任务已添加");
    return true;
  } catch (e) {
    toast(String(e), "err");
    return false;
  }
}

export async function deleteTaskFlow(id: number) {
  try {
    await api.deleteTask(id);
    await refresh();
    toast("任务已删除");
  } catch (e) {
    toast(String(e), "err");
  }
}

export async function saveEntryFlow(input: EntryInput) {
  try {
    if (input.id) await api.updateEntry(input);
    else await api.addEntry(input);
    await refresh();
    toast(input.id ? "记录已更新" : "已记录");
    return true;
  } catch (e) {
    toast(String(e), "err");
    return false;
  }
}

export async function deleteEntryFlow(id: number) {
  try {
    await api.deleteEntry(id);
    await refresh();
    toast("记录已删除");
  } catch (e) {
    toast(String(e), "err");
  }
}

export async function saveProjectFlow(input: ProjectInput) {
  try {
    const id = await api.saveProject(input);
    await refresh();
    toast(input.id ? "项目已更新" : "项目已创建");
    return id;
  } catch (e) {
    toast(String(e), "err");
    return null;
  }
}

export async function deleteProjectFlow(id: number) {
  try {
    await api.deleteProject(id);
    await refresh();
    toast("项目已删除");
  } catch (e) {
    toast(String(e), "err");
  }
}

export async function saveSettingsFlow(settings: Settings) {
  try {
    await api.saveSettings(settings);
    await refresh();
    toast("设置已保存");
  } catch (e) {
    toast(String(e), "err");
  }
}

export function openTaskModal(task?: Task, column?: Column) {
  state.modal = { type: "task", task, column };
}

export function openEntryModal(entry?: Entry, day?: string, task?: Task) {
  state.modal = { type: "entry", entry, day: day ?? state.date, task };
}

export function openProjectModal(project?: Project) {
  state.modal = { type: "project", project };
}

export function closeModal() {
  state.modal = null;
}
