import type { ImportReport } from "@/bindings/ImportReport";
import { invoke } from "./invoke";

export const importPostman = (path: string) => invoke<ImportReport>("import_postman", { path });
