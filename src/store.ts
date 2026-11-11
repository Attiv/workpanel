import { reactive } from "vue";
import * as api from "./api";
import type {
  AttachmentDraft,
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
import { t } from "./i18n";

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
  /** 真正的「今天」，跨天时由 syncToday() 推进 */
  today: todayISO(),
  /** 当前查看的日期，默认跟着 today 走，也可以手动翻到别的日子 */
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

/**
 * 跨天检查。应用常驻时 `date` 只在启动那一刻算过，过了午夜会停在昨天：
 * 新建任务还写昨天、昨天的「今天」也不让位。这里由计时器每秒调一次，
 * 窗口重新获得焦点时也会调（合盖过夜醒来同样能补上）。
 *
 * 只有用户当前正看着「今天」时才跟着跳到新的一天；翻到历史日期时不打扰。
 */
export async function syncToday() {
  const now = todayISO();
  if (now === state.today) return false;
  const following = state.date === state.today;
  state.today = now;
  if (!following) return false;
  state.date = now;
  await refresh();
  toast(t("toast.newDay", { day: now.slice(5) }));
  return true;
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
      if (state.snap?.timer?.taskId === task.id) await api.stopTimer();
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
    if (entry) toast(t("toast.minutes", { n: entry.minutes }));
  } catch (e) {
    toast(String(e), "err");
  }
}

export async function saveTaskFlow(input: TaskInput) {
  try {
    const id = await api.saveTask(input);
    await refresh();
    toast(t(input.id ? "toast.taskUpdated" : "toast.taskAdded"));
    return id;
  } catch (e) {
    toast(String(e), "err");
    return null;
  }
}

export async function deleteTaskFlow(id: number) {
  try {
    await api.deleteTask(id);
    await refresh();
    toast(t("toast.taskDeleted"));
  } catch (e) {
    toast(String(e), "err");
  }
}

export async function saveEntryFlow(input: EntryInput) {
  try {
    let id = input.id;
    if (input.id) await api.updateEntry(input);
    else id = await api.addEntry(input);
    await refresh();
    toast(t(input.id ? "toast.entryUpdated" : "toast.entryAdded"));
    return id ?? null;
  } catch (e) {
    toast(String(e), "err");
    return null;
  }
}

/**
 * 对话框里的图片：新增的入库，标了删除的删掉。
 * 必须等主体有了 id 才能调用。
 */
export async function syncAttachments(
  ownerType: "task" | "entry",
  ownerId: number,
  drafts: AttachmentDraft[],
) {
  try {
    for (const draft of drafts) {
      if (draft.removed) {
        if (draft.id) await api.deleteAttachment(draft.id);
      } else if (!draft.id) {
        await api.addAttachment({
          ownerType,
          ownerId,
          dataUrl: draft.dataUrl,
          width: draft.width,
          height: draft.height,
        });
      }
    }
    await refresh();
    return true;
  } catch {
    toast(t("dlg.attachSaveFailed"), "err");
    return false;
  }
}

export async function deleteEntryFlow(id: number) {
  try {
    await api.deleteEntry(id);
    await refresh();
    toast(t("toast.entryDeleted"));
  } catch (e) {
    toast(String(e), "err");
  }
}

export async function saveProjectFlow(input: ProjectInput) {
  try {
    const id = await api.saveProject(input);
    await refresh();
    toast(t(input.id ? "toast.projectUpdated" : "toast.projectAdded"));
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
    toast(t("toast.projectDeleted"));
  } catch (e) {
    toast(String(e), "err");
  }
}

export async function saveSettingsFlow(settings: Settings) {
  try {
    await api.saveSettings(settings);
    await refresh();
    toast(t("toast.settingsSaved"));
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
