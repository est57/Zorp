<script setup lang="ts">
import { ref } from "vue";
import type { RequestSpec } from "@/bindings/RequestSpec";
import CodeEditor from "@/components/ui/CodeEditor.vue";
import SegmentedTabs from "@/components/ui/SegmentedTabs.vue";
import { PRE_REQUEST_SNIPPETS, TEST_SNIPPETS } from "./scriptSnippets";

const props = defineProps<{ spec: RequestSpec }>();

const phase = ref<"preRequest" | "test">(
  props.spec.scripts.test && !props.spec.scripts.preRequest ? "test" : "preRequest",
);
const phases = [
  { id: "preRequest", label: "Sebelum request" },
  { id: "test", label: "Tes setelah response" },
];

function insert(code: string) {
  const current = props.spec.scripts[phase.value];
  props.spec.scripts[phase.value] = current.trim()
    ? `${current.replace(/\s+$/, "")}\n\n${code}\n`
    : `${code}\n`;
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <SegmentedTabs v-model="phase" :items="phases" label="Jenis script" />
    <div class="flex min-h-0 flex-1 max-md:flex-col">
      <div class="min-h-0 min-w-0 flex-1">
        <CodeEditor
          :key="phase"
          v-model="spec.scripts[phase]"
          language="javascript"
          :label="phase === 'preRequest' ? 'Script sebelum request' : 'Script tes'"
        />
      </div>
      <aside
        class="w-52 shrink-0 overflow-y-auto border-l border-line p-2 text-xs max-md:w-full max-md:border-t max-md:border-l-0"
      >
        <p class="label-caps px-1.5 pb-1">Contoh</p>
        <button
          v-for="snippet in phase === 'preRequest' ? PRE_REQUEST_SNIPPETS : TEST_SNIPPETS"
          :key="snippet.label"
          type="button"
          class="block w-full rounded px-1.5 py-1 text-left text-accent hover:bg-surface-2"
          @click="insert(snippet.code)"
        >
          {{ snippet.label }}
        </button>
        <p class="px-1.5 pt-2 leading-[18px] text-subtle">
          Script berjalan terisolasi tanpa akses file atau jaringan, dengan batas waktu 5 detik.
        </p>
      </aside>
    </div>
  </div>
</template>
