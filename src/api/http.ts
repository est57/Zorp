import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ProgressEvent } from "@/bindings/ProgressEvent";
import type { RequestSpec } from "@/bindings/RequestSpec";
import type { ResponseData } from "@/bindings/ResponseData";
import { invoke, isTauri } from "./invoke";

export const sendRequest = (requestId: string, spec: RequestSpec, recordHistory = true) =>
  invoke<ResponseData>("send_request", { requestId, spec, recordHistory });

export const cancelRequest = (requestId: string) => invoke<void>("cancel_request", { requestId });

export async function onProgress(handler: (event: ProgressEvent) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen<ProgressEvent>("request://progress", (event) => handler(event.payload));
}
