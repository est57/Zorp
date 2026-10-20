import { listen } from "@tauri-apps/api/event";
import type { SyncStatus } from "@/bindings/SyncStatus";
import type { Workspace } from "@/bindings/Workspace";
import type { WorkspaceList } from "@/bindings/WorkspaceList";
import { invoke } from "./invoke";

export const listWorkspaces = () => invoke<WorkspaceList>("list_workspaces");

export const createWorkspace = (name: string) => invoke<Workspace>("create_workspace", { name });

export const switchWorkspace = (id: string) => invoke<void>("switch_workspace", { id });

export const renameWorkspace = (id: string, name: string) =>
  invoke<void>("rename_workspace", { id, name });

export const removeWorkspace = (id: string) => invoke<void>("remove_workspace", { id });

export const syncStatus = () => invoke<SyncStatus>("sync_status");

export const openFolder = (path: string) => invoke<Workspace>("open_folder", { path });

export const linkFolder = (path: string) => invoke<void>("link_folder", { path });

export const unlinkFolder = () => invoke<void>("unlink_folder");

/** Data was read from the workspace folder, for example after `git pull`. */
export const onWorkspaceChanged = (handler: () => void) =>
  listen("workspace://changed", () => handler());

export const onSyncStatus = (handler: (status: SyncStatus) => void) =>
  listen<SyncStatus>("workspace://sync", (event) => handler(event.payload));
