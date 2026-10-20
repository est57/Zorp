import { defineStore } from "pinia";
import { ref, shallowRef } from "vue";

export type DialogRequest =
  | {
      kind: "prompt";
      title: string;
      label: string;
      value: string;
      confirmText: string;
      resolve: (value: string | null) => void;
    }
  | {
      kind: "confirm";
      title: string;
      message: string;
      confirmText: string;
      danger: boolean;
      resolve: (ok: boolean) => void;
    };

export const useDialogsStore = defineStore("dialogs", () => {
  const current = shallowRef<DialogRequest | null>(null);
  /** Tab waiting for the user to choose where to save it. */
  const saveTabId = ref<string | null>(null);
  /** Collection or folder whose settings (auth, scripts, variables, docs) are open. */
  const nodeTarget = ref<{ kind: "collection" | "folder"; id: string } | null>(null);
  /** Tab whose code snippet is shown. */
  const snippetTabId = ref<string | null>(null);
  const curlImportOpen = ref(false);
  const openApiOpen = ref(false);
  /** Collection whose mock server dialog is open. */
  const mockTarget = ref<{ collectionId: string; name: string } | null>(null);
  const cookiesOpen = ref(false);
  const settingsOpen = ref(false);
  /** Environment open in the editor; null id means a new one. */
  const environmentEditor = ref<{ id: string | null } | null>(null);
  /** Collection or folder opened in the runner. */
  const runnerTarget = ref<{ collectionId: string; folderId: string | null; name: string } | null>(
    null,
  );
  /** Command palette scope; null when closed. */
  const palette = ref<"all" | "environments" | null>(null);

  function prompt(title: string, label: string, value = "", confirmText = "Simpan") {
    return new Promise<string | null>((resolve) => {
      current.value = { kind: "prompt", title, label, value, confirmText, resolve };
    });
  }

  function confirm(title: string, message: string, confirmText = "Hapus", danger = true) {
    return new Promise<boolean>((resolve) => {
      current.value = { kind: "confirm", title, message, confirmText, danger, resolve };
    });
  }

  function close() {
    current.value = null;
  }

  return {
    current,
    saveTabId,
    nodeTarget,
    snippetTabId,
    curlImportOpen,
    openApiOpen,
    mockTarget,
    cookiesOpen,
    settingsOpen,
    environmentEditor,
    runnerTarget,
    palette,
    prompt,
    confirm,
    close,
  };
});
