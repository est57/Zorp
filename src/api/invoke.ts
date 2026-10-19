import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import type { ErrorPayload } from "@/bindings/ErrorPayload";

export class ApiError extends Error {
  readonly kind: ErrorPayload["kind"] | "unavailable";
  readonly names: string[];

  constructor(kind: ApiError["kind"], message: string, names: string[] = []) {
    super(message);
    this.kind = kind;
    this.names = names;
  }
}

export const toApiError = (error: unknown): ApiError =>
  error instanceof ApiError ? error : new ApiError("internal", String(error));

function isErrorPayload(value: unknown): value is ErrorPayload {
  return typeof value === "object" && value !== null && "kind" in value && "message" in value;
}

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new ApiError("unavailable", "Backend hanya tersedia di aplikasi desktop.");
  }
  try {
    return await tauriInvoke<T>(command, args);
  } catch (error) {
    if (isErrorPayload(error)) throw new ApiError(error.kind, error.message, error.names ?? []);
    throw new ApiError("internal", String(error));
  }
}
