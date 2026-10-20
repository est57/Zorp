<script setup lang="ts">
import { onMounted } from "vue";
import { getExample } from "@/api/examples";
import AppButton from "@/components/ui/AppButton.vue";
import { useExampleActions } from "@/composables/useExampleActions";
import { useCollectionsStore } from "@/stores/collections";
import { useRunnerStore } from "@/stores/runner";
import type { Tab } from "@/stores/tabs";
import { useTabsStore } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";
import { exampleResponse } from "@/utils/example";
import { specColor, specLabel } from "@/utils/request";
import { statusTone } from "@/utils/format";
import ResponsePanel from "./ResponsePanel.vue";

const props = defineProps<{ tab: Tab & { example: NonNullable<Tab["example"]> } }>();
const runner = useRunnerStore();
const tabs = useTabsStore();
const collections = useCollectionsStore();
const toasts = useToastsStore();
const actions = useExampleActions();

// Examples load lazily, including after tabs are restored on startup.
onMounted(async () => {
  if (runner.get(props.tab.id).status !== "idle") return;
  const example = await toasts.guard(() => getExample(props.tab.id));
  if (example) runner.show(props.tab.id, exampleResponse(example, props.tab.draft.url));
});

function openRequest() {
  const request = collections.tree.requests.find((r) => r.spec.id === props.tab.example.requestId);
  if (request) tabs.openSaved(request);
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col bg-surface">
    <div
      class="mx-3 mt-3 flex flex-wrap items-center gap-x-3 gap-y-2 rounded-md border border-line px-3 py-2 text-xs text-muted"
    >
      <span class="font-mono font-semibold" :class="statusTone(tab.example.status)">{{
        tab.example.status
      }}</span>
      <span class="min-w-0 flex-1">
        Contoh respons <b class="font-medium text-fg">{{ tab.example.name }}</b
        >. Hanya-baca.
      </span>
      <AppButton variant="ghost" icon="pencil" @click="actions.rename(tab.example)"
        >Ganti nama</AppButton
      >
      <AppButton variant="ghost" icon="trash" @click="actions.remove(tab.example)">Hapus</AppButton>
      <AppButton icon="send" @click="openRequest">Buka request</AppButton>
    </div>
    <div class="flex min-w-0 items-center gap-2 px-3 py-2.5 font-mono text-xs">
      <span class="shrink-0 font-semibold" :class="specColor(tab.draft)">{{
        specLabel(tab.draft)
      }}</span>
      <span class="truncate text-muted" :title="tab.draft.url">{{ tab.draft.url }}</span>
    </div>
    <div class="min-h-0 flex-1 border-t border-line">
      <ResponsePanel :run="runner.get(tab.id)" example />
    </div>
  </div>
</template>
