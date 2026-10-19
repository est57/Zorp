import { defineStore } from "pinia";
import { ref } from "vue";
import type { AppInfo } from "@/bindings/AppInfo";
import { getAppInfo } from "@/api/app";
import { ApiError } from "@/api/invoke";

export type BackendStatus = "loading" | "ready" | "unavailable" | "error";

export const useAppStore = defineStore("app", () => {
  const info = ref<AppInfo | null>(null);
  const status = ref<BackendStatus>("loading");
  const errorMessage = ref("");

  async function load() {
    status.value = "loading";
    try {
      info.value = await getAppInfo();
      status.value = "ready";
    } catch (error) {
      const apiError = error instanceof ApiError ? error : null;
      status.value = apiError?.kind === "unavailable" ? "unavailable" : "error";
      errorMessage.value = apiError?.message ?? String(error);
    }
  }

  return { info, status, errorMessage, load };
});
