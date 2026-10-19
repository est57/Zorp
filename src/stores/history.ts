import { defineStore } from "pinia";
import { ref } from "vue";
import type { HistoryItem } from "@/bindings/HistoryItem";
import { clearHistory, listHistory } from "@/api/history";
import { isTauri } from "@/api/invoke";

export const useHistoryStore = defineStore("history", () => {
  const items = ref<HistoryItem[]>([]);
  const nextCursor = ref<number | null>(null);
  const loading = ref(false);

  async function load() {
    if (!isTauri()) return;
    const page = await listHistory(null);
    items.value = page.items;
    nextCursor.value = page.nextCursor;
  }

  async function loadMore() {
    if (nextCursor.value === null || loading.value) return;
    loading.value = true;
    try {
      const page = await listHistory(nextCursor.value);
      items.value.push(...page.items);
      nextCursor.value = page.nextCursor;
    } finally {
      loading.value = false;
    }
  }

  async function clear() {
    await clearHistory();
    items.value = [];
    nextCursor.value = null;
  }

  return { items, nextCursor, loading, load, loadMore, clear };
});
