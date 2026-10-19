<script setup lang="ts">
import { computed } from "vue";
import type { Body } from "@/bindings/Body";
import type { RequestSpec } from "@/bindings/RequestSpec";
import AppButton from "@/components/ui/AppButton.vue";
import CodeEditor from "@/components/ui/CodeEditor.vue";
import KeyValueTable from "@/components/ui/KeyValueTable.vue";
import { selectClass } from "@/components/ui/styles";
import { fileName, pickFile } from "@/utils/files";
import { jsonError, prettyJson } from "@/utils/json";
import MultipartTable from "./MultipartTable.vue";

const props = defineProps<{ spec: RequestSpec }>();

const types: { type: Body["type"]; label: string }[] = [
  { type: "none", label: "Tanpa body" },
  { type: "json", label: "JSON" },
  { type: "form_url_encoded", label: "x-www-form-urlencoded" },
  { type: "multipart", label: "form-data" },
  { type: "raw", label: "Raw" },
  { type: "binary", label: "Binary" },
];
const rawTypes = [
  "text/plain",
  "application/xml",
  "text/html",
  "application/javascript",
  "text/csv",
];

const currentText = () => ("text" in props.spec.body ? props.spec.body.text : "");

function setType(type: Body["type"]) {
  const text = currentText();
  const next: Record<Body["type"], Body> = {
    none: { type: "none" },
    json: { type: "json", text },
    raw: { type: "raw", text, contentType: "text/plain" },
    form_url_encoded: { type: "form_url_encoded", fields: [] },
    multipart: { type: "multipart", parts: [] },
    binary: { type: "binary", path: "" },
  };
  props.spec.body = next[type];
}

const error = computed(() =>
  props.spec.body.type === "json" ? jsonError(props.spec.body.text) : null,
);

function beautify() {
  if (props.spec.body.type !== "json") return;
  const pretty = prettyJson(props.spec.body.text);
  if (pretty) props.spec.body.text = pretty;
}

async function chooseBinary() {
  const path = await pickFile();
  if (path && props.spec.body.type === "binary") props.spec.body.path = path;
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div class="flex flex-wrap items-center gap-2 px-3 py-2">
      <select
        :value="spec.body.type"
        aria-label="Tipe body"
        :class="selectClass"
        @change="setType(($event.target as HTMLSelectElement).value as Body['type'])"
      >
        <option v-for="t in types" :key="t.type" :value="t.type">{{ t.label }}</option>
      </select>
      <select
        v-if="spec.body.type === 'raw'"
        v-model="spec.body.contentType"
        aria-label="Content type"
        :class="selectClass"
      >
        <option v-for="ct in rawTypes" :key="ct" :value="ct">{{ ct }}</option>
      </select>
      <span v-if="error" class="min-w-0 truncate text-xs text-danger" :title="error"
        >JSON tidak valid: {{ error }}</span
      >
      <AppButton v-if="spec.body.type === 'json'" variant="ghost" class="ml-auto" @click="beautify"
        >Rapikan</AppButton
      >
    </div>

    <div class="min-h-0 flex-1 overflow-auto border-t border-line">
      <p v-if="spec.body.type === 'none'" class="p-3 text-xs text-muted">
        Request ini tidak mengirim body.
      </p>
      <CodeEditor
        v-else-if="spec.body.type === 'json' || spec.body.type === 'raw'"
        v-model="spec.body.text"
        :language="spec.body.type === 'json' ? 'json' : 'text'"
        label="Body request"
      />
      <KeyValueTable
        v-else-if="spec.body.type === 'form_url_encoded'"
        v-model="spec.body.fields"
        name="Field form"
      />
      <MultipartTable v-else-if="spec.body.type === 'multipart'" v-model="spec.body.parts" />
      <div v-else-if="spec.body.type === 'binary'" class="flex items-center gap-3 p-3 text-xs">
        <AppButton icon="file" @click="chooseBinary">Pilih file</AppButton>
        <span class="truncate text-muted" :title="spec.body.path">
          {{ spec.body.path ? fileName(spec.body.path) : "Belum ada file dipilih" }}
        </span>
      </div>
    </div>
  </div>
</template>
