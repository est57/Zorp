<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import {
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import AppIcon from "@/components/ui/AppIcon.vue";
import { type Command, useCommands } from "@/composables/useCommands";
import { useDialogsStore } from "@/stores/dialogs";
import { fuzzyMatch, highlight } from "@/utils/fuzzy";
import { methodColor, methodLabel } from "@/utils/request";

const MAX_RESULTS = 50;

const dialogs = useDialogsStore();
const commands = useCommands();
const query = ref("");
const activeIndex = ref(0);
const list = ref<HTMLElement>();

const open = computed({
  get: () => dialogs.palette !== null,
  set: (v) => !v && (dialogs.palette = null),
});
const source = computed(() =>
  dialogs.palette === "environments" ? commands.environments.value : commands.all.value,
);

const results = computed(() => {
  const ranked: { command: Command; score: number; indices: number[] }[] = [];
  for (const command of source.value) {
    const label = fuzzyMatch(query.value, command.label);
    const detail = label ? null : fuzzyMatch(query.value, command.detail ?? "");
    const match = label ?? (detail ? { score: detail.score * 0.5, indices: [] } : null);
    if (match) ranked.push({ command, ...match });
  }
  if (query.value.trim()) ranked.sort((a, b) => b.score - a.score);
  return ranked.slice(0, MAX_RESULTS);
});

watch(open, (isOpen) => {
  if (isOpen) query.value = "";
});
watch([query, () => dialogs.palette], () => (activeIndex.value = 0));

function move(step: number) {
  if (!results.value.length) return;
  activeIndex.value = (activeIndex.value + step + results.value.length) % results.value.length;
  void nextTick(() =>
    list.value?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" }),
  );
}

function run(index = activeIndex.value) {
  const item = results.value[index];
  if (!item) return;
  dialogs.palette = null;
  void item.command.run();
}
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-40 bg-black/35" />
      <DialogContent
        class="fixed top-[12vh] left-1/2 z-50 flex max-h-[70vh] w-[calc(100vw-32px)] max-w-xl -translate-x-1/2 flex-col overflow-hidden rounded-[10px] border border-line bg-surface text-fg shadow-float"
      >
        <DialogTitle class="sr-only">Command palette</DialogTitle>
        <DialogDescription class="sr-only"
          >Cari request, environment, atau aksi lalu tekan Enter.</DialogDescription
        >
        <div class="flex items-center gap-2 border-b border-line px-3">
          <AppIcon name="search" class="shrink-0 text-subtle" />
          <input
            v-model="query"
            role="combobox"
            aria-expanded="true"
            aria-controls="palette-results"
            :aria-activedescendant="results.length ? `palette-item-${activeIndex}` : undefined"
            :placeholder="
              dialogs.palette === 'environments'
                ? 'Pilih environment'
                : 'Cari request, environment, atau aksi'
            "
            class="h-11 min-w-0 flex-1 bg-transparent text-[13px] outline-none placeholder:text-subtle"
            autofocus
            @keydown.down.prevent="move(1)"
            @keydown.up.prevent="move(-1)"
            @keydown.enter.prevent="run()"
          />
        </div>
        <ul
          id="palette-results"
          ref="list"
          role="listbox"
          aria-label="Hasil"
          class="min-h-0 flex-1 overflow-y-auto p-1.5"
        >
          <li v-if="!results.length" class="px-3 py-6 text-center text-xs text-muted">
            Tidak ada hasil untuk "{{ query }}".
          </li>
          <li
            v-for="(item, index) in results"
            :id="`palette-item-${index}`"
            :key="item.command.id"
            role="option"
            :aria-selected="index === activeIndex"
            class="flex h-9 cursor-default items-center gap-2.5 rounded-md px-2.5 text-xs"
            :class="index === activeIndex ? 'bg-accent-soft' : ''"
            @mousemove="activeIndex = index"
            @click="run(index)"
          >
            <span
              v-if="item.command.method"
              class="w-8 shrink-0 text-right font-mono text-[10px] font-semibold"
              :class="item.command.tag?.className ?? methodColor(item.command.method)"
              >{{ item.command.tag?.text ?? methodLabel(item.command.method) }}</span
            >
            <span class="min-w-0 truncate">
              <template v-for="(part, i) in highlight(item.command.label, item.indices)" :key="i">
                <mark v-if="part.hit" class="bg-transparent font-semibold text-accent">{{
                  part.text
                }}</mark>
                <template v-else>{{ part.text }}</template>
              </template>
            </span>
            <span v-if="item.command.detail" class="min-w-0 truncate text-subtle">{{
              item.command.detail
            }}</span>
            <span class="ml-auto flex shrink-0 items-center gap-2 pl-2">
              <kbd v-if="item.command.keys" class="font-mono text-[10px] text-subtle">{{
                item.command.keys
              }}</kbd>
              <span class="text-[10px] text-subtle max-sm:hidden">{{ item.command.group }}</span>
            </span>
          </li>
        </ul>
        <footer
          class="flex gap-4 border-t border-line px-3 py-1.5 text-[11px] text-subtle max-sm:hidden"
        >
          <span>Panah atas/bawah untuk memilih</span><span>Enter untuk menjalankan</span
          ><span>Esc untuk menutup</span>
        </footer>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
