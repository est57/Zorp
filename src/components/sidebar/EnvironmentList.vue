<script setup lang="ts">
import { computed } from "vue";
import type { Environment } from "@/bindings/Environment";
import AppButton from "@/components/ui/AppButton.vue";
import AppIcon from "@/components/ui/AppIcon.vue";
import IconButton from "@/components/ui/IconButton.vue";
import { useExport } from "@/composables/useExport";
import { useImport } from "@/composables/useImport";
import { useDialogsStore } from "@/stores/dialogs";
import { useEnvironmentsStore } from "@/stores/environments";
import { useToastsStore } from "@/stores/toasts";
import EnvironmentEditor from "./EnvironmentEditor.vue";

const envs = useEnvironmentsStore();
const dialogs = useDialogsStore();
const toasts = useToastsStore();
const exporter = useExport();
const { environmentFromPostman } = useImport();

const GLOBAL = "global";
const editingGlobal = computed(() => dialogs.environmentEditor?.id === GLOBAL);
const editing = computed(() =>
  editingGlobal.value
    ? envs.globals
    : (envs.items.find((e) => e.id === dialogs.environmentEditor?.id) ?? null),
);
const editorOpen = computed({
  get: () => dialogs.environmentEditor !== null,
  set: (open) => {
    if (!open) dialogs.environmentEditor = null;
  },
});

const edit = (env: Environment | null) => (dialogs.environmentEditor = { id: env?.id ?? null });

async function remove(env: Environment) {
  const ok = await dialogs.confirm(
    "Hapus environment?",
    `"${env.name}" beserta nilai rahasianya akan dihapus permanen.`,
  );
  if (ok) await toasts.guard(() => envs.remove(env.id));
}
</script>

<template>
  <div class="grid gap-1 px-1.5 pb-3">
    <button
      type="button"
      class="flex h-8 items-center gap-2 rounded px-1.5 text-left text-xs font-medium hover:bg-surface-2"
      title="Variabel yang berlaku di semua koleksi workspace ini"
      @click="dialogs.environmentEditor = { id: GLOBAL }"
    >
      <AppIcon name="globe" :size="13" class="shrink-0 text-muted" />
      Global
      <span class="ml-auto text-[10px] font-normal text-subtle tabular-nums"
        >{{ envs.globals?.variables.length ?? 0 }} var</span
      >
    </button>
    <div class="mx-1.5 h-px bg-line" />
    <div v-if="envs.items.length === 0" class="grid gap-3 px-1.5 py-2 text-xs text-muted">
      <p>Belum ada environment. Environment menyimpan variabel seperti base URL dan token.</p>
      <div class="flex flex-wrap items-center gap-1">
        <AppButton icon="plus" @click="edit(null)">Buat environment</AppButton>
        <AppButton variant="ghost" icon="download" @click="environmentFromPostman"
          >Impor dari Postman</AppButton
        >
      </div>
    </div>
    <div
      v-for="env in envs.items"
      :key="env.id"
      class="group flex h-8 items-center gap-2 rounded px-1.5 text-xs hover:bg-surface-2"
      :class="env.isActive && 'bg-accent-soft'"
    >
      <button
        type="button"
        class="flex min-w-0 flex-1 items-center gap-2 text-left"
        :aria-pressed="env.isActive"
        @click="toasts.guard(() => envs.activate(env.isActive ? null : env.id))"
      >
        <span
          class="size-2 shrink-0 rounded-full"
          :class="env.isActive ? 'bg-success' : 'bg-line'"
        />
        <span class="truncate">{{ env.name }}</span>
        <span class="ml-auto text-[10px] text-subtle tabular-nums"
          >{{ env.variables.length }} var</span
        >
      </button>
      <IconButton
        icon="pencil"
        label="Edit environment"
        class="opacity-0 group-hover:opacity-100 focus:opacity-100"
        @click="edit(env)"
      />
      <IconButton
        icon="download"
        label="Ekspor ke Postman"
        class="opacity-0 group-hover:opacity-100 focus:opacity-100"
        @click="exporter.environmentToPostman(env)"
      />
      <IconButton
        icon="trash"
        label="Hapus environment"
        class="opacity-0 group-hover:opacity-100 focus:opacity-100"
        @click="remove(env)"
      />
    </div>
    <EnvironmentEditor v-model:open="editorOpen" :environment="editing" :global="editingGlobal" />
  </div>
</template>
