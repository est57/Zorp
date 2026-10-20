import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { MockLogEntry } from "@/bindings/MockLogEntry";
import type { MockRouteInfo } from "@/bindings/MockRouteInfo";
import type { MockStatus } from "@/bindings/MockStatus";
import { invoke, isTauri } from "./invoke";

export const mockRoutes = (collectionId: string) =>
  invoke<MockRouteInfo[]>("mock_routes", { collectionId });

export const listMocks = () => invoke<MockStatus[]>("list_mocks");

export const startMock = (collectionId: string, port: number | null) =>
  invoke<MockStatus>("start_mock", { collectionId, port });

export const stopMock = (collectionId: string) => invoke<void>("stop_mock", { collectionId });

export async function onMockLog(handler: (entry: MockLogEntry) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen<MockLogEntry>("mock://log", (event) => handler(event.payload));
}
