<script setup lang="ts">
import { computed, ref } from "vue";
import CodeEditor from "@/components/ui/CodeEditor.vue";
import MarkdownView from "@/components/ui/MarkdownView.vue";
import { useMediaQuery } from "@/composables/useMediaQuery";

const model = defineModel<string | undefined>({ required: true });
withDefaults(defineProps<{ hint?: string }>(), {
  hint: "Catatan Markdown untuk request ini, ikut tersimpan di koleksi.",
});

const wide = useMediaQuery("(min-width: 1100px)");
const view = ref<"edit" | "preview">(model.value ? "preview" : "edit");
const docs = computed({
  get: () => model.value ?? "",
  set: (value: string) => (model.value = value),
});

const segment = (active: boolean) =>
  `rounded px-2.5 py-0.5 text-xs font-medium ${active ? "bg-accent-soft text-accent" : "text-muted hover:text-fg"}`;
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div class="flex items-center justify-between gap-3 px-3 py-2">
      <span class="text-xs text-subtle"
        >Catatan Markdown untuk request ini, ikut tersimpan di koleksi.</span
      >
      <div
        v-if="!wide"
        class="flex gap-0.5 rounded-md border border-line p-0.5"
        role="group"
        aria-label="Tampilan docs"
      >
        <button type="button" :class="segment(view === 'edit')" @click="view = 'edit'">Edit</button>
        <button type="button" :class="segment(view === 'preview')" @click="view = 'preview'">
          Pratinjau
        </button>
      </div>
    </div>
    <div class="grid min-h-0 flex-1 border-t border-line" :class="wide && 'grid-cols-2'">
      <div v-if="wide || view === 'edit'" class="min-h-0 overflow-auto">
        <CodeEditor v-model="docs" label="Dokumentasi request" />
      </div>
      <div
        v-if="wide || view === 'preview'"
        class="min-h-0 overflow-auto px-4 py-3"
        :class="wide && 'border-l border-line'"
      >
        <MarkdownView v-if="docs.trim()" :source="docs" />
        <p v-else class="text-xs text-subtle">
          Belum ada dokumentasi. Tulis tujuan request, field penting, dan contoh pemakaiannya.
        </p>
      </div>
    </div>
  </div>
</template>
