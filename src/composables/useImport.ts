import { open } from "@tauri-apps/plugin-dialog";
import type { ImportReport } from "@/bindings/ImportReport";
import { importEnvironment } from "@/api/exporters";
import { importPostman } from "@/api/importers";
import { useCollectionsStore } from "@/stores/collections";
import { useEnvironmentsStore } from "@/stores/environments";
import { useToastsStore } from "@/stores/toasts";

export function useImport() {
  const collections = useCollectionsStore();
  const toasts = useToastsStore();

  async function fromPostman() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Koleksi Postman", extensions: ["json"] }],
    });
    if (typeof path !== "string") return;
    const report = await toasts.guard(() => importPostman(path));
    if (report) await announce(report);
  }

  /** Opens the imported collection and reports the result. */
  async function announce(report: ImportReport) {
    collections.toggle(report.collectionId, true);
    await collections.load();
    toasts.push(
      `"${report.collectionName}" diimpor: ${report.requests} request, ${report.folders} folder`,
      "success",
    );
    if (report.warnings.length) {
      const more = report.warnings.length > 1 ? ` (dan ${report.warnings.length - 1} lainnya)` : "";
      toasts.push(`${report.warnings[0]}${more}`, "danger");
    }
  }

  async function environmentFromPostman() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Environment Postman", extensions: ["json"] }],
    });
    if (typeof path !== "string") return;
    const env = await toasts.guard(() => importEnvironment(path));
    if (!env) return;
    await useEnvironmentsStore().load();
    toasts.push(
      `Environment "${env.name}" diimpor dengan ${env.variables.length} variabel`,
      "success",
    );
  }

  return { fromPostman, environmentFromPostman, announce };
}
