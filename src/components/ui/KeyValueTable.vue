<script setup lang="ts">
import { computed, ref } from "vue";
import type { KeyValue } from "@/bindings/KeyValue";
import { fromBulk, toBulk } from "@/utils/bulk";
import { emptyRow } from "@/utils/request";
import IconButton from "./IconButton.vue";
import VariableInput from "./VariableInput.vue";
import { checkboxClass } from "./styles";

const props = withDefaults(
  defineProps<{ modelValue: KeyValue[]; describe?: boolean; name: string }>(),
  {
    describe: false,
  },
);
const emit = defineEmits<{ "update:modelValue": [rows: KeyValue[]] }>();

const bulk = ref(false);
const bulkText = ref("");
const rows = computed(() => [...props.modelValue, emptyRow()]);
const isDraftRow = (index: number) => index === props.modelValue.length;

function update(index: number, patch: Partial<KeyValue>) {
  const next = props.modelValue.map((r) => ({ ...r }));
  if (isDraftRow(index)) next.push({ ...emptyRow(), ...patch });
  else next[index] = { ...next[index], ...patch };
  emit("update:modelValue", next);
}

function remove(index: number) {
  emit(
    "update:modelValue",
    props.modelValue.filter((_, i) => i !== index),
  );
}

function toggleBulk() {
  if (bulk.value) emit("update:modelValue", fromBulk(bulkText.value, props.modelValue));
  else bulkText.value = toBulk(props.modelValue);
  bulk.value = !bulk.value;
}

const columns = computed(() =>
  props.describe
    ? "grid-cols-[28px_minmax(0,1fr)_minmax(0,1.4fr)_28px] @2xl:grid-cols-[28px_minmax(0,1fr)_minmax(0,1.4fr)_minmax(0,1fr)_28px]"
    : "grid-cols-[28px_minmax(0,1fr)_minmax(0,1.6fr)_28px]",
);
</script>

<template>
  <div class="@container text-xs">
    <div class="flex h-8 items-center justify-between px-3">
      <span class="label-caps">{{ name }}</span>
      <button type="button" class="text-[11px] text-subtle hover:text-fg" @click="toggleBulk">
        {{ bulk ? "Mode tabel" : "Edit massal" }}
      </button>
    </div>
    <textarea
      v-if="bulk"
      v-model="bulkText"
      :aria-label="`Edit massal ${name}`"
      spellcheck="false"
      placeholder="key: value (awali dengan // untuk menonaktifkan)"
      class="block min-h-32 w-full resize-y border-t border-line bg-bg p-3 font-mono text-xs leading-5 text-fg outline-none"
    />
    <div v-else role="table" :aria-label="name" class="border-t border-line">
      <div
        v-for="(row, index) in rows"
        :key="index"
        role="row"
        class="group grid h-8 items-stretch border-b border-line"
        :class="[columns, !row.enabled && !isDraftRow(index) && 'opacity-50']"
      >
        <div class="grid place-items-center">
          <input
            v-if="!isDraftRow(index)"
            type="checkbox"
            :class="checkboxClass"
            :checked="row.enabled"
            :aria-label="`Aktifkan ${row.key || 'baris'}`"
            @change="update(index, { enabled: !row.enabled })"
          />
        </div>
        <input
          :value="row.key"
          placeholder="Key"
          aria-label="Key"
          spellcheck="false"
          class="min-w-0 border-l border-line bg-transparent px-2 font-mono outline-none placeholder:text-subtle focus:bg-bg"
          @input="update(index, { key: ($event.target as HTMLInputElement).value })"
        />
        <div class="flex min-w-0 border-l border-line focus-within:bg-bg">
          <VariableInput
            :model-value="row.value"
            label="Value"
            placeholder="Value"
            @update:model-value="update(index, { value: $event })"
          />
        </div>
        <input
          v-if="describe"
          :value="row.description"
          placeholder="Deskripsi"
          aria-label="Deskripsi"
          class="hidden min-w-0 border-l border-line bg-transparent px-2 text-muted outline-none placeholder:text-subtle focus:bg-bg @2xl:block"
          @input="update(index, { description: ($event.target as HTMLInputElement).value })"
        />
        <div class="grid place-items-center border-l border-line">
          <IconButton
            v-if="!isDraftRow(index)"
            icon="x"
            label="Hapus baris"
            class="!size-6 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100"
            @click="remove(index)"
          />
        </div>
      </div>
    </div>
  </div>
</template>
