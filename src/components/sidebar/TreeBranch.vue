<script setup lang="ts">
import type { SavedRequest } from "@/bindings/SavedRequest";
import AppIcon from "@/components/ui/AppIcon.vue";
import { useCollectionsStore } from "@/stores/collections";
import { useTabsStore } from "@/stores/tabs";
import { methodColor, methodLabel } from "@/utils/request";
import TreeRow, { type MenuItem } from "./TreeRow.vue";
import { useTreeActions } from "./useTreeActions";

defineOptions({ name: "TreeBranch" });
const props = defineProps<{ collectionId: string; folderId: string | null; depth: number }>();

const collections = useCollectionsStore();
const tabs = useTabsStore();
const actions = useTreeActions();

const folderMenu = (id: string, name: string): MenuItem[] => [
  { label: "Request baru", icon: "plus", run: () => actions.newRequestIn(props.collectionId, id) },
  {
    label: "Folder baru",
    icon: "folderPlus",
    run: () => actions.newFolder(props.collectionId, id),
  },
  { label: "Ganti nama", icon: "pencil", run: () => actions.rename("folder", id, name) },
  { label: "Hapus", icon: "trash", danger: true, run: () => actions.remove("folder", id, name) },
];

const requestMenu = (item: SavedRequest): MenuItem[] => [
  { label: "Buka", icon: "send", run: () => tabs.openSaved(item) },
  { label: "Duplikat", icon: "copy", run: () => actions.duplicate(item) },
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
        draggable
        droppable
        @activate="collections.toggle(folder.id)"
        @dragstart="drag('folder', folder.id)"
        @drop="actions.dropInto(collectionId, folder.id)"
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
    <TreeRow
      v-for="item in collections.requestsOf(collectionId, folderId)"
      :key="item.spec.id"
      :label="item.spec.name"
      :depth="depth"
      :selected="tabs.activeId === item.spec.id"
      :menu="requestMenu(item)"
      draggable
      @activate="tabs.openSaved(item)"
      @dragstart="drag('request', item.spec.id)"
    >
      <template #lead>
        <span
          class="w-8 shrink-0 text-right font-mono text-[10px] font-semibold"
          :class="methodColor(item.spec.method)"
        >
          {{ methodLabel(item.spec.method) }}
        </span>
      </template>
    </TreeRow>
  </div>
</template>
