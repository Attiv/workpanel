export type Status = "todo" | "doing" | "done";

export interface Project {
  id: number;
  name: string;
  color: string;
  aliases: string[];
  archived: boolean;
  openTasks: number;
  todayMinutes: number;
  weekMinutes: number;
  totalMinutes: number;
}

export interface Task {
  id: number;
  projectId: number | null;
  projectName: string | null;
  color: string | null;
  title: string;
  note: string;
  status: Status;
  plannedDate: string | null;
  dueDate: string | null;
  estimateMin: number | null;
  todayMinutes: number;
  doneAt: string | null;
  createdAt: string;
}

export interface Entry {
  id: number;
  taskId: number | null;
  projectId: number | null;
  projectName: string | null;
  color: string | null;
  taskTitle: string | null;
  day: string;
  minutes: number;
  note: string;
  startedAt: string | null;
  endedAt: string | null;
}

export interface TimerState {
  taskId: number | null;
  projectId: number | null;
  projectName: string | null;
  taskTitle: string | null;
  note: string;
  startedAt: string;
}

export interface Settings {
  aiBase: string;
  aiModel: string;
  aiKey: string;
  workdayMinutes: number;
  reportLang: "zh" | "en" | "both";
}

export interface Stats {
  todayMinutes: number;
  weekMinutes: number;
  openTasks: number;
  todayEntries: number;
  todayTasks: number;
}

export interface SeriesPoint {
  day: string;
  projectId: number | null;
  minutes: number;
}

export interface Snapshot {
  date: string;
  projects: Project[];
  tasks: Task[];
  entries: Entry[];
  timer: TimerState | null;
  settings: Settings;
  stats: Stats;
  series: SeriesPoint[];
}

export interface ReportLine {
  text: string;
  en: string | null;
  minutes: number;
  taskId: number | null;
}

export interface ReportGroup {
  projectId: number | null;
  name: string;
  color: string;
  minutes: number;
  lines: ReportLine[];
}

export interface Report {
  markdown: string;
  groups: ReportGroup[];
  totalMinutes: number;
  missingEn: string[];
}

export interface ImportResult {
  path: string;
  boards: number;
  cards: number;
  createdTasks: number;
  createdProjects: number;
  skipped: number;
}

export interface TaskInput {
  id?: number;
  projectId: number | null;
  title: string;
  note: string;
  status: Status;
  plannedDate: string | null;
  dueDate: string | null;
  estimateMin: number | null;
}

export interface EntryInput {
  id?: number;
  projectId: number | null;
  taskId: number | null;
  day: string;
  minutes: number;
  note: string;
}

export interface ProjectInput {
  id?: number;
  name: string;
  color: string;
  aliases: string[];
}
