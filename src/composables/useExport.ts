import { save } from "@tauri-apps/plugin-dialog";
import type { Collection } from "@/bindings/Collection";
import type { Environment } from "@/bindings/Environment";
import { exportCollection, exportEnvironment } from "@/api/exporters";
import { useToastsStore } from "@/stores/toasts";

const JSON_FILTER = [{ name: "JSON", extensions: ["json"] }];
const safeName = (name: string) => name.replace(/[\\/:*?"<>|]+/g, "-").trim() || "zorp";

export function useExport() {
  const toasts = useToastsStore();

  async function collectionToPostman(collection: Collection) {
    const path = await save({
      defaultPath: `${safeName(collection.name)}.postman_collection.json`,
      filters: JSON_FILTER,
    });
    if (!path) return;
    const report = await toasts.guard(() => exportCollection(collection.id, path));
    if (!report) return;
    toasts.push(`"${collection.name}" diekspor: ${report.requests} request`, "success");
    if (report.skipped.length) {
      toasts.push(
        `${report.skipped.length} request WebSocket tidak diekspor karena format Postman belum mendukungnya`,
        "info",
      );
    }
  }

  async function environmentToPostman(env: Environment) {
    const path = await save({
      defaultPath: `${safeName(env.name)}.postman_environment.json`,
      filters: JSON_FILTER,
    });
    if (!path) return;
    const ok = await toasts.guard(async () => {
      await exportEnvironment(env.id, path);
      return true;
    });
    if (!ok) return;
    const secrets = env.variables.filter((v) => v.secret).length;
    const note = secrets ? `. Nilai ${secrets} variabel rahasia sengaja dikosongkan` : "";
    toasts.push(`Environment "${env.name}" diekspor${note}`, "success");
  }

  return { collectionToPostman, environmentToPostman };
}
