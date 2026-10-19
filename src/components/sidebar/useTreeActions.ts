import { ref } from "vue";
import type { NodeKind } from "@/bindings/NodeKind";
import type { SavedRequest } from "@/bindings/SavedRequest";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import { useTabsStore } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";
import { newId, newRequest } from "@/utils/request";

export interface DragItem {
  kind: Exclude<NodeKind, "collection">;
  id: string;
  collectionId: string;
}

const dragging = ref<DragItem | null>(null);
const LABELS: Record<NodeKind, string> = {
  collection: "koleksi",
  folder: "folder",
  request: "request",
};

export function useTreeActions() {
  const collections = useCollectionsStore();
  const tabs = useTabsStore();
  const dialogs = useDialogsStore();
  const toasts = useToastsStore();

  async function newCollection() {
    const name = await dialogs.prompt("Koleksi baru", "Nama koleksi", "", "Buat");
    if (name) await toasts.guard(() => collections.createCollection(name));
  }

  async function newFolder(collectionId: string, parentId: string | null) {
    const name = await dialogs.prompt("Folder baru", "Nama folder", "", "Buat");
    if (name) await toasts.guard(() => collections.createFolder(collectionId, parentId, name));
  }

  async function newRequestIn(collectionId: string, folderId: string | null) {
    const saved = await toasts.guard(() =>
      collections.saveRequest(newRequest(), collectionId, folderId),
    );
    if (saved) tabs.openSaved(saved);
  }

  async function duplicate(item: SavedRequest) {
    const spec = { ...item.spec, id: newId(), name: `${item.spec.name} (salinan)` };
    const saved = await toasts.guard(() =>
      collections.saveRequest(spec, item.collectionId, item.folderId),
    );
    if (saved) tabs.openSaved(saved);
  }

  async function rename(kind: NodeKind, id: string, current: string) {
    const name = await dialogs.prompt(`Ganti nama ${LABELS[kind]}`, "Nama", current);
    if (name && name !== current) await toasts.guard(() => collections.rename(kind, id, name));
  }

  async function remove(kind: NodeKind, id: string, name: string) {
    const detail = kind === "request" ? "" : " beserta seluruh isinya";
    const ok = await dialogs.confirm(
      `Hapus ${LABELS[kind]}?`,
      `"${name}"${detail} akan dihapus permanen.`,
    );
    if (ok) await toasts.guard(() => collections.remove(kind, id));
  }

  /** Moves the dragged node into a collection or folder, after its existing children. */
  async function dropInto(collectionId: string, folderId: string | null) {
    const item = dragging.value;
    dragging.value = null;
    if (!item || item.id === folderId) return;
    if (item.kind === "folder" && item.collectionId !== collectionId) {
      toasts.push("Folder hanya bisa dipindahkan di dalam koleksi yang sama", "danger");
      return;
    }
    const siblings =
      item.kind === "folder"
        ? collections.foldersOf(collectionId, folderId)
        : collections.requestsOf(collectionId, folderId);
    const sortOrder = Math.max(0, ...siblings.map((s) => s.sortOrder)) + 1;
    await toasts.guard(() =>
      collections.move(item.kind, item.id, { collectionId, folderId, sortOrder }),
    );
  }

  return { dragging, newCollection, newFolder, newRequestIn, duplicate, rename, remove, dropInto };
}
