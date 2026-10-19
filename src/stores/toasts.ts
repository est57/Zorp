import { defineStore } from "pinia";
import { ref } from "vue";
import { toApiError } from "@/api/invoke";

export interface Toast {
  id: number;
  message: string;
  tone: "info" | "success" | "danger";
}

const DURATION_MS = 4000;

export const useToastsStore = defineStore("toasts", () => {
  const items = ref<Toast[]>([]);
  let counter = 0;

  function push(message: string, tone: Toast["tone"] = "info") {
    const id = ++counter;
    items.value.push({ id, message, tone });
    setTimeout(() => dismiss(id), DURATION_MS);
  }

  function dismiss(id: number) {
    items.value = items.value.filter((t) => t.id !== id);
  }

  /** Runs an action and reports failures as a toast. */
  async function guard<T>(action: () => Promise<T>): Promise<T | undefined> {
    try {
      return await action();
    } catch (error) {
      push(toApiError(error).message, "danger");
      return undefined;
    }
  }

  return { items, push, dismiss, guard };
});
