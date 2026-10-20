<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import type { RealtimeEvent } from "@/bindings/RealtimeEvent";

const props = defineProps<{ events: RealtimeEvent[] }>();
const list = ref<HTMLElement>();

const LABELS: Record<RealtimeEvent["kind"], { text: string; tone: string }> = {
  open: { text: "INFO", tone: "text-success" },
  message: { text: "MASUK", tone: "text-[var(--info)]" },
  sent: { text: "KELUAR", tone: "text-[var(--post)]" },
  close: { text: "INFO", tone: "text-muted" },
  error: { text: "ERROR", tone: "text-danger" },
};

const time = (at: number) =>
  new Date(at).toLocaleTimeString("id-ID", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }) + `.${String(at % 1000).padStart(3, "0")}`;

// Follow new messages unless the user scrolled up to read older ones.
watch(
  () => props.events.length,
  async () => {
    const el = list.value;
    const atBottom = !el || el.scrollHeight - el.scrollTop - el.clientHeight < 40;
    await nextTick();
    if (atBottom && list.value) list.value.scrollTop = list.value.scrollHeight;
  },
);
</script>

<template>
  <p v-if="!events.length" class="p-3 text-xs text-muted">
    Belum ada pesan. Klik Hubungkan untuk membuka koneksi.
  </p>
  <ol
    v-else
    ref="list"
    class="h-full overflow-y-auto font-mono text-xs select-text"
    aria-live="polite"
  >
    <li
      v-for="(e, i) in events"
      :key="i"
      class="grid grid-cols-[96px_56px_minmax(0,1fr)] gap-2 border-b border-line px-3 py-1.5 max-sm:grid-cols-[56px_minmax(0,1fr)]"
    >
      <span class="text-subtle tabular-nums max-sm:hidden">{{ time(e.at) }}</span>
      <span class="font-semibold" :class="LABELS[e.kind].tone">{{ LABELS[e.kind].text }}</span>
      <span
        class="min-w-0 break-words whitespace-pre-wrap"
        :class="e.kind === 'error' && 'text-danger'"
      >
        <span v-if="e.event" class="mr-1.5 rounded bg-surface-2 px-1 text-[10px] text-muted">{{
          e.event
        }}</span
        >{{ e.data }}
      </span>
    </li>
  </ol>
</template>
