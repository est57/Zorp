<script setup lang="ts">
import { computed } from "vue";
import type { HistoryItem } from "@/bindings/HistoryItem";
import AppButton from "@/components/ui/AppButton.vue";
import { useDialogsStore } from "@/stores/dialogs";
import { useHistoryStore } from "@/stores/history";
import { useTabsStore } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";
import { dayLabel, formatDuration, statusTone, timeLabel } from "@/utils/format";
import { methodColor, methodLabel } from "@/utils/request";

const props = defineProps<{ filter: string }>();
const history = useHistoryStore();
const tabs = useTabsStore();
const dialogs = useDialogsStore();
const toasts = useToastsStore();

const groups = computed(() => {
  const query = props.filter.toLowerCase();
  const result: { label: string; items: HistoryItem[] }[] = [];
  for (const item of history.items) {
    if (query && !`${item.request.method} ${item.request.url}`.toLowerCase().includes(query))
      continue;
    const label = dayLabel(item.sentAt);
    const group = result.at(-1);
    if (group?.label === label) group.items.push(item);
    else result.push({ label, items: [item] });
  }
  return result;
});

async function clear() {
  const ok = await dialogs.confirm(
    "Hapus semua riwayat?",
    "Seluruh riwayat request di workspace ini akan dihapus permanen.",
  );
  if (ok) await toasts.guard(() => history.clear());
}
</script>

<template>
  <p v-if="history.items.length === 0" class="px-3 py-4 text-xs text-muted">
    Belum ada riwayat. Request yang dikirim akan tercatat di sini.
  </p>
  <div v-else class="grid gap-3 px-1.5 pb-3">
    <section v-for="group in groups" :key="group.label" class="grid">
      <h3 class="label-caps px-1.5 py-1">{{ group.label }}</h3>
      <button
        v-for="item in group.items"
        :key="item.id"
        type="button"
        class="grid grid-cols-[32px_minmax(0,1fr)_auto] items-center gap-x-1.5 rounded px-1 py-1 text-left text-xs hover:bg-surface-2"
        :title="item.request.url"
        @click="tabs.openCopy(item.request)"
      >
        <span
          class="text-right font-mono text-[10px] font-semibold"
          :class="methodColor(item.request.method)"
        >
          {{ methodLabel(item.request.method) }}
        </span>
        <span class="truncate font-mono text-[11px]">{{ item.request.url || "(URL kosong)" }}</span>
        <span class="font-mono text-[10px] font-semibold" :class="statusTone(item.status)">
          {{ item.status ?? "Gagal" }}
        </span>
        <span />
        <span class="text-[10px] text-subtle tabular-nums">
          {{ timeLabel(item.sentAt)
          }}<template v-if="item.durationMs != null">
            · {{ formatDuration(item.durationMs) }}</template
          >
        </span>
      </button>
    </section>
    <AppButton v-if="history.nextCursor !== null" variant="ghost" @click="history.loadMore()"
      >Muat lebih banyak</AppButton
    >
    <AppButton variant="ghost" icon="trash" class="justify-self-start !text-danger" @click="clear"
      >Hapus riwayat</AppButton
    >
  </div>
</template>
