<script setup lang="ts">
import type { ExampleSummary } from "@/bindings/ExampleSummary";
import type { SavedRequest } from "@/bindings/SavedRequest";
import AppIcon from "@/components/ui/AppIcon.vue";
import { useCredentialWarnings } from "@/composables/useCredentialWarnings";
import { useExampleActions } from "@/composables/useExampleActions";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import { useTabsStore } from "@/stores/tabs";
import { specColor, specLabel } from "@/utils/request";
import TreeRow, { type MenuItem } from "./TreeRow.vue";
import { useTreeActions } from "./useTreeActions";

defineOptions({ name: "TreeBranch" });
const props = defineProps<{ collectionId: string; folderId: string | null; depth: number }>();

const collections = useCollectionsStore();
const tabs = useTabsStore();
const actions = useTreeActions();
const dialogs = useDialogsStore();
const { warningFor } = useCredentialWarnings();
const examples = useExampleActions();

const folderMenu = (id: string, name: string): MenuItem[] => [
  { label: "Request baru", icon: "plus", run: () => actions.newRequestIn(props.collectionId, id) },
  {
    label: "Folder baru",
    icon: "folderPlus",
    run: () => actions.newFolder(props.collectionId, id),
  },
  {
    label: "Jalankan",
    icon: "send",
    run: () => (dialogs.runnerTarget = { collectionId: props.collectionId, folderId: id, name }),
  },
  { label: "Atur auth", icon: "lock", run: () => (dialogs.authTarget = { kind: "folder", id }) },
  { label: "Ganti nama", icon: "pencil", run: () => actions.rename("folder", id, name) },
  { label: "Hapus", icon: "trash", danger: true, run: () => actions.remove("folder", id, name) },
];

const requestMenu = (item: SavedRequest): MenuItem[] => [
  { label: "Buka", icon: "send", run: () => tabs.openSaved(item) },
  { label: "Duplikat", icon: "copy", run: () => actions.duplicate(item) },
  { label: "Salin sebagai cURL", icon: "code", run: () => actions.copyCurl(item) },
  {
    label: "Ganti nama",
    icon: "pencil",
    run: () => actions.rename("request", item.spec.id, item.spec.name),
  },
  {
    label: "Hapus",
    icon: "trash",
    danger: true,
    run: () => actions.remove("request", item.spec.id, item.spec.name),
  },
];

const exampleMenu = (ex: ExampleSummary): MenuItem[] => [
  { label: "Buka", icon: "send", run: () => examples.open(ex) },
  { label: "Ganti nama", icon: "pencil", run: () => examples.rename(ex) },
  { label: "Hapus", icon: "trash", danger: true, run: () => examples.remove(ex) },
];

const statusChip = (status: number) =>
  status < 300
    ? "bg-[color-mix(in_srgb,var(--success)_14%,transparent)] text-success"
    : status < 400
      ? "bg-surface-2 text-muted"
      : "bg-[color-mix(in_srgb,var(--danger)_14%,transparent)] text-danger";

const drag = (kind: "folder" | "request", id: string) =>
  (actions.dragging.value = { kind, id, collectionId: props.collectionId });
</script>

<template>
  <div role="group">
    <template v-for="folder in collections.foldersOf(collectionId, folderId)" :key="folder.id">
      <TreeRow
        :label="folder.name"
        :depth="depth"
        :expanded="collections.expanded.has(folder.id)"
        :menu="folderMenu(folder.id, folder.name)"
        :warning="warningFor(folder.id)"
        draggable
        zones="all"
        @activate="collections.toggle(folder.id)"
        @dragstart="drag('folder', folder.id)"
        @drop="
          (zone) =>
            actions.drop({ kind: 'folder', id: folder.id, collectionId, parentId: folderId, zone })
        "
      >
        <template #lead>
          <AppIcon
            :name="collections.expanded.has(folder.id) ? 'chevronDown' : 'chevronRight'"
            :size="12"
            class="shrink-0 text-subtle"
          />
          <AppIcon name="folder" :size="14" class="shrink-0 text-muted" />
        </template>
      </TreeRow>
      <TreeBranch
        v-if="collections.expanded.has(folder.id)"
        :collection-id="collectionId"
        :folder-id="folder.id"
        :depth="depth + 1"
      />
    </template>
    <template v-for="item in collections.requestsOf(collectionId, folderId)" :key="item.spec.id">
      <TreeRow
        :label="item.spec.name"
        :depth="depth"
        :selected="tabs.activeId === item.spec.id"
        :menu="requestMenu(item)"
        :warning="warningFor(item.spec.id)"
        draggable
        zones="edges"
        @activate="tabs.openSaved(item)"
        @dragstart="drag('request', item.spec.id)"
        @drop="
          (zone) =>
            actions.drop({
              kind: 'request',
              id: item.spec.id,
              collectionId,
              parentId: folderId,
              zone,
            })
        "
      >
        <template #lead>
          <span
            v-if="collections.examplesOf(item.spec.id).length"
            class="grid size-3 shrink-0 place-items-center text-subtle"
            :aria-label="
              collections.expanded.has(item.spec.id) ? 'Sembunyikan contoh' : 'Tampilkan contoh'
            "
            @click.stop="collections.toggle(item.spec.id)"
          >
            <AppIcon
              :name="collections.expanded.has(item.spec.id) ? 'chevronDown' : 'chevronRight'"
              :size="12"
            />
          </span>
          <span v-else class="w-3 shrink-0" />
          <span
            class="w-8 shrink-0 text-right font-mono text-[10px] font-semibold"
            :class="specColor(item.spec)"
          >
            {{ specLabel(item.spec) }}
          </span>
        </template>
      </TreeRow>
      <template v-if="collections.expanded.has(item.spec.id)">
        <TreeRow
          v-for="ex in collections.examplesOf(item.spec.id)"
          :key="ex.id"
          :label="ex.name"
          :depth="depth + 1"
          :selected="tabs.activeId === ex.id"
          :menu="exampleMenu(ex)"
          zones="edges"
          class="text-muted"
          @activate="examples.open(ex)"
        >
          <template #lead>
            <span class="w-3 shrink-0" />
            <span
              class="shrink-0 rounded px-1 font-mono text-[10px] leading-4 font-semibold"
              :class="statusChip(ex.status)"
              >{{ ex.status }}</span
            >
          </template>
        </TreeRow>
      </template>
    </template>
  </div>
</template>
