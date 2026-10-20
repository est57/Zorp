import type { ExampleSummary } from "@/bindings/ExampleSummary";
import type { ResponseData } from "@/bindings/ResponseData";
import * as api from "@/api/examples";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import type { Tab } from "@/stores/tabs";
import { useTabsStore } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";
import { exampleName } from "@/utils/example";

export function useExampleActions() {
  const collections = useCollectionsStore();
  const dialogs = useDialogsStore();
  const tabs = useTabsStore();
  const toasts = useToastsStore();

  /** Saves the response shown in a tab as an example of its stored request. */
  async function save(tab: Tab, response: ResponseData) {
    if (!tab.collectionId) {
      toasts.push("Simpan request ke koleksi dulu, lalu simpan contohnya", "danger");
      return;
    }
    if (response.body.type !== "text") {
      toasts.push("Hanya respons teks hingga 1 MB yang bisa disimpan sebagai contoh", "danger");
      return;
    }
    const name = await dialogs.prompt(
      "Simpan sebagai contoh",
      "Nama contoh",
      exampleName(response),
    );
    if (!name?.trim()) return;
    const saved = await toasts.guard(() =>
      api.saveExample({
        requestId: tab.draft.id,
        name: name.trim(),
        status: response.status,
        statusText: response.statusText,
        headers: response.headers,
        body: response.body.type === "text" ? response.body.text : "",
      }),
    );
    if (!saved) return;
    collections.toggle(tab.draft.id, true);
    await collections.load();
    toasts.push(`Contoh "${saved.name}" disimpan`, "success");
  }

  function open(example: ExampleSummary) {
    const request = collections.tree.requests.find((r) => r.spec.id === example.requestId);
    if (request) tabs.openExample(example, request);
  }

  async function rename(example: ExampleSummary) {
    const name = await dialogs.prompt("Ganti nama contoh", "Nama contoh", example.name);
    if (!name?.trim() || name.trim() === example.name) return;
    await toasts.guard(() => api.renameExample(example.id, name.trim()));
    await collections.load();
    const tab = tabs.tabs.find((t) => t.id === example.id);
    if (tab?.example) tab.example = { ...tab.example, name: name.trim() };
  }

  async function remove(example: ExampleSummary) {
    const ok = await dialogs.confirm("Hapus contoh?", `Contoh "${example.name}" akan dihapus.`);
    if (!ok) return;
    await toasts.guard(() => api.deleteExample(example.id));
    await collections.load();
  }

  return { save, open, rename, remove };
}
