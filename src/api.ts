import { invoke } from "@tauri-apps/api/core";
import type {
  Entry,
  EntryInput,
  ImportResult,
  ProjectInput,
  Report,
  Settings,
  Snapshot,
  TaskInput,
  TimerState,
} from "./types";

export const snapshot = (date: string) => invoke<Snapshot>("snapshot", { date });

export const appInfo = () =>
  invoke<{ dbPath: string; version: string; view: string; zoom: number | null }>("app_info");

export const saveProject = (project: ProjectInput) =>
  invoke<number>("save_project", { project });

export const deleteProject = (id: number) => invoke<void>("delete_project", { id });

export const saveTask = (task: TaskInput) => invoke<number>("save_task", { task });

export const deleteTask = (id: number) => invoke<void>("delete_task", { id });

export const setTaskStatus = (id: number, status: string) =>
  invoke<void>("set_task_status", { id, status });

export const setTaskPlan = (id: number, plannedDate: string | null) =>
  invoke<void>("set_task_plan", { id, plannedDate });

export const startTimer = (taskId: number | null, note = "") =>
  invoke<TimerState | null>("start_timer", { taskId, note });

export const stopTimer = (note?: string) =>
  invoke<Entry | null>("stop_timer", { note: note ?? null });

export const addEntry = (entry: EntryInput) => invoke<number>("add_entry", { entry });

export const updateEntry = (entry: EntryInput) => invoke<void>("update_entry", { entry });

export const deleteEntry = (id: number) => invoke<void>("delete_entry", { id });

export const buildReport = (d0: string, d1: string, lang: string) =>
  invoke<Report>("build_report", { d0, d1, lang });

export const saveSettings = (settings: Settings) =>
  invoke<void>("save_settings", { settings });

export const importKanri = (path?: string) =>
  invoke<ImportResult>("import_kanri", { path: path ?? null });

export const saveTextFile = (path: string, content: string) =>
  invoke<void>("save_text_file", { path, content });

export const writeTempHtml = (content: string) =>
  invoke<string>("write_temp_html", { content });

export const translateMissing = (texts: string[]) =>
  invoke<{ zh: string; en: string }[]>("translate_missing", { texts });

export const polishText = (text: string) => invoke<string>("polish_text", { text });

export const testAi = () => invoke<string>("test_ai");
