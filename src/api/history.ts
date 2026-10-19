import type { HistoryPage } from "@/bindings/HistoryPage";
import { invoke } from "./invoke";

export const listHistory = (cursor: number | null, limit = 50) =>
  invoke<HistoryPage>("list_history", { cursor, limit });

export const clearHistory = () => invoke<void>("clear_history");
