<script setup lang="ts">
import { ref } from "vue";
import type { RunItemResult } from "@/bindings/RunItemResult";
import { formatDuration, statusTone } from "@/utils/format";
import { methodColor, methodLabel } from "@/utils/request";

defineProps<{ item: RunItemResult }>();
const open = ref(false);
</script>

<template>
  <li class="border-b border-line text-xs">
    <button
      type="button"
      class="flex w-full items-center gap-2 py-1.5 text-left"
      :aria-expanded="open"
      @click="open = !open"
    >
      <span
        class="w-12 shrink-0 font-mono text-[11px] font-semibold"
        :class="item.passed ? 'text-success' : 'text-danger'"
      >
        {{ item.passed ? "LULUS" : "GAGAL" }}
      </span>
      <span
        class="w-8 shrink-0 font-mono text-[10px] font-semibold"
        :class="methodColor(item.method)"
        >{{ methodLabel(item.method) }}</span
      >
      <span class="min-w-0 flex-1 truncate">{{ item.name }}</span>
      <span v-if="item.tests.length" class="shrink-0 text-subtle tabular-nums">
        {{ item.tests.filter((t) => t.passed).length }}/{{ item.tests.length }} tes
      </span>
      <span
        class="w-10 shrink-0 text-right font-mono font-semibold"
        :class="statusTone(item.status)"
        >{{ item.status ?? "-" }}</span
      >
      <span class="w-16 shrink-0 text-right font-mono text-muted tabular-nums max-sm:hidden">
        {{ item.durationMs === null ? "-" : formatDuration(item.durationMs) }}
      </span>
    </button>
    <div v-if="open" class="grid gap-1 pb-2 pl-14 select-text">
      <p v-if="item.error" class="font-mono break-words text-danger">{{ item.error }}</p>
      <p v-for="(t, i) in item.tests" :key="i" class="break-words">
        <span :class="t.passed ? 'text-success' : 'text-danger'">{{
          t.passed ? "Lulus" : "Gagal"
        }}</span>
        {{ t.name }}<span v-if="t.error" class="font-mono text-danger">: {{ t.error }}</span>
      </p>
      <p v-if="!item.error && !item.tests.length" class="text-subtle">
        Tidak ada tes untuk request ini.
      </p>
    </div>
  </li>
</template>
