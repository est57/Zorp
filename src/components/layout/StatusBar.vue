<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "@/stores/app";
import SyncIndicator from "./SyncIndicator.vue";
import UpdateNotice from "./UpdateNotice.vue";

const app = useAppStore();

const status = computed(() => {
  switch (app.status) {
    case "ready":
      return { text: "Database siap", tone: "text-success" };
    case "loading":
      return { text: "Memuat", tone: "text-muted" };
    case "unavailable":
      return { text: "Mode pratinjau browser, backend tidak aktif", tone: "text-warning" };
    default:
      return { text: `Gagal memuat backend: ${app.errorMessage}`, tone: "text-danger" };
  }
});
</script>

<template>
  <footer
    class="flex h-6 shrink-0 items-center gap-3.5 overflow-hidden border-t border-line bg-surface px-3 text-[11px] whitespace-nowrap text-muted"
  >
    <span class="flex min-w-0 items-center gap-1.5 truncate" :class="status.tone" role="status">
      <span class="size-1.5 shrink-0 rounded-full bg-current" />
      {{ status.text }}
    </span>
    <SyncIndicator />
    <span class="flex-1" />
    <UpdateNotice />
    <template v-if="app.info">
      <span class="hidden tabular-nums sm:inline">Skema v{{ app.info.schemaVersion }}</span>
      <span class="tabular-nums">Zorp {{ app.info.version }}</span>
    </template>
  </footer>
</template>
