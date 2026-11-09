export const pad = (n: number) => String(n).padStart(2, "0");

export function todayISO(): string {
  const d = new Date();
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function isoOf(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function parseISO(s: string): Date {
  const [y, m, d] = s.split("-").map(Number);
  return new Date(y, (m || 1) - 1, d || 1);
}

export function shiftISO(s: string, days: number): string {
  const d = parseISO(s);
  d.setDate(d.getDate() + days);
  return isoOf(d);
}

export function fmtDur(min: number): string {
  if (!min || min <= 0) return "0m";
  if (min < 60) return `${Math.round(min)}m`;
  const h = Math.floor(min / 60);
  const m = Math.round(min % 60);
  return m === 0 ? `${h}h` : `${h}h${pad(m)}m`;
}

export function fmtClock(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  return `${pad(h)}:${pad(m)}:${pad(s)}`;
}

const WEEK_ZH = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

export function weekdayZh(d: Date): string {
  return WEEK_ZH[d.getDay()];
}

export function fmtDay(iso: string): string {
  const d = parseISO(iso);
  return `${d.getMonth() + 1}月${d.getDate()}日 ${weekdayZh(d)}`;
}

export function fmtDayShort(iso: string): string {
  const d = parseISO(iso);
  return `${d.getMonth() + 1}/${d.getDate()}`;
}

export function dueLabel(due: string | null, date: string): string {
  if (!due) return "";
  const diff = Math.round(
    (parseISO(due).getTime() - parseISO(date).getTime()) / 86400000,
  );
  if (diff < 0) return `逾期 ${Math.abs(diff)} 天`;
  if (diff === 0) return "今天到期";
  if (diff === 1) return "明天到期";
  if (diff < 7) return `${weekdayZh(parseISO(due))}到期`;
  return `${parseISO(due).getMonth() + 1}/${parseISO(due).getDate()} 到期`;
}

export function isOverdue(due: string | null, date: string): boolean {
  return !!due && due < date;
}

/** 解析 "1h30" / "90m" / "45" / "1.5h" 为分钟 */
export function parseDuration(raw: string): number | null {
  const s = raw.trim().toLowerCase().replace(/\s/g, "");
  if (!s) return null;
  let total = 0;
  let matched = false;
  const re = /(\d+(?:\.\d+)?)(h|m)?/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(s))) {
    const v = parseFloat(m[1]);
    if (Number.isNaN(v)) continue;
    matched = true;
    if (m[2] === "h") total += v * 60;
    else if (m[2] === "m") total += v;
    else total += s.endsWith("h") ? v * 60 : v;
  }
  if (!matched) return null;
  return Math.round(total);
}

export function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
