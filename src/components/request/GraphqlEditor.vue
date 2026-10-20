<script setup lang="ts">
import { computed, ref, shallowRef } from "vue";
import { autocompletion } from "@codemirror/autocomplete";
import { graphql } from "cm6-graphql";
import { buildClientSchema, getIntrospectionQuery, type GraphQLSchema } from "graphql";
import type { RequestSpec } from "@/bindings/RequestSpec";
import { sendRequest } from "@/api/http";
import { toApiError } from "@/api/invoke";
import AppButton from "@/components/ui/AppButton.vue";
import CodeEditor from "@/components/ui/CodeEditor.vue";
import { cloneSpec, newId } from "@/utils/request";
import { schemaCache } from "./graphqlSchemas";

const props = defineProps<{ spec: RequestSpec & { body: { type: "graphql" } } }>();

const schema = shallowRef<GraphQLSchema | null>(schemaCache.get(props.spec.id) ?? null);
const loading = ref(false);
const message = ref(schema.value ? describe(schema.value) : "");
const failed = ref(false);

const extensions = computed(() => [graphql(schema.value ?? undefined), autocompletion()]);

function describe(s: GraphQLSchema) {
  const types = Object.keys(s.getTypeMap()).filter((n) => !n.startsWith("__")).length;
  return `Skema dimuat: ${types} tipe. Tekan Ctrl+Spasi untuk saran.`;
}

/** Sends an introspection query with the same URL, headers and auth, without adding to history. */
async function loadSchema() {
  loading.value = true;
  failed.value = false;
  const probe = cloneSpec(props.spec);
  probe.method = "POST";
  probe.body = { type: "graphql", query: getIntrospectionQuery(), variables: "" };
  probe.scripts.test = "";
  try {
    const response = await sendRequest(newId(), probe, false);
    const text = response.body.type === "text" ? response.body.text : "";
    const json = JSON.parse(text);
    if (!json.data)
      throw new Error(json.errors?.[0]?.message ?? `Server membalas status ${response.status}`);
    schema.value = buildClientSchema(json.data);
    schemaCache.set(props.spec.id, schema.value);
    message.value = describe(schema.value);
  } catch (error) {
    failed.value = true;
    const reason = error instanceof SyntaxError ? "respons bukan JSON" : toApiError(error).message;
    message.value = `Skema tidak bisa dimuat: ${reason}. Pastikan introspeksi diaktifkan di server.`;
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div class="flex flex-wrap items-center gap-2 border-t border-line px-3 py-1.5 text-xs">
      <AppButton :disabled="loading" @click="loadSchema">{{
        loading ? "Memuat skema" : "Ambil skema"
      }}</AppButton>
      <span
        class="min-w-0 truncate"
        :class="failed ? 'text-danger' : 'text-muted'"
        :title="message"
        >{{ message }}</span
      >
    </div>
    <div
      class="grid min-h-0 flex-1 grid-cols-[minmax(0,3fr)_minmax(0,2fr)] border-t border-line max-md:grid-cols-1"
    >
      <section class="flex min-h-0 flex-col">
        <h3 class="label-caps px-3 py-1">Query</h3>
        <div class="min-h-0 flex-1">
          <CodeEditor v-model="spec.body.query" :extensions="extensions" label="Query GraphQL" />
        </div>
      </section>
      <section class="flex min-h-0 flex-col border-l border-line max-md:border-t max-md:border-l-0">
        <h3 class="label-caps px-3 py-1">Variables (JSON)</h3>
        <div class="min-h-0 flex-1">
          <CodeEditor v-model="spec.body.variables" language="json" label="Variables GraphQL" />
        </div>
      </section>
    </div>
  </div>
</template>
