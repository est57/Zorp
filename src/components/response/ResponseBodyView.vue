<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { ResponseData } from "@/bindings/ResponseData";
import CodeEditor from "@/components/ui/CodeEditor.vue";
import { prettyJson } from "@/utils/json";

const props = defineProps<{ response: ResponseData; mode: "pretty" | "raw" }>();

const contentType = computed(
  () => props.response.headers.find(([k]) => k.toLowerCase() === "content-type")?.[1] ?? "",
);
const isJson = computed(() => contentType.value.includes("json"));

const rawText = computed(() => {
  const body = props.response.body;
  if (body.type === "text") return body.text;
  if (body.type === "too_large") return body.preview;
  return "";
});

const displayed = ref("");
watch(
  [rawText, () => props.mode, isJson],
  () => {
    displayed.value =
      props.mode === "pretty" && isJson.value
        ? (prettyJson(rawText.value) ?? rawText.value)
        : rawText.value;
  },
  { immediate: true },
);

const image = computed(() => {
  const body = props.response.body;
  return body.type === "base64" && body.mime.startsWith("image/") && !body.mime.includes("svg")
    ? `data:${body.mime};base64,${body.data}`
    : null;
});
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <p
      v-if="response.body.type === 'too_large'"
      class="border-b border-line bg-surface-2 px-3 py-2 text-xs text-warning"
    >
      Response terlalu besar untuk ditampilkan penuh. Menampilkan 1 MB pertama. File lengkap:
      <span class="font-mono break-all text-fg select-text">{{ response.body.path }}</span>
    </p>
    <div v-if="image" class="grid min-h-0 flex-1 place-items-center overflow-auto p-4">
      <img :src="image" alt="Pratinjau gambar dari response" class="max-h-full max-w-full" />
    </div>
    <p v-else-if="response.body.type === 'base64'" class="p-3 text-xs text-muted">
      Response berupa data biner ({{ response.body.mime || "tipe tidak diketahui" }}) dan tidak bisa
      ditampilkan sebagai teks.
    </p>
    <p v-else-if="displayed === ''" class="p-3 text-xs text-muted">Response tidak memiliki body.</p>
    <div v-else class="min-h-0 flex-1">
      <CodeEditor
        v-model="displayed"
        readonly
        :language="isJson ? 'json' : 'text'"
        label="Body response"
      />
    </div>
  </div>
</template>
