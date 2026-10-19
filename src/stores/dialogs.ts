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
  /** Collection or folder whose auth is being edited. */
  const authTarget = ref<{ kind: "collection" | "folder"; id: string } | null>(null);
  /** Tab whose code snippet is shown. */
  const snippetTabId = ref<string | null>(null);
  const curlImportOpen = ref(false);
  const cookiesOpen = ref(false);
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
    authTarget,
    snippetTabId,
    curlImportOpen,
    cookiesOpen,
    palette,
    prompt,
    confirm,
    close,
  };
});
