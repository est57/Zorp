import { defineStore } from "pinia";
import { ref, shallowRef } from "vue";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { isTauri } from "@/api/invoke";
import { useToastsStore } from "./toasts";

export type UpdateStatus = "idle" | "checking" | "available" | "downloading" | "ready" | "error";

export const useUpdaterStore = defineStore("updater", () => {
  const status = ref<UpdateStatus>("idle");
  const update = shallowRef<Update | null>(null);
  const percent = ref(0);

  /** Checks GitHub Releases for a newer signed build. Silent checks only report found updates. */
  async function checkForUpdate(silent = false) {
    if (!isTauri() || status.value === "checking" || status.value === "downloading") return;
    const toasts = useToastsStore();
    status.value = "checking";
    try {
      update.value = await check();
      status.value = update.value ? "available" : "idle";
      if (!update.value && !silent) toasts.push("Zorp sudah versi terbaru", "success");
    } catch (error) {
      status.value = "idle";
      if (!silent) toasts.push(`Gagal memeriksa pembaruan: ${error}`, "danger");
    }
  }

  async function install() {
    const pending = update.value;
    if (!pending) return;
    status.value = "downloading";
    percent.value = 0;
    let total = 0;
    let received = 0;
    try {
      await pending.downloadAndInstall((event) => {
        if (event.event === "Started") total = event.data.contentLength ?? 0;
        if (event.event === "Progress") {
          received += event.data.chunkLength;
          percent.value = total ? Math.min(100, Math.round((received / total) * 100)) : 0;
        }
      });
      status.value = "ready";
    } catch (error) {
      status.value = "available";
      useToastsStore().push(`Pembaruan gagal dipasang: ${error}`, "danger");
    }
  }

  return { status, update, percent, checkForUpdate, install, restart: relaunch };
});
