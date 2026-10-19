<script setup lang="ts">
import { SIDEBAR_MAX, SIDEBAR_MIN } from "@/stores/layout";

const props = defineProps<{ width: number }>();
const emit = defineEmits<{ resize: [width: number] }>();

const KEY_STEP = 16;

function onPointerDown(event: PointerEvent) {
  const target = event.currentTarget as HTMLElement;
  const startX = event.clientX;
  const startWidth = props.width;
  target.setPointerCapture(event.pointerId);

  const onMove = (move: PointerEvent) => emit("resize", startWidth + move.clientX - startX);
  const onUp = () => {
    target.removeEventListener("pointermove", onMove);
    target.removeEventListener("pointerup", onUp);
    document.body.style.cursor = "";
  };
  document.body.style.cursor = "col-resize";
  target.addEventListener("pointermove", onMove);
  target.addEventListener("pointerup", onUp);
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "ArrowLeft") emit("resize", props.width - KEY_STEP);
  else if (event.key === "ArrowRight") emit("resize", props.width + KEY_STEP);
  else return;
  event.preventDefault();
}
</script>

<template>
  <div
    role="separator"
    aria-orientation="vertical"
    aria-label="Ubah lebar sidebar"
    :aria-valuenow="width"
    :aria-valuemin="SIDEBAR_MIN"
    :aria-valuemax="SIDEBAR_MAX"
    tabindex="0"
    class="group absolute inset-y-0 -right-1 z-10 w-2 cursor-col-resize"
    @pointerdown="onPointerDown"
    @keydown="onKeydown"
  >
    <span
      class="mx-auto block h-full w-px bg-transparent transition-colors group-hover:bg-accent group-focus-visible:bg-accent"
    />
  </div>
</template>
