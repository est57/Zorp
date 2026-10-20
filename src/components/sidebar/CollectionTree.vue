<script setup lang="ts">
import type { Collection } from "@/bindings/Collection";
import AppButton from "@/components/ui/AppButton.vue";
import AppIcon from "@/components/ui/AppIcon.vue";
import { useExport } from "@/composables/useExport";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import TreeBranch from "./TreeBranch.vue";
import TreeRow, { type MenuItem } from "./TreeRow.vue";
import { useTreeActions } from "./useTreeActions";

const collections = useCollectionsStore();
const actions = useTreeActions();
const dialogs = useDialogsStore();
const exporter = useExport();

const menu = (c: Collection): MenuItem[] => [
  { label: "Request baru", icon: "plus", run: () => actions.newRequestIn(c.id, null) },
  { label: "Folder baru", icon: "folderPlus", run: () => actions.newFolder(c.id, null) },
  {
    label: "Jalankan",
    icon: "send",
    run: () => (dialogs.runnerTarget = { collectionId: c.id, folderId: null, name: c.name }),
  },
  {
    label: "Atur auth",
    icon: "lock",
    run: () => (dialogs.authTarget = { kind: "collection", id: c.id }),
  },
  { label: "Ekspor ke Postman", icon: "download", run: () => exporter.collectionToPostman(c) },
  { label: "Ganti nama", icon: "pencil", run: () => actions.rename("collection", c.id, c.name) },
  {
    label: "Hapus",
    icon: "trash",
    danger: true,
    run: () => actions.remove("collection", c.id, c.name),
  },
];
</script>

<template>
  <div v-if="collections.isEmpty" class="grid gap-3 px-3 py-4 text-xs text-muted">
    <p>Belum ada koleksi. Koleksi menyimpan request agar mudah dipakai ulang.</p>
    <div><AppButton icon="plus" @click="actions.newCollection">Buat koleksi</AppButton></div>
  </div>
  <div v-else role="tree" aria-label="Koleksi" class="grid px-1.5 pb-3">
    <template v-for="c in collections.tree.collections" :key="c.id">
      <TreeRow
        :label="c.name"
        :depth="0"
        :expanded="collections.expanded.has(c.id)"
        :menu="menu(c)"
        zones="inside"
        class="font-medium"
        @activate="collections.toggle(c.id)"
        @drop="
          (zone) =>
            actions.drop({ kind: 'collection', id: c.id, collectionId: c.id, parentId: null, zone })
        "
      >
        <template #lead>
          <AppIcon
            :name="collections.expanded.has(c.id) ? 'chevronDown' : 'chevronRight'"
            :size="12"
            class="shrink-0 text-subtle"
          />
        </template>
      </TreeRow>
      <TreeBranch
        v-if="collections.expanded.has(c.id)"
        :collection-id="c.id"
        :folder-id="null"
        :depth="1"
      />
    </template>
  </div>
</template>
