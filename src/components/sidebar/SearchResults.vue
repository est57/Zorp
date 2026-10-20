<script setup lang="ts">
import { computed } from "vue";
import { useCollectionsStore } from "@/stores/collections";
import { useTabsStore } from "@/stores/tabs";
import { specColor, specLabel } from "@/utils/request";

const props = defineProps<{ query: string }>();
const collections = useCollectionsStore();
const tabs = useTabsStore();

const results = computed(() => {
  const q = props.query.toLowerCase();
  return collections.tree.requests.filter((r) =>
    `${r.spec.name} ${r.spec.url}`.toLowerCase().includes(q),
  );
});
</script>

<template>
  <p v-if="results.length === 0" class="px-3 py-4 text-xs text-muted">
    Tidak ada request yang cocok.
  </p>
  <div v-else class="grid px-1.5 pb-3">
    <button
      v-for="item in results"
      :key="item.spec.id"
      type="button"
      class="flex h-7 items-center gap-1.5 rounded px-1 text-left text-xs hover:bg-surface-2"
      :class="tabs.activeId === item.spec.id && 'bg-accent-soft'"
      @click="tabs.openSaved(item)"
    >
      <span
        class="w-8 shrink-0 text-right font-mono text-[10px] font-semibold"
        :class="specColor(item.spec)"
      >
        {{ specLabel(item.spec) }}
      </span>
      <span class="truncate">{{ item.spec.name }}</span>
    </button>
  </div>
</template>
