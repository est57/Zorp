import type { WorkspaceSettings } from "@/bindings/WorkspaceSettings";
import { invoke } from "./invoke";

export const getSettings = () => invoke<WorkspaceSettings>("get_settings");

export const saveSettings = (settings: WorkspaceSettings) =>
  invoke<WorkspaceSettings>("save_settings", { settings });
