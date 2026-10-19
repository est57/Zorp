<script setup lang="ts">
import { ref } from "vue";
import { useLayoutStore } from "@/stores/layout";

const layout = useLayoutStore();
const container = ref<HTMLDivElement>();

function onPointerDown(event: PointerEvent) {
  const handle = event.currentTarget as HTMLElement;
  const box = container.value?.getBoundingClientRect();
  if (!box) return;
  handle.setPointerCapture(event.pointerId);
  const onMove = (move: PointerEvent) =>
    layout.setSplitRatio((move.clientY - box.top) / box.height);
  const onUp = () => {
    handle.removeEventListener("pointermove", onMove);
    handle.removeEventListener("pointerup", onUp);
  };
  handle.addEventListener("pointermove", onMove);
  handle.addEventListener("pointerup", onUp);
}

function onKeydown(event: KeyboardEvent) {
  const step = event.key === "ArrowUp" ? -0.05 : event.key === "ArrowDown" ? 0.05 : 0;
  if (!step) return;
  event.preventDefault();
  layout.setSplitRatio(layout.splitRatio + step);
}
</script>

<template>
  <div ref="container" class="flex min-h-0 flex-1 flex-col">
    <div class="min-h-0 bg-surface" :style="{ flexBasis: `${layout.splitRatio * 100}%` }">
      <slot name="top" />
    </div>
    <div
      role="separator"
      aria-orientation="horizontal"
      aria-label="Ubah tinggi panel request"
      :aria-valuenow="Math.round(layout.splitRatio * 100)"
      tabindex="0"
      class="group grid h-1.5 shrink-0 cursor-row-resize place-items-center border-y border-line bg-bg"
      @pointerdown="onPointerDown"
      @keydown="onKeydown"
    >
      <span
        class="h-0.5 w-8 rounded-full bg-line group-hover:bg-accent group-focus-visible:bg-accent"
      />
    </div>
    <div class="min-h-0 flex-1">
      <slot name="bottom" />
    </div>
  </div>
</template>
