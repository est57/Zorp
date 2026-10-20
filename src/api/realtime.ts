import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { RealtimeEvent } from "@/bindings/RealtimeEvent";
import type { RequestSpec } from "@/bindings/RequestSpec";
import { invoke, isTauri } from "./invoke";

export const realtimeConnect = (connectionId: string, spec: RequestSpec) =>
  invoke<void>("realtime_connect", { connectionId, spec });

export const realtimeSend = (connectionId: string, text: string) =>
  invoke<void>("realtime_send", { connectionId, text });

export const realtimeClose = (connectionId: string) =>
  invoke<void>("realtime_close", { connectionId });

export async function onRealtimeEvent(
  handler: (event: RealtimeEvent) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen<RealtimeEvent>("realtime://event", (event) => handler(event.payload));
}
