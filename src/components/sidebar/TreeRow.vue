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

export interface MenuItem {
  label: string;
  icon: IconName;
  danger?: boolean;
  run: () => void;
}

defineProps<{
  label: string;
  depth: number;
  selected?: boolean;
  expanded?: boolean;
  draggable?: boolean;
  droppable?: boolean;
  menu: MenuItem[];
}>();
const emit = defineEmits<{ activate: []; dragstart: []; drop: [] }>();

const over = ref(false);

function onDrop() {
  over.value = false;
  emit("drop");
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
          over && 'outline-1 outline-accent outline-dashed',
        ]"
        :style="{ paddingLeft: `${4 + depth * 12}px` }"
        @click="emit('activate')"
        @dragstart="emit('dragstart')"
        @dragover.prevent="droppable && (over = true)"
        @dragleave="over = false"
        @drop.prevent="droppable && onDrop()"
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
