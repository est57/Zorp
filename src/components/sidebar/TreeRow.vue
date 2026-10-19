<script setup lang="ts">
import { ref } from "vue";
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuPortal,
  ContextMenuRoot,
  ContextMenuTrigger,
} from "reka-ui";
import AppIcon from "@/components/ui/AppIcon.vue";
import type { IconName } from "@/components/ui/icons";
import type { DropZone } from "@/utils/order";

export interface MenuItem {
  label: string;
  icon: IconName;
  danger?: boolean;
  run: () => void;
}

const props = withDefaults(
  defineProps<{
    label: string;
    depth: number;
    selected?: boolean;
    expanded?: boolean;
    draggable?: boolean;
    /** inside: drop into only; edges: before or after; all: both. */
    zones: "inside" | "edges" | "all";
    menu: MenuItem[];
  }>(),
  { selected: false, expanded: undefined, draggable: false },
);
const emit = defineEmits<{ activate: []; dragstart: []; drop: [zone: DropZone] }>();

const zone = ref<DropZone | null>(null);

function zoneAt(event: DragEvent): DropZone {
  if (props.zones === "inside") return "inside";
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const ratio = (event.clientY - rect.top) / rect.height;
  if (props.zones === "edges") return ratio < 0.5 ? "before" : "after";
  return ratio < 0.25 ? "before" : ratio > 0.75 ? "after" : "inside";
}

function onDragStart(event: DragEvent) {
  // WebKit only starts a drag when data is attached.
  event.dataTransfer?.setData("text/plain", props.label);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  emit("dragstart");
}

function onDrop(event: DragEvent) {
  const at = zoneAt(event);
  zone.value = null;
  emit("drop", at);
}
</script>

<template>
  <ContextMenuRoot>
    <ContextMenuTrigger as-child>
      <button
        type="button"
        role="treeitem"
        :aria-expanded="expanded"
        :aria-selected="selected"
        :draggable="draggable"
        class="flex h-7 w-full min-w-0 items-center gap-1.5 rounded pr-2 text-left text-xs"
        :class="[
          selected ? 'bg-accent-soft' : 'hover:bg-surface-2',
          zone === 'inside' && 'outline-1 outline-accent outline-dashed',
          zone === 'before' && 'shadow-[inset_0_2px_0_var(--accent)]',
          zone === 'after' && 'shadow-[inset_0_-2px_0_var(--accent)]',
        ]"
        :style="{ paddingLeft: `${4 + depth * 12}px` }"
        @click="emit('activate')"
        @dragstart="onDragStart"
        @dragover.prevent="zone = zoneAt($event)"
        @dragleave="zone = null"
        @drop.prevent="onDrop"
      >
        <slot name="lead" />
        <span class="truncate">{{ label }}</span>
      </button>
    </ContextMenuTrigger>
    <ContextMenuPortal>
      <ContextMenuContent
        class="z-50 min-w-44 rounded-md border border-line bg-surface p-1 text-xs shadow-float"
      >
        <ContextMenuItem
          v-for="item in menu"
          :key="item.label"
          class="flex h-7 cursor-default items-center gap-2 rounded px-2 outline-none data-highlighted:bg-surface-2"
          :class="item.danger ? 'text-danger' : 'text-fg'"
          @select="item.run"
        >
          <AppIcon :name="item.icon" :size="14" />
          {{ item.label }}
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenuPortal>
  </ContextMenuRoot>
</template>
