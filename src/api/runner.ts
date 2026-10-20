import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { RunnerEvent } from "@/bindings/RunnerEvent";
import type { RunnerOptions } from "@/bindings/RunnerOptions";
import type { RunDataPreview } from "@/bindings/RunDataPreview";
import type { RunSummary } from "@/bindings/RunSummary";
import { invoke, isTauri } from "./invoke";

export const runCollection = (
  runId: string,
  collectionId: string,
  folderId: string | null,
  options: RunnerOptions,
) => invoke<RunSummary>("run_collection", { runId, collectionId, folderId, options });

export const cancelRun = (runId: string) => invoke<void>("cancel_run", { runId });

export async function onRunnerItem(handler: (event: RunnerEvent) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen<RunnerEvent>("runner://item", (event) => handler(event.payload));
}

export const previewRunData = (path: string) =>
  invoke<RunDataPreview>("preview_run_data", { path });

export const saveRunReport = (path: string, contents: string) =>
  invoke<void>("save_run_report", { path, contents });
