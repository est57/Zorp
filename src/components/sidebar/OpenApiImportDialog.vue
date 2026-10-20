<script setup lang="ts">
import { open as pickFile } from "@tauri-apps/plugin-dialog";
import { computed, reactive, ref, watch } from "vue";
import type { GroupBy } from "@/bindings/GroupBy";
import type { OpenApiSummary } from "@/bindings/OpenApiSummary";
import { importOpenApi, previewOpenApi } from "@/api/importers";
import { toApiError } from "@/api/invoke";
import AppButton from "@/components/ui/AppButton.vue";
import ModalDialog from "@/components/ui/ModalDialog.vue";
import { checkboxClass, inputClass, selectClass } from "@/components/ui/styles";
import { useImport } from "@/composables/useImport";
import { useDialogsStore } from "@/stores/dialogs";
import OpenApiSummaryCard from "./OpenApiSummaryCard.vue";

const dialogs = useDialogsStore();
const { announce } = useImport();

const open = computed({
  get: () => dialogs.openApiOpen,
  set: (v) => (dialogs.openApiOpen = v),
});
const kind = ref<"file" | "url">("file");
const source = ref("");
const summary = ref<OpenApiSummary | null>(null);
const busy = ref(false);
const error = ref("");
const options = reactive({
  groupBy: "tag" as GroupBy,
  baseUrl: "",
  exampleBodies: true,
  collectionAuth: true,
});

watch(open, (isOpen) => {
  if (!isOpen) return;
  Object.assign(options, {
    groupBy: "tag",
    baseUrl: "",
    exampleBodies: true,
    collectionAuth: true,
  });
  source.value = "";
  summary.value = null;
  error.value = "";
});

async function run<T>(task: () => Promise<T>): Promise<T | undefined> {
  busy.value = true;
  error.value = "";
  try {
    return await task();
  } catch (e) {
    error.value = toApiError(e).message;
  } finally {
    busy.value = false;
  }
}

async function load() {
  if (!source.value.trim()) return;
  summary.value = null;
  const result = await run(() => previewOpenApi(source.value));
  if (!result) return;
  summary.value = result;
  options.baseUrl = result.servers[0] ?? "";
}

async function chooseFile() {
  const path = await pickFile({
    multiple: false,
    filters: [{ name: "OpenAPI / Swagger", extensions: ["json", "yaml", "yml"] }],
  });
  if (typeof path !== "string") return;
  source.value = path;
  await load();
}

async function submit() {
  if (!summary.value) return;
  const report = await run(() => importOpenApi(source.value, { ...options }));
  if (!report) return;
  open.value = false;
  await announce(report);
}

const segment = (active: boolean) =>
  `rounded px-2.5 py-1 font-medium ${active ? "bg-accent-soft text-accent" : "text-muted hover:text-fg"}`;
</script>

<template>
  <ModalDialog
    v-model:open="open"
    wide
    title="Impor OpenAPI / Swagger"
    description="Membuat koleksi dari spesifikasi OpenAPI 3.x atau Swagger 2.0 (JSON atau YAML)."
  >
    <form class="grid gap-3.5 text-xs" @submit.prevent="submit">
      <div class="grid gap-1.5">
        <div class="flex items-center justify-between">
          <span class="text-muted">Sumber</span>
          <div
            class="flex gap-0.5 rounded-md border border-line p-0.5"
            role="group"
            aria-label="Jenis sumber"
          >
            <button type="button" :class="segment(kind === 'file')" @click="kind = 'file'">
              File
            </button>
            <button type="button" :class="segment(kind === 'url')" @click="kind = 'url'">
              URL
            </button>
          </div>
        </div>
        <div v-if="kind === 'file'" class="flex gap-2">
          <span :class="[inputClass, 'flex items-center truncate font-mono']" :title="source">
            {{ source || "Belum ada file dipilih" }}
          </span>
          <AppButton icon="file" @click="chooseFile">Pilih file</AppButton>
        </div>
        <div v-else class="flex gap-2">
          <input
            id="openapi-url"
            v-model="source"
            type="url"
            placeholder="https://petstore3.swagger.io/api/v3/openapi.json"
            aria-label="URL dokumen OpenAPI"
            :class="[inputClass, 'font-mono']"
            @keydown.enter.prevent="load"
          />
          <AppButton :disabled="busy || !source.trim()" @click="load">Muat</AppButton>
        </div>
      </div>

      <p v-if="busy && !summary" class="text-muted" role="status">Membaca dokumen...</p>
      <p v-if="error" class="whitespace-pre-line text-danger" role="alert">{{ error }}</p>

      <template v-if="summary">
        <OpenApiSummaryCard :summary="summary" />
        <div class="grid grid-cols-[150px_minmax(0,1fr)] gap-3">
          <label class="grid gap-1.5">
            <span class="text-muted">Kelompokkan request</span>
            <select v-model="options.groupBy" :class="selectClass">
              <option value="tag">Berdasarkan tag</option>
              <option value="path">Berdasarkan path</option>
            </select>
          </label>
          <label class="grid gap-1.5">
            <span class="text-muted"
              >Base URL, disimpan sebagai <span class="font-mono" v-text="'{{baseUrl}}'"
            /></span>
            <input
              id="openapi-base-url"
              v-model="options.baseUrl"
              list="openapi-servers"
              :class="[inputClass, 'font-mono']"
            />
            <datalist id="openapi-servers">
              <option v-for="s in summary.servers" :key="s" :value="s" />
            </datalist>
          </label>
        </div>
        <label class="flex items-start gap-2">
          <input
            v-model="options.exampleBodies"
            type="checkbox"
            :class="[checkboxClass, 'mt-0.5']"
          />
          <span
            >Isi body contoh dari skema
            <small class="block text-[11px] text-subtle"
              >Memakai contoh atau nilai default dari spesifikasi.</small
            ></span
          >
        </label>
        <label class="flex items-start gap-2">
          <input
            v-model="options.collectionAuth"
            type="checkbox"
            :class="[checkboxClass, 'mt-0.5']"
          />
          <span
            >Pasang auth koleksi dari security
            <small class="block text-[11px] text-subtle"
              >Kredensial selalu berupa variabel seperti
              <span class="font-mono" v-text="'{{token}}'" />, tidak pernah diisi langsung.</small
            ></span
          >
        </label>
      </template>

      <!-- Kept in view while the options above scroll on short windows. -->
      <div
        class="sticky -bottom-1 flex justify-end gap-2 border-t border-line bg-surface pt-3 pb-1"
      >
        <AppButton @click="open = false">Batal</AppButton>
        <AppButton type="submit" variant="primary" :disabled="!summary || busy">
          {{ summary ? `Impor ${summary.endpoints} request` : "Impor" }}
        </AppButton>
      </div>
    </form>
  </ModalDialog>
</template>
