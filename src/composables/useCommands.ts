import { computed } from "vue";
import type { HttpMethod } from "@/bindings/HttpMethod";
import { useTreeActions } from "@/components/sidebar/useTreeActions";
import { modKey } from "@/composables/usePlatform";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import { useEnvironmentsStore } from "@/stores/environments";
import { useLayoutStore } from "@/stores/layout";
import { useTabsStore } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";
import { useImport } from "./useImport";
import { useRequestActions } from "./useRequestActions";

export interface Command {
  id: string;
  group: "Request" | "Aksi" | "Environment";
  label: string;
  detail?: string;
  method?: HttpMethod;
  keys?: string;
  run: () => unknown;
}

export function useCommands() {
  const tabs = useTabsStore();
  const collections = useCollectionsStore();
  const envs = useEnvironmentsStore();
  const layout = useLayoutStore();
  const dialogs = useDialogsStore();
  const toasts = useToastsStore();
  const actions = useRequestActions();
  const tree = useTreeActions();
  const { fromPostman } = useImport();

  function locationOf(collectionId: string, folderId: string | null): string {
    const names: string[] = [];
    let current = folderId;
    while (current) {
      const folder = collections.tree.folders.find((f) => f.id === current);
      if (!folder) break;
      names.unshift(folder.name);
      current = folder.parentId;
    }
    const collection = collections.tree.collections.find((c) => c.id === collectionId);
    return [collection?.name, ...names].filter(Boolean).join(" / ");
  }

  const requests = computed<Command[]>(() =>
    collections.tree.requests.map((item) => ({
      id: `request:${item.spec.id}`,
      group: "Request",
      label: item.spec.name,
      detail: locationOf(item.collectionId, item.folderId),
      method: item.spec.method,
      run: () => tabs.openSaved(item),
    })),
  );

  const environments = computed<Command[]>(() => [
    {
      id: "env:none",
      group: "Environment",
      label: "Tanpa environment",
      run: () => toasts.guard(() => envs.activate(null)),
    },
    ...envs.items.map((env): Command => ({
      id: `env:${env.id}`,
      group: "Environment",
      label: `Gunakan ${env.name}`,
      detail: env.isActive ? "Aktif" : `${env.variables.length} variabel`,
      run: () => toasts.guard(() => envs.activate(env.id)),
    })),
  ]);

  const actionsList = computed<Command[]>(() => {
    const tab = tabs.active;
    const list: Command[] = [
      {
        id: "new",
        group: "Aksi",
        label: "Request baru",
        keys: `${modKey} T`,
        run: () => tabs.openNew(),
      },
      { id: "collection", group: "Aksi", label: "Koleksi baru", run: () => tree.newCollection() },
      {
        id: "curl",
        group: "Aksi",
        label: "Impor dari cURL",
        run: () => (dialogs.curlImportOpen = true),
      },
      { id: "postman", group: "Aksi", label: "Impor koleksi Postman", run: fromPostman },
      {
        id: "history",
        group: "Aksi",
        label: "Buka riwayat",
        run: () => layout.selectView("history"),
      },
      {
        id: "envs",
        group: "Aksi",
        label: "Kelola environment",
        run: () => layout.selectView("environments"),
      },
      {
        id: "sidebar",
        group: "Aksi",
        label: "Tampilkan atau sembunyikan sidebar",
        keys: `${modKey} B`,
        run: () => layout.toggleSidebar(),
      },
      { id: "theme", group: "Aksi", label: "Ganti tema", run: () => layout.cycleTheme() },
    ];
    if (tab) {
      list.unshift(
        {
          id: "send",
          group: "Aksi",
          label: "Kirim request",
          keys: `${modKey} Enter`,
          run: () => actions.send(tab),
        },
        {
          id: "save",
          group: "Aksi",
          label: "Simpan request",
          keys: `${modKey} S`,
          run: () => actions.save(tab),
        },
        {
          id: "code",
          group: "Aksi",
          label: "Lihat kode request",
          run: () => (dialogs.snippetTabId = tab.id),
        },
        {
          id: "close",
          group: "Aksi",
          label: "Tutup tab",
          keys: `${modKey} W`,
          run: () => actions.close(tab),
        },
      );
    }
    return list;
  });

  const all = computed(() => [...actionsList.value, ...requests.value, ...environments.value]);

  return { all, environments };
}
