import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import { useRunnerStore } from "@/stores/runner";
import { type Tab, useTabsStore } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";

export function useRequestActions() {
  const tabs = useTabsStore();
  const runner = useRunnerStore();
  const collections = useCollectionsStore();
  const dialogs = useDialogsStore();
  const toasts = useToastsStore();

  async function save(tab: Tab) {
    if (!tab.collectionId) {
      dialogs.saveTabId = tab.id;
      return;
    }
    const saved = await toasts.guard(() =>
      collections.saveRequest(tab.draft, tab.collectionId!, tab.folderId),
    );
    if (saved) {
      tabs.markSaved(saved);
      toasts.push("Request disimpan", "success");
    }
  }

  async function close(tab: Tab) {
    if (tabs.isDirty(tab)) {
      const ok = await dialogs.confirm(
        "Tutup tanpa menyimpan?",
        `Perubahan pada "${tab.draft.name}" akan hilang.`,
        "Tutup tanpa menyimpan",
      );
      if (!ok) return;
    }
    void runner.cancel(tab.id);
    runner.forget(tab.id);
    tabs.close(tab.id);
  }

  const send = (tab: Tab) => runner.send(tab);
  const cancel = (tab: Tab) => runner.cancel(tab.id);

  return { save, close, send, cancel };
}
