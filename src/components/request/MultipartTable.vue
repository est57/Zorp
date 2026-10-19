<script setup lang="ts">
import type { MultipartPart } from "@/bindings/MultipartPart";
import IconButton from "@/components/ui/IconButton.vue";
import AppButton from "@/components/ui/AppButton.vue";
import { checkboxClass, inputClass, selectClass } from "@/components/ui/styles";
import { fileName, pickFile } from "@/utils/files";

const parts = defineModel<MultipartPart[]>({ required: true });

function add() {
  parts.value = [...parts.value, { type: "text", key: "", value: "", enabled: true }];
}

function setKind(index: number, type: MultipartPart["type"]) {
  const { key, enabled } = parts.value[index];
  parts.value[index] =
    type === "file" ? { type, key, enabled, path: "" } : { type, key, enabled, value: "" };
}

async function choose(part: MultipartPart) {
  const path = await pickFile();
  if (path && part.type === "file") part.path = path;
}
</script>

<template>
  <div class="grid gap-1.5 p-3 text-xs">
    <div
      v-for="(part, index) in parts"
      :key="index"
      class="grid grid-cols-[20px_minmax(0,1fr)_76px_minmax(0,1.4fr)_28px] items-center gap-2"
      :class="!part.enabled && 'opacity-50'"
    >
      <input
        v-model="part.enabled"
        type="checkbox"
        :class="checkboxClass"
        aria-label="Aktifkan bagian"
      />
      <input
        v-model="part.key"
        placeholder="Key"
        aria-label="Key"
        :class="[inputClass, 'font-mono']"
      />
      <select
        :value="part.type"
        aria-label="Tipe bagian"
        :class="selectClass"
        @change="
          setKind(index, ($event.target as HTMLSelectElement).value as MultipartPart['type'])
        "
      >
        <option value="text">Teks</option>
        <option value="file">File</option>
      </select>
      <input
        v-if="part.type === 'text'"
        v-model="part.value"
        placeholder="Value"
        aria-label="Value"
        :class="[inputClass, 'font-mono']"
      />
      <button
        v-else
        type="button"
        :class="[inputClass, 'truncate text-left', !part.path && 'text-subtle']"
        :title="part.path"
        @click="choose(part)"
      >
        {{ part.path ? fileName(part.path) : "Pilih file" }}
      </button>
      <IconButton icon="x" label="Hapus bagian" @click="parts.splice(index, 1)" />
    </div>
    <div>
      <AppButton variant="ghost" icon="plus" @click="add">Tambah bagian</AppButton>
    </div>
  </div>
</template>
