<script setup lang="ts">
import { useToastsStore } from "@/stores/toasts";

const toasts = useToastsStore();
</script>

<template>
  <div
    class="pointer-events-none fixed right-4 bottom-8 z-50 grid w-[min(360px,calc(100vw-32px))] gap-2"
    aria-live="polite"
  >
    <div
      v-for="toast in toasts.items"
      :key="toast.id"
      role="status"
      class="pointer-events-auto flex items-start gap-2.5 rounded-md border border-line bg-surface px-3 py-2.5 text-xs shadow-float"
    >
      <span
        class="mt-1 size-2 shrink-0 rounded-full"
        :class="{
          'bg-success': toast.tone === 'success',
          'bg-danger': toast.tone === 'danger',
          'bg-accent': toast.tone === 'info',
        }"
      />
      <span class="min-w-0 flex-1 break-words">{{ toast.message }}</span>
      <button
        type="button"
        class="text-subtle hover:text-fg"
        aria-label="Tutup notifikasi"
        @click="toasts.dismiss(toast.id)"
      >
        Tutup
      </button>
    </div>
  </div>
</template>
