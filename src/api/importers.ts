import type { ImportReport } from "@/bindings/ImportReport";
import type { OpenApiOptions } from "@/bindings/OpenApiOptions";
import type { OpenApiSummary } from "@/bindings/OpenApiSummary";
import { invoke } from "./invoke";

export const importPostman = (path: string) => invoke<ImportReport>("import_postman", { path });

/** `source` is a local file path or an http(s) URL. */
export const previewOpenApi = (source: string) =>
  invoke<OpenApiSummary>("preview_openapi", { source });

export const importOpenApi = (source: string, options: OpenApiOptions) =>
  invoke<ImportReport>("import_openapi", { source, options });
