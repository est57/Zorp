import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { RequestSpec } from "@/bindings/RequestSpec";
import type { SavedRequest } from "@/bindings/SavedRequest";
import { cloneSpec, newId, newRequest, snapshot } from "@/utils/request";
import { paramsFromUrl } from "@/utils/url";

export interface Tab {
  id: string;
  draft: RequestSpec;
  saved: string;
  collectionId: string | null;
  folderId: string | null;
}

export const useTabsStore = defineStore("tabs", () => {
  const tabs = ref<Tab[]>([]);
  const activeId = ref<string | null>(null);

  const active = computed(() => tabs.value.find((t) => t.id === activeId.value) ?? null);
  const isDirty = (tab: Tab) => snapshot(tab.draft) !== tab.saved;

  function add(source: RequestSpec, collectionId: string | null, folderId: string | null) {
    // The URL query is the source of truth for enabled params.
    const spec = { ...cloneSpec(source), params: paramsFromUrl(source.url, source.params) };
    const tab: Tab = {
      id: spec.id,
      draft: spec,
      saved: snapshot(spec),
      collectionId,
      folderId,
    };
    tabs.value.push(tab);
    activeId.value = tab.id;
  }

  function openSaved(item: SavedRequest) {
    const existing = tabs.value.find((t) => t.id === item.spec.id);
    if (existing) activeId.value = existing.id;
    else add(item.spec, item.collectionId, item.folderId);
  }

  function openNew(spec: RequestSpec = newRequest()) {
    add(spec, null, null);
  }

  /** Opens a detached copy, used for history entries and duplicates. */
  function openCopy(spec: RequestSpec) {
    const copy = { ...cloneSpec(spec), id: newId() };
    add(copy, null, null);
    // A copy has never been saved, so start it as modified.
    tabs.value[tabs.value.length - 1].saved = "";
  }

  function close(id: string) {
    const index = tabs.value.findIndex((t) => t.id === id);
    if (index < 0) return;
    tabs.value.splice(index, 1);
    if (activeId.value === id) {
      activeId.value = tabs.value[Math.min(index, tabs.value.length - 1)]?.id ?? null;
    }
  }

  function markSaved(item: SavedRequest) {
    const tab = tabs.value.find((t) => t.id === item.spec.id);
    if (!tab) return;
    tab.draft = cloneSpec(item.spec);
    tab.saved = snapshot(item.spec);
    tab.collectionId = item.collectionId;
    tab.folderId = item.folderId;
  }

  /** Keeps open tabs consistent after a request is renamed, moved or deleted in the sidebar. */
  function syncFromTree(saved: SavedRequest[]) {
    const byId = new Map(saved.map((s) => [s.spec.id, s]));
    for (const tab of tabs.value) {
      if (tab.collectionId === null) continue;
      const item = byId.get(tab.id);
      if (!item) {
        tab.collectionId = null;
        tab.folderId = null;
        tab.saved = "";
        continue;
      }
      tab.collectionId = item.collectionId;
      tab.folderId = item.folderId;
      if (tab.draft.name !== item.spec.name) {
        tab.draft.name = item.spec.name;
        if (tab.saved) tab.saved = snapshot({ ...JSON.parse(tab.saved), name: item.spec.name });
      }
    }
  }

  function reset() {
    tabs.value = [];
    activeId.value = null;
  }

  return {
    tabs,
    activeId,
    active,
    isDirty,
    openSaved,
    openNew,
    openCopy,
    close,
    markSaved,
    syncFromTree,
    reset,
  };
});
