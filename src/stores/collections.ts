import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { Auth } from "@/bindings/Auth";
import type { CollectionTree } from "@/bindings/CollectionTree";
import type { MoveTarget } from "@/bindings/MoveTarget";
import type { NodeKind } from "@/bindings/NodeKind";
import type { RequestSpec } from "@/bindings/RequestSpec";
import * as api from "@/api/collections";
import { isTauri } from "@/api/invoke";
import { useTabsStore } from "./tabs";

const EMPTY: CollectionTree = { collections: [], folders: [], requests: [] };

export const useCollectionsStore = defineStore("collections", () => {
  const tree = ref<CollectionTree>(EMPTY);
  const expanded = ref(new Set<string>());

  const bySortOrder = (a: { sortOrder: number }, b: { sortOrder: number }) =>
    a.sortOrder - b.sortOrder;

  const foldersOf = (collectionId: string, parentId: string | null) =>
    tree.value.folders
      .filter((f) => f.collectionId === collectionId && f.parentId === parentId)
      .sort(bySortOrder);

  const requestsOf = (collectionId: string, folderId: string | null) =>
    tree.value.requests
      .filter((r) => r.collectionId === collectionId && r.folderId === folderId)
      .sort(bySortOrder);

  const isEmpty = computed(() => tree.value.collections.length === 0);

  async function load() {
    if (!isTauri()) return;
    tree.value = await api.listTree();
    useTabsStore().syncFromTree(tree.value.requests);
  }

  function toggle(id: string, open = !expanded.value.has(id)) {
    if (open) expanded.value.add(id);
    else expanded.value.delete(id);
  }

  async function createCollection(name: string) {
    const collection = await api.createCollection(name);
    toggle(collection.id, true);
    await load();
    return collection;
  }

  async function createFolder(collectionId: string, parentId: string | null, name: string) {
    const folder = await api.createFolder(collectionId, parentId, name);
    toggle(parentId ?? collectionId, true);
    await load();
    return folder;
  }

  async function saveRequest(spec: RequestSpec, collectionId: string, folderId: string | null) {
    const saved = await api.saveRequest(spec, collectionId, folderId);
    toggle(folderId ?? collectionId, true);
    await load();
    return saved;
  }

  async function rename(kind: NodeKind, id: string, name: string) {
    await api.renameNode(kind, id, name);
    await load();
  }

  async function remove(kind: NodeKind, id: string) {
    await api.deleteNode(kind, id);
    await load();
  }

  async function move(kind: NodeKind, id: string, target: MoveTarget) {
    await api.moveNode(kind, id, target);
    toggle(target.folderId ?? target.collectionId, true);
    await load();
  }

  async function setAuth(kind: NodeKind, id: string, auth: Auth) {
    await api.setNodeAuth(kind, id, auth);
    await load();
  }

  return {
    tree,
    expanded,
    isEmpty,
    foldersOf,
    requestsOf,
    load,
    toggle,
    createCollection,
    createFolder,
    saveRequest,
    rename,
    remove,
    move,
    setAuth,
  };
});
