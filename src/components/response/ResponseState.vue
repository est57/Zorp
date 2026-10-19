<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from "vue";
import type { RunState } from "@/stores/runner";
import { modKey } from "@/composables/usePlatform";
import { formatBytes } from "@/utils/format";

const props = defineProps<{ run: RunState }>();

const now = ref(Date.now());
const timer = setInterval(() => (now.value = Date.now()), 100);
onBeforeUnmount(() => clearInterval(timer));

const elapsed = computed(() =>
  (((props.run.finishedAt ?? now.value) - (props.run.startedAt ?? now.value)) / 1000).toFixed(1),
);
const phase = computed(() =>
  props.run.phase === "downloading"
    ? `Mengunduh response ${formatBytes(props.run.bytes ?? 0)}`
    : "Menunggu response",
);
const errorTitle = computed(() => {
  switch (props.run.error?.kind) {
    case "unresolved_variable":
      return "Variabel tidak ditemukan";
    case "invalid_url":
      return "URL tidak valid";
    case "timeout":
      return "Batas waktu habis";
    case "script":
      return "Script gagal";
    case "dns":
    case "connect":
    case "tls":
    case "network":
      return "Gagal terhubung";
    default:
      return "Request gagal";
  }
});
</script>

<template>
  <div v-if="run.status === 'loading'" class="grid gap-2 p-3" aria-live="polite">
    <p class="text-xs text-muted">
      {{ phase }} <span class="font-mono text-fg tabular-nums">{{ elapsed }} s</span>
    </p>
    <span
      v-for="w in [40, 72, 58, 66, 30]"
      :key="w"
      class="h-2.5 animate-pulse rounded bg-surface-2"
      :style="{ width: `${w}%` }"
    />
  </div>
  <div v-else-if="run.status === 'error' && run.error" class="p-3" role="alert">
    <div class="grid max-w-2xl gap-1 rounded-md border border-danger px-3.5 py-3 text-xs">
      <b class="font-mono text-[11px] tracking-wide text-danger uppercase">{{ errorTitle }}</b>
      <span class="select-text">{{ run.error.message }}</span>
      <span v-if="run.error.kind === 'unresolved_variable'" class="text-muted">
        Tambahkan variabel tersebut ke environment aktif atau pilih environment lain, lalu kirim
        ulang.
      </span>
    </div>
  </div>
  <div v-else class="grid h-full place-content-center gap-2 p-6 text-center text-muted">
    <template v-if="run.status === 'cancelled'">
      <h3 class="text-[15px] font-medium text-fg">Request dibatalkan</h3>
      <p class="text-xs">Dibatalkan setelah {{ elapsed }} s. Tidak ada response yang diterima.</p>
    </template>
    <template v-else>
      <h3 class="text-[15px] font-medium text-fg">Kirim request untuk melihat response</h3>
      <p class="text-xs">
        Tekan
        <kbd
          class="rounded border border-b-2 border-line bg-bg px-1.5 font-mono text-[11px] text-fg"
          >{{ modKey }}</kbd
        >
        <kbd
          class="rounded border border-b-2 border-line bg-bg px-1.5 font-mono text-[11px] text-fg"
          >Enter</kbd
        >
        atau tombol Kirim.
      </p>
    </template>
  </div>
</template>
