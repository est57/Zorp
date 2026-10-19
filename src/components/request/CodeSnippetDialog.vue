<script setup lang="ts">
import { computed, ref } from "vue";
import AppButton from "@/components/ui/AppButton.vue";
import CodeEditor from "@/components/ui/CodeEditor.vue";
import ModalDialog from "@/components/ui/ModalDialog.vue";
import SegmentedTabs from "@/components/ui/SegmentedTabs.vue";
import { checkboxClass } from "@/components/ui/styles";
import { useSnippets } from "@/composables/useSnippets";
import { useDialogsStore } from "@/stores/dialogs";
import { useTabsStore } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";
import { SNIPPET_TARGETS } from "@/utils/codegen";

const dialogs = useDialogsStore();
const tabs = useTabsStore();
const toasts = useToastsStore();
const snippets = useSnippets();

const target = ref("curl");
const resolve = ref(true);
const tab = computed(() => tabs.tabs.find((t) => t.id === dialogs.snippetTabId) ?? null);
const open = computed({
  get: () => tab.value !== null,
  set: (v) => !v && (dialogs.snippetTabId = null),
});
const items = SNIPPET_TARGETS.map((t) => ({ id: t.id, label: t.label }));

const code = computed({
  get: () => {
    if (!tab.value) return "";
    const req = snippets.build(
      tab.value.draft,
      tab.value.collectionId,
      tab.value.folderId,
      resolve.value,
    );
    return (SNIPPET_TARGETS.find((t) => t.id === target.value) ?? SNIPPET_TARGETS[0]).generate(req);
  },
  set: () => {},
});

async function copy() {
  try {
    await navigator.clipboard.writeText(code.value);
    toasts.push("Kode disalin ke clipboard", "success");
  } catch {
    toasts.push("Gagal menyalin ke clipboard", "danger");
  }
}
</script>

<template>
  <ModalDialog
    v-model:open="open"
    wide
    title="Kode request"
    description="Variabel rahasia tetap ditulis sebagai {{nama}} dan tidak pernah dimasukkan ke kode."
  >
    <div class="grid min-h-0 gap-3 text-xs">
      <SegmentedTabs v-model="target" :items="items" label="Bahasa" class="!px-0" />
      <div class="h-72 overflow-hidden rounded-md border border-line">
        <CodeEditor v-model="code" readonly label="Kode request" />
      </div>
      <div class="flex flex-wrap items-center justify-between gap-3">
        <label class="flex items-center gap-2 text-muted">
          <input v-model="resolve" type="checkbox" :class="checkboxClass" />
          Ganti variabel dengan nilai dari environment aktif
        </label>
        <AppButton variant="primary" icon="copy" @click="copy">Salin</AppButton>
      </div>
    </div>
  </ModalDialog>
</template>
