import { open } from "@tauri-apps/plugin-dialog";
import { useDialogsStore } from "@/stores/dialogs";
import { useTabsStore } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";
import { useWorkspacesStore } from "@/stores/workspaces";

export function useWorkspaceActions() {
  const workspaces = useWorkspacesStore();
  const dialogs = useDialogsStore();
  const tabs = useTabsStore();
  const toasts = useToastsStore();

  /** Asks before open tabs with unsaved changes are closed. */
  async function confirmLeave() {
    const dirty = tabs.tabs.filter((t) => tabs.isDirty(t)).length;
    if (dirty === 0) return true;
    return dialogs.confirm(
      "Tutup tab yang belum disimpan?",
      `${dirty} tab memiliki perubahan yang belum disimpan dan akan ditutup saat pindah workspace.`,
      "Tutup tab",
    );
  }

  async function pickFolder(title: string) {
    const path = await open({ directory: true, multiple: false, title });
    return typeof path === "string" ? path : null;
  }

  async function switchTo(id: string) {
    if (id === workspaces.activeId || !(await confirmLeave())) return;
    await toasts.guard(() => workspaces.switchTo(id));
  }

  async function create() {
    const name = await dialogs.prompt("Workspace baru", "Nama workspace", "", "Buat");
    if (!name?.trim() || !(await confirmLeave())) return;
    await toasts.guard(() => workspaces.create(name.trim()));
  }

  async function openFolder() {
    const path = await pickFolder("Buka folder workspace");
    if (!path || !(await confirmLeave())) return;
    const ok = await toasts.guard(() => workspaces.openFolder(path).then(() => true));
    if (ok) toasts.push(`Workspace ${workspaces.active?.name ?? ""} dibuka`, "success");
  }

  async function linkFolder() {
    const path = await pickFolder("Pilih folder kosong untuk workspace ini");
    if (!path) return;
    const ok = await toasts.guard(() => workspaces.linkFolder(path).then(() => true));
    if (ok) toasts.push("Workspace disimpan ke folder dan akan tersinkron otomatis", "success");
  }

  async function unlinkFolder() {
    const ok = await dialogs.confirm(
      "Berhenti sinkron folder?",
      "Data tetap ada di aplikasi dan file di folder tidak dihapus, tetapi perubahan tidak lagi disinkronkan.",
      "Berhenti sinkron",
      false,
    );
    if (ok) await toasts.guard(() => workspaces.unlinkFolder());
  }

  async function rename() {
    const current = workspaces.active;
    if (!current) return;
    const name = await dialogs.prompt("Ganti nama workspace", "Nama workspace", current.name);
    if (name?.trim() && name.trim() !== current.name) {
      await toasts.guard(() => workspaces.rename(current.id, name.trim()));
    }
  }

  async function remove() {
    const current = workspaces.active;
    if (!current || workspaces.items.length < 2) return;
    const files = current.folder ? " File di folder tidak dihapus." : "";
    const ok = await dialogs.confirm(
      `Hapus workspace ${current.name}?`,
      `Koleksi, environment, riwayat, dan cookie workspace ini dihapus dari aplikasi.${files}`,
    );
    if (ok && (await confirmLeave())) await toasts.guard(() => workspaces.remove(current.id));
  }

  return { switchTo, create, openFolder, linkFolder, unlinkFolder, rename, remove };
}
