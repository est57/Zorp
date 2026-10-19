import { ref } from "vue";
import type { NodeKind } from "@/bindings/NodeKind";
import type { SavedRequest } from "@/bindings/SavedRequest";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import { useTabsStore } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";
import { type DropZone, orderAtEnd, orderNextTo } from "@/utils/order";
import { newId, newRequest } from "@/utils/request";

export interface DropTarget {
  kind: NodeKind;
  id: string;
  collectionId: string;
  /** Folder containing the target row; null at collection root. */
  parentId: string | null;
  zone: DropZone;
}

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

  function siblingsOf(
    kind: DragItem["kind"],
    collectionId: string,
    folderId: string | null,
    exclude: string,
  ) {
    const list =
      kind === "folder"
        ? collections
            .foldersOf(collectionId, folderId)
            .map((f) => ({ id: f.id, sortOrder: f.sortOrder }))
        : collections
            .requestsOf(collectionId, folderId)
            .map((r) => ({ id: r.spec.id, sortOrder: r.sortOrder }));
    return list.filter((s) => s.id !== exclude).sort((x, y) => x.sortOrder - y.sortOrder);
  }

  /** Resolves where the dragged node lands relative to the row it was dropped on. */
  async function drop(target: DropTarget) {
    const item = dragging.value;
    dragging.value = null;
    if (!item || item.id === target.id) return;

    const intoTarget =
      target.kind === "collection" ||
      target.zone === "inside" ||
      (target.kind === "folder" && item.kind === "request");
    const folderId = target.kind === "collection" ? null : intoTarget ? target.id : target.parentId;
    const siblings = siblingsOf(item.kind, target.collectionId, folderId, item.id);
    const sameKind = (target.kind === "folder") === (item.kind === "folder");
    const sortOrder =
      intoTarget || !sameKind || target.zone === "inside"
        ? orderAtEnd(siblings)
        : orderNextTo(siblings, target.id, target.zone);

    await toasts.guard(() =>
      collections.move(item.kind, item.id, {
        collectionId: target.collectionId,
        folderId,
        sortOrder,
      }),
    );
  }

  return { dragging, newCollection, newFolder, newRequestIn, duplicate, rename, remove, drop };
}
