<script setup lang="ts">
import { open as pickFile } from "@tauri-apps/plugin-dialog";
import { ref } from "vue";
import type { RunDataPreview } from "@/bindings/RunDataPreview";
import { previewRunData } from "@/api/runner";
import { toApiError } from "@/api/invoke";
import AppButton from "@/components/ui/AppButton.vue";
import { inputClass } from "@/components/ui/styles";

const path = defineModel<string | null>({ required: true });
defineProps<{ disabled: boolean }>();
const emit = defineEmits<{ rows: [count: number] }>();

const preview = ref<RunDataPreview | null>(null);
const error = ref("");
// Values in columns that look like credentials are hidden in the preview.
const sensitive = (column: string) => /pass|pwd|secret|token|key/i.test(column);

function useData(on: boolean) {
  if (on) return void choose();
  path.value = null;
  preview.value = null;
  error.value = "";
  emit("rows", 1);
}

async function choose() {
  const picked = await pickFile({
    multiple: false,
    filters: [{ name: "Data iterasi", extensions: ["csv", "json"] }],
  });
  if (typeof picked !== "string") return;
  error.value = "";
  try {
    preview.value = await previewRunData(picked);
    path.value = picked;
    emit("rows", preview.value.total);
  } catch (e) {
    error.value = toApiError(e).message;
  }
}

const segment = (active: boolean) =>
  `rounded px-2.5 py-0.5 font-medium ${active ? "bg-accent-soft text-accent" : "text-muted hover:text-fg"}`;
</script>

<template>
  <div class="grid gap-2">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <span class="text-muted">Data iterasi</span>
      <div
        class="flex gap-0.5 rounded-md border border-line p-0.5"
        role="group"
        aria-label="Sumber data"
      >
        <button type="button" :disabled="disabled" :class="segment(!path)" @click="useData(false)">
          Tanpa data
        </button>
        <button
          type="button"
          :disabled="disabled"
          :class="segment(Boolean(path))"
          @click="useData(true)"
        >
          File CSV / JSON
        </button>
      </div>
    </div>
    <template v-if="path && preview">
      <div class="flex gap-2">
        <span :class="[inputClass, 'flex items-center truncate font-mono']" :title="path">{{
          path
        }}</span>
        <AppButton :disabled="disabled" @click="choose">Ganti</AppButton>
      </div>
      <div class="overflow-x-auto rounded-md border border-line">
        <table class="w-full text-left text-[11px]">
          <thead class="bg-surface-2 text-subtle">
            <tr>
              <th class="px-2 py-1 font-medium">#</th>
              <th v-for="c in preview.columns" :key="c" class="px-2 py-1 font-mono font-medium">
                {{ c }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(row, i) in preview.rows" :key="i" class="border-t border-line">
              <td class="px-2 py-1 text-subtle tabular-nums">{{ i + 1 }}</td>
              <td v-for="(value, j) in row" :key="j" class="px-2 py-1 font-mono whitespace-nowrap">
                {{ sensitive(preview.columns[j]) && value ? "••••••••" : value }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <p class="text-[11px] leading-4 text-subtle">
        {{ preview.total }} iterasi{{
          preview.total > preview.rows.length
            ? `, ${preview.rows.length} baris pertama ditampilkan`
            : ""
        }}. Setiap kolom menjadi variabel, misalnya
        <code class="font-mono" v-text="`{{${preview.columns[0]}}}`" />, dan menimpa environment
        selama iterasi itu. File hanya dibaca saat runner berjalan dan tidak disimpan ke workspace.
      </p>
    </template>
    <p v-if="error" class="text-danger" role="alert">{{ error }}</p>
  </div>
</template>
