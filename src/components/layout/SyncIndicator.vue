<script setup lang="ts">
import { computed } from "vue";
import { useDialogsStore } from "@/stores/dialogs";
import { useToastsStore } from "@/stores/toasts";
import { useWorkspacesStore } from "@/stores/workspaces";

const workspaces = useWorkspacesStore();
const dialogs = useDialogsStore();
const toasts = useToastsStore();

const folderName = computed(() => workspaces.sync.folder?.split(/[\\/]/).pop() ?? "");

async function showProblems() {
  const list = workspaces.sync.problems.slice(0, 8).join("\n");
  const ok = await dialogs.confirm(
    "Sinkron folder dijeda",
    `File berikut tidak bisa dibaca. Selama dijeda, file di folder tidak diubah. Perbaiki file lalu simpan; sinkron berlanjut otomatis.\n\n${list}`,
    "Coba lagi",
    false,
  );
  if (ok) await toasts.guard(() => workspaces.retrySync());
}
</script>

<template>
  <template v-if="workspaces.sync.folder">
    <button
      v-if="workspaces.sync.problems.length"
      type="button"
      class="flex items-center gap-1.5 text-warning hover:underline"
      @click="showProblems"
    >
      <span class="size-1.5 rounded-full bg-current" />
      Sinkron dijeda
    </button>
    <span v-else class="flex min-w-0 items-center gap-1.5 truncate" :title="workspaces.sync.folder">
      <span class="size-1.5 shrink-0 rounded-full bg-success" />
      Tersinkron: {{ folderName }}
    </span>
  </template>
</template>
