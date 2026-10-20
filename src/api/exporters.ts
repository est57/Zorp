import type { Environment } from "@/bindings/Environment";
import type { ExportReport } from "@/bindings/ExportReport";
import { invoke } from "./invoke";

export const exportCollection = (collectionId: string, path: string) =>
  invoke<ExportReport>("export_collection", { collectionId, path });

export const exportEnvironment = (environmentId: string, path: string) =>
  invoke<void>("export_environment", { environmentId, path });

export const importEnvironment = (path: string) =>
  invoke<Environment>("import_environment", { path });
