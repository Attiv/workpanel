<script setup lang="ts">
import { onBeforeUnmount, onMounted } from "vue";
import Icon from "./Icon.vue";

const props = withDefaults(defineProps<{ title: string; width?: number }>(), { width: 460 });
const emit = defineEmits<{ (e: "close"): void }>();

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    emit("close");
  }
}

onMounted(() => window.addEventListener("keydown", onKey));
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));
</script>

<template>
  <Teleport to="body">
    <div class="overlay" @pointerdown.self="emit('close')">
      <div class="sheet" :style="{ width: props.width + 'px' }" role="dialog" aria-modal="true">
        <header class="sheet-head">
          <h2>{{ props.title }}</h2>
          <button class="close" title="关闭" @click="emit('close')">
            <Icon name="close" :size="13" />
          </button>
        </header>
        <div class="sheet-body">
          <slot />
        </div>
        <footer v-if="$slots.footer" class="sheet-foot">
          <slot name="footer" />
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 90;
  background: color-mix(in srgb, #0b0f15 42%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 24px;
  animation: overlay-in 0.14s ease-out;
}

.sheet {
  background: var(--surface);
  border: 0.5px solid var(--line-strong);
  border-radius: 12px;
  box-shadow: 0 24px 60px rgba(10, 14, 20, 0.28);
  max-height: 100%;
  display: flex;
  flex-direction: column;
  animation: sheet-in 0.17s cubic-bezier(0.22, 1, 0.36, 1);
  overflow: hidden;
}

.sheet-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px 10px;
  flex: none;
}

.sheet-head h2 {
  margin: 0;
  font-size: 13.5px;
  font-weight: 500;
}

.close {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  display: grid;
  place-items: center;
  color: var(--text-3);
}

.close:hover {
  background: var(--sunken);
  color: var(--text);
}

.sheet-body {
  padding: 2px 14px 14px;
  overflow-y: auto;
}

.sheet-foot {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-top: 0.5px solid var(--line);
  background: color-mix(in srgb, var(--sunken) 55%, var(--surface));
  flex: none;
}

@keyframes overlay-in {
  from {
    opacity: 0;
  }
}

@keyframes sheet-in {
  from {
    opacity: 0;
    transform: translateY(6px) scale(0.97);
  }
}
</style>
