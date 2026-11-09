import { ref } from "vue";
import { getCurrentWebview } from "@tauri-apps/api/webview";

const KEY = "wp.zoom";
const STEPS = [0.8, 0.9, 1, 1.1, 1.25, 1.4, 1.6];

function stored(): number {
  const raw = Number(localStorage.getItem(KEY));
  return STEPS.includes(raw) ? raw : 1;
}

export const zoomLevel = ref(stored());

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

let applied = 0;

export async function applyZoom(scale = zoomLevel.value, persist = true) {
  const i = STEPS.indexOf(scale);
  zoomLevel.value = i >= 0 ? scale : 1;
  if (persist) localStorage.setItem(KEY, String(zoomLevel.value));
  if (zoomLevel.value === applied) return;
  applied = zoomLevel.value;
  if (!inTauri) return; // 浏览器里没有这个接口
  try {
    await getCurrentWebview().setZoom(zoomLevel.value);
  } catch (e) {
    console.error("缩放失败", e);
  }
}

function step(dir: number) {
  const i = STEPS.indexOf(zoomLevel.value);
  const j = Math.min(STEPS.length - 1, Math.max(0, (i < 0 ? STEPS.indexOf(1) : i) + dir));
  applyZoom(STEPS[j]);
}

export const zoomIn = () => step(1);
export const zoomOut = () => step(-1);
export const zoomReset = () => applyZoom(1);
