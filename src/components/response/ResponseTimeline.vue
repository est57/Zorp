<script setup lang="ts">
import { computed } from "vue";
import type { Timing } from "@/bindings/Timing";
import { formatDuration } from "@/utils/format";

const props = defineProps<{ timing: Timing }>();

const phases = computed(() => {
  const t = props.timing;
  const dns = t.dnsMs ?? 0;
  const connect = t.connectMs ?? 0;
  const rows = [
    { label: "DNS", ms: t.dnsMs, tone: "bg-[var(--info)]" },
    { label: "Koneksi (TCP + TLS)", ms: t.connectMs, tone: "bg-[var(--patch)]" },
    {
      label: "Menunggu server",
      ms: Math.max(0, t.ttfbMs - dns - connect),
      tone: "bg-[var(--post)]",
    },
    { label: "Unduh", ms: t.downloadMs, tone: "bg-[var(--get)]" },
  ];
  const total = Math.max(1, t.totalMs);
  let offset = 0;
  return rows.map((row) => {
    const start = offset;
    offset += row.ms ?? 0;
    return { ...row, left: (start / total) * 100, width: ((row.ms ?? 0) / total) * 100 };
  });
});

const reused = computed(() => props.timing.dnsMs === null && props.timing.connectMs === null);
</script>

<template>
  <div class="grid max-w-3xl gap-2 p-3 text-xs">
    <div
      v-for="row in phases"
      :key="row.label"
      class="grid grid-cols-[150px_minmax(0,1fr)_72px] items-center gap-3 max-sm:grid-cols-[110px_minmax(0,1fr)_64px]"
    >
      <span class="text-muted">{{ row.label }}</span>
      <div class="relative h-2 rounded-sm bg-surface-2">
        <span
          v-if="row.ms"
          class="absolute inset-y-0 min-w-0.5 rounded-sm"
          :class="row.tone"
          :style="{ left: `${row.left}%`, width: `${row.width}%` }"
        />
      </div>
      <span class="text-right font-mono tabular-nums">{{
        row.ms === null ? "-" : formatDuration(row.ms)
      }}</span>
    </div>
    <div
      class="grid grid-cols-[150px_minmax(0,1fr)_72px] gap-3 border-t border-line pt-2 font-medium max-sm:grid-cols-[110px_minmax(0,1fr)_64px]"
    >
      <span>Total</span><span /><span class="text-right font-mono tabular-nums">{{
        formatDuration(timing.totalMs)
      }}</span>
    </div>
    <p v-if="reused" class="text-subtle">
      Koneksi sebelumnya dipakai ulang, sehingga tidak ada waktu DNS dan koneksi.
    </p>
  </div>
</template>
