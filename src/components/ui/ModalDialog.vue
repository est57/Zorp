<script setup lang="ts">
import {
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";

defineProps<{ title: string; description?: string; wide?: boolean }>();
const open = defineModel<boolean>("open", { required: true });
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-40 bg-black/35" />
      <DialogContent
        class="fixed top-[12vh] left-1/2 z-50 grid max-h-[76vh] w-[calc(100vw-32px)] -translate-x-1/2 grid-rows-[auto_minmax(0,1fr)] gap-4 rounded-[10px] border border-line bg-surface p-5 text-fg shadow-float"
        :class="wide ? 'max-w-2xl' : 'max-w-md'"
        v-bind="description ? {} : { 'aria-describedby': undefined }"
      >
        <header class="grid gap-1">
          <DialogTitle class="text-base font-semibold">{{ title }}</DialogTitle>
          <DialogDescription v-if="description" class="text-xs whitespace-pre-line text-muted">{{
            description
          }}</DialogDescription>
        </header>
        <!-- Inset padding keeps focus rings of edge controls from being clipped by the scroller. -->
        <div class="-m-1 min-h-0 overflow-auto p-1">
          <slot />
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
