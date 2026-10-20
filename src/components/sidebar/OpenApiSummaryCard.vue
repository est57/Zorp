<script setup lang="ts">
import type { HttpMethod } from "@/bindings/HttpMethod";
import type { OpenApiSummary } from "@/bindings/OpenApiSummary";
import AppIcon from "@/components/ui/AppIcon.vue";
import { methodColor, methodLabel } from "@/utils/request";

defineProps<{ summary: OpenApiSummary }>();
</script>

<template>
  <div class="grid gap-2 rounded-lg border border-line bg-bg px-3 py-2.5 text-xs">
    <div class="flex items-center justify-between gap-2">
      <b class="truncate text-[13px] font-semibold">{{ summary.title }}</b>
      <span
        class="shrink-0 rounded bg-[color-mix(in_srgb,var(--success)_14%,transparent)] px-1.5 font-mono text-[10px] leading-4 font-semibold text-success"
        >{{ summary.format }}</span
      >
    </div>
    <div class="flex flex-wrap gap-x-3.5 gap-y-1 text-muted">
      <span v-if="summary.version">Versi {{ summary.version }}</span>
      <span>{{ summary.endpoints }} endpoint</span>
      <span>{{ summary.groups.length }} grup</span>
      <span v-if="summary.auth.length">Auth: {{ summary.auth.join(", ") }}</span>
    </div>
    <div class="grid gap-0.5 border-t border-line pt-1.5">
      <div
        v-for="g in summary.groups.slice(0, 5)"
        :key="g.name"
        class="flex h-6 items-center gap-1.5"
      >
        <AppIcon name="folder" :size="13" class="text-subtle" />
        <span class="truncate">{{ g.name }}</span>
        <span class="rounded-lg bg-surface-2 px-1.5 text-[10px] text-muted">{{ g.count }}</span>
      </div>
      <p v-if="summary.groups.length > 5" class="text-subtle">
        dan {{ summary.groups.length - 5 }} grup lainnya
      </p>
      <div
        v-for="s in summary.samples.slice(0, 3)"
        :key="s.method + s.name"
        class="flex h-6 items-center gap-2 pl-0.5 text-muted"
      >
        <span
          class="w-9 shrink-0 text-right font-mono text-[10px] font-semibold"
          :class="methodColor(s.method as HttpMethod)"
          >{{ methodLabel(s.method as HttpMethod) }}</span
        >
        <span class="truncate">{{ s.name }}</span>
      </div>
    </div>
  </div>
</template>
