import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { SyncStatus } from "@/bindings/SyncStatus";
import type { Workspace } from "@/bindings/Workspace";
import { isTauri } from "@/api/invoke";
import * as api from "@/api/workspaces";
import { useAppStore } from "./app";
import { useCollectionsStore } from "./collections";
import { useEnvironmentsStore } from "./environments";
import { useHistoryStore } from "./history";
import { useRealtimeStore } from "./realtime";
import { useTabsStore } from "./tabs";

export const useWorkspacesStore = defineStore("workspaces", () => {
  const items = ref<Workspace[]>([]);
  const activeId = ref<string | null>(null);
  const sync = ref<SyncStatus>({ folder: null, problems: [] });
  let listening = false;

  const active = computed(() => items.value.find((w) => w.id === activeId.value) ?? null);

  async function load() {
    if (!isTauri()) return;
    const list = await api.listWorkspaces();
    items.value = list.workspaces;
    activeId.value = list.activeId;
    sync.value = await api.syncStatus();
    listen();
  }

  function listen() {
    if (listening) return;
    listening = true;
    void api.onWorkspaceChanged(() => {
      void useCollectionsStore().load();
      void useEnvironmentsStore().load();
      void load();
    });
    void api.onSyncStatus((status) => (sync.value = status));
  }

  /** Reloads everything that belongs to the active workspace. */
  async function reloadData() {
    await Promise.all([
      load(),
      useAppStore().load(),
      useCollectionsStore().load(),
      useEnvironmentsStore().load(),
      useHistoryStore().load(),
    ]);
  }

  /** Closes all tabs and live connections, which belong to the workspace being left. */
  async function leave() {
    const tabs = useTabsStore();
    const realtime = useRealtimeStore();
    await Promise.all(tabs.tabs.map((t) => realtime.disconnect(t.id).catch(() => undefined)));
    tabs.tabs.forEach((t) => realtime.clear(t.id));
    tabs.reset();
  }

  async function switchTo(id: string) {
    if (id === activeId.value) return;
    await api.switchWorkspace(id);
    await leave();
    await reloadData();
  }

  async function create(name: string) {
    await api.createWorkspace(name);
    await leave();
    await reloadData();
  }

  async function openFolder(path: string) {
    const workspace = await api.openFolder(path);
    if (workspace.id !== activeId.value) await leave();
    await reloadData();
  }

  async function linkFolder(path: string) {
    await api.linkFolder(path);
    await load();
  }

  async function unlinkFolder() {
    await api.unlinkFolder();
    await load();
  }

  /** Reconnects to the folder and reads it again, keeping open tabs. */
  async function retrySync() {
    if (!activeId.value) return;
    await api.switchWorkspace(activeId.value);
    await reloadData();
  }

  async function rename(id: string, name: string) {
    await api.renameWorkspace(id, name);
    await Promise.all([load(), useAppStore().load()]);
  }

  /** Removes a workspace; the active one is left first for another workspace. */
  async function remove(id: string) {
    if (id === activeId.value) {
      const next = items.value.find((w) => w.id !== id);
      if (!next) return;
      await switchTo(next.id);
    }
    await api.removeWorkspace(id);
    await load();
  }

  return {
    items,
    activeId,
    active,
    sync,
    load,
    switchTo,
    create,
    openFolder,
    linkFolder,
    unlinkFolder,
    retrySync,
    rename,
    remove,
  };
});
