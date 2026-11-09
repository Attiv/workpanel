<script setup lang="ts">
const props = defineProps<{ name: string; size?: number }>();

const paths: Record<string, string> = {
  board: "M4 5.5h5.5v13H4zM14.5 5.5H20v8h-5.5z",
  folder: "M3.5 7.5a2 2 0 0 1 2-2h3l2 2h6a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-11a2 2 0 0 1-2-2z",
  list: "M8 6h12M8 12h12M8 18h12M4.2 6h.01M4.2 12h.01M4.2 18h.01",
  doc: "M6 3.5h7l4 4v13H6zM13 3.5V8h4",
  gear: "M12 15.2a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4zM12 3v2.2M12 18.8V21M4.2 7.4l1.9 1.1M17.9 15.5l1.9 1.1M4.2 16.6l1.9-1.1M17.9 8.5l1.9-1.1",
  play: "M8 5.2 18.4 12 8 18.8z",
  stop: "M7.5 7.5h9v9h-9z",
  plus: "M12 5v14M5 12h14",
  left: "M14.5 5.5 8 12l6.5 6.5",
  right: "M9.5 5.5 16 12l-6.5 6.5",
  calendar: "M4.5 6.5h15v13h-15zM8 4v4M16 4v4M4.5 10.8h15",
  sparkle: "M12 4l1.6 4.4L18 10l-4.4 1.6L12 16l-1.6-4.4L6 10l4.4-1.6zM18.5 15.5l.8 2.2 2.2.8-2.2.8-.8 2.2-.8-2.2-2.2-.8 2.2-.8z",
  copy: "M9 9.5h10.5V20H9zM5 15.5V4h10.5",
  download: "M12 4v10.5M7.8 10.4 12 14.6l4.2-4.2M5 19.5h14",
  printer: "M7 9V4h10v5M7 17H4.8V9h14.4v8H17M7.5 14h9v6.2h-9z",
  trash: "M5 7h14M9.2 7V4.8h5.6V7M7.2 7l.9 12.7h7.8L16.8 7",
  edit: "M5 19h4L20 8l-4-4L5 15z",
  check: "M5 12.5 9.5 17 19 7",
  close: "M6 6l12 12M18 6 6 18",
  search: "M11 4.2a6.8 6.8 0 1 0 0 13.6 6.8 6.8 0 0 0 0-13.6zM16.2 16.2 21 21",
  timer: "M12 8.2V13l3.2 1.9M12 3.8a8.2 8.2 0 1 0 0 16.4 8.2 8.2 0 0 0 0-16.4z",
  arrow: "M5 12h13M13 7l5 5-5 5",
  image: "M4.5 5.5h15v13h-15zM4.5 16l4.6-4.6 3.4 3.4 2.4-2.4 4.6 4.6M15.6 9.3h.01",
};

const filled = new Set(["play", "stop"]);
const d = () => paths[props.name] ?? "";
</script>

<template>
  <svg
    :width="size ?? 15"
    :height="size ?? 15"
    viewBox="0 0 24 24"
    :fill="filled.has(name) ? 'currentColor' : 'none'"
    stroke="currentColor"
    :stroke-width="filled.has(name) ? 0 : 1.7"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    focusable="false"
  >
    <path :d="d()" />
  </svg>
</template>
