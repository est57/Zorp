<script setup lang="ts">
import { computed } from "vue";
import type { Scripts } from "@/bindings/Scripts";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";

const props = defineProps<{ requestId: string }>();
const collections = useCollectionsStore();
const dialogs = useDialogsStore();

interface Step {
  kind: "collection" | "folder";
  id: string;
  label: string;
  scripts: Scripts;
}

const hasScript = (s: Scripts) => Boolean(s.preRequest.trim() || s.test.trim());

/** Collection and folders around the request, outermost first. */
const steps = computed<Step[]>(() => {
  const request = collections.tree.requests.find((r) => r.spec.id === props.requestId);
  if (!request) return [];
  const folders: Step[] = [];
  let current = request.folderId;
  while (current) {
    const folder = collections.tree.folders.find((f) => f.id === current);
    if (!folder) break;
    folders.unshift({
      kind: "folder",
      id: folder.id,
      label: `Folder ${folder.name}`,
      scripts: folder.scripts,
    });
    current = folder.parentId;
  }
  const collection = collections.tree.collections.find((c) => c.id === request.collectionId);
  return collection
    ? [
        {
          kind: "collection",
          id: collection.id,
          label: `Koleksi ${collection.name}`,
          scripts: collection.scripts,
        },
        ...folders,
      ]
    : folders;
});
</script>

<template>
  <div
    v-if="steps.length"
    class="mx-3 mt-2 flex flex-wrap items-center gap-x-2 gap-y-1.5 rounded-md border border-line bg-bg px-2.5 py-1.5 text-xs"
  >
    <span class="label-caps mr-1">Urutan script</span>
    <template v-for="step in steps" :key="step.id">
      <button
        type="button"
        class="grid rounded-md border px-2 py-0.5 text-left leading-4 hover:bg-surface-2"
        :class="hasScript(step.scripts) ? 'border-accent text-accent' : 'border-line text-muted'"
        :title="`Buka pengaturan ${step.label}`"
        @click="dialogs.nodeTarget = { kind: step.kind, id: step.id }"
      >
        {{ step.label }}
        <small class="text-[10px] text-subtle">{{
          hasScript(step.scripts) ? "punya script" : "kosong"
        }}</small>
      </button>
      <span class="text-subtle" aria-hidden="true">›</span>
    </template>
    <span class="rounded-md bg-accent-soft px-2 py-1 leading-4 font-medium">Request ini</span>
  </div>
</template>
