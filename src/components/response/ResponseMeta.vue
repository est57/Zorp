<script setup lang="ts">
import type { ResponseData } from "@/bindings/ResponseData";
import { formatBytes, formatDuration, statusTone } from "@/utils/format";

defineProps<{ response: ResponseData }>();
</script>

<template>
  <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
    <span
      class="inline-flex h-[22px] items-center gap-1.5 rounded-full bg-surface-2 px-2 font-mono font-semibold"
      :class="statusTone(response.status)"
    >
      <span class="size-[7px] rounded-full bg-current" />
      {{ response.status }} {{ response.statusText }}
    </span>
    <span
      class="font-mono text-muted"
      :title="`Menunggu respons ${response.timing.ttfbMs} ms, unduh ${response.timing.downloadMs} ms`"
    >
      Waktu
      <b class="font-semibold text-fg tabular-nums">{{
        formatDuration(response.timing.totalMs)
      }}</b>
    </span>
    <span class="font-mono text-muted">
      Ukuran <b class="font-semibold text-fg tabular-nums">{{ formatBytes(response.sizeBytes) }}</b>
    </span>
    <span class="font-mono text-subtle max-md:hidden">{{ response.httpVersion }}</span>
  </div>
</template>
