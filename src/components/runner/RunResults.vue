<script setup lang="ts">
import { computed } from "vue";
import type { RunItemResult } from "@/bindings/RunItemResult";
import RunResultRow from "./RunResultRow.vue";

const props = defineProps<{ items: RunItemResult[]; grouped: boolean }>();

const groups = computed(() => {
  const map = new Map<number, RunItemResult[]>();
  for (const item of props.items)
    map.set(item.iteration, [...(map.get(item.iteration) ?? []), item]);
  return [...map.entries()].map(([iteration, list]) => ({
    iteration,
    list,
    label: list[0]?.iterationLabel ?? null,
    passed: list.filter((i) => i.passed).length,
  }));
});
</script>

<template>
  <ul v-if="!grouped" class="max-h-[40vh] overflow-y-auto">
    <RunResultRow v-for="item in items" :key="`${item.index}-${item.requestId}`" :item="item" />
  </ul>
  <div v-else class="grid max-h-[40vh] gap-2 overflow-y-auto">
    <details
      v-for="g in groups"
      :key="g.iteration"
      class="rounded-md border border-line px-2.5"
      :open="g.passed < g.list.length"
    >
      <summary class="flex cursor-pointer items-center gap-2 py-1.5 text-xs font-medium">
        <span
          class="rounded px-1.5 font-mono text-[10px] leading-4 font-semibold"
          :class="
            g.passed === g.list.length
              ? 'bg-[color-mix(in_srgb,var(--success)_14%,transparent)] text-success'
              : 'bg-[color-mix(in_srgb,var(--danger)_14%,transparent)] text-danger'
          "
          >{{ g.passed }}/{{ g.list.length }}</span
        >
        Iterasi {{ g.iteration }}
        <span v-if="g.label" class="truncate font-mono text-[11px] font-normal text-muted">{{
          g.label
        }}</span>
      </summary>
      <ul class="border-t border-line">
        <RunResultRow
          v-for="item in g.list"
          :key="`${item.index}-${item.requestId}`"
          :item="item"
        />
      </ul>
    </details>
  </div>
</template>
