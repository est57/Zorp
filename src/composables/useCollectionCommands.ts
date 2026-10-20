import { computed } from "vue";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import type { Command } from "./useCommands";
import { useExport } from "./useExport";

/** Palette commands that act on a whole collection: run it or export it. */
export function useCollectionCommands() {
  const collections = useCollectionsStore();
  const dialogs = useDialogsStore();
  const exporter = useExport();

  return computed<Command[]>(() =>
    collections.tree.collections.flatMap((c): Command[] => [
      {
        id: `run:${c.id}`,
        group: "Aksi",
        label: `Jalankan koleksi ${c.name}`,
        run: () => (dialogs.runnerTarget = { collectionId: c.id, folderId: null, name: c.name }),
      },
      {
        id: `export:${c.id}`,
        group: "Aksi",
        label: `Ekspor koleksi ${c.name} ke Postman`,
        run: () => exporter.collectionToPostman(c),
      },
    ]),
  );
}
