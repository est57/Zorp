<script setup lang="ts">
import { computed, ref } from "vue";
import AppIcon from "./AppIcon.vue";
import VariableInput from "./VariableInput.vue";

const model = defineModel<string>({ required: true });
defineProps<{ label: string; placeholder?: string; inputId?: string }>();

const revealed = ref(false);
// A value written as {{variable}} is a reference, not a secret, so it stays readable.
const literal = computed(() => model.value.trim() !== "" && !model.value.includes("{{"));
const masked = computed(() => literal.value && !revealed.value);
</script>

<template>
  <div
    class="flex h-7 min-w-0 items-center rounded border border-line bg-bg focus-within:border-accent"
  >
    <VariableInput
      v-model="model"
      :label="label"
      :placeholder="placeholder"
      :input-id="inputId"
      :masked="masked"
    />
    <span
      v-if="literal"
      class="flex shrink-0 px-1 text-warning"
      title="Nilai ditulis langsung dan ikut tersimpan bersama request. Simpan sebagai variabel rahasia di environment, lalu pakai {{nama}}."
    >
      <AppIcon name="alert" :size="13" />
      <span class="sr-only">Kredensial ditulis langsung</span>
    </span>
    <button
      type="button"
      class="grid size-6 shrink-0 place-items-center rounded text-subtle hover:text-fg disabled:opacity-40"
      :disabled="!literal"
      :aria-label="revealed ? `Sembunyikan ${label}` : `Tampilkan ${label}`"
      :aria-pressed="revealed"
      @click="revealed = !revealed"
    >
      <AppIcon :name="revealed ? 'eyeOff' : 'eye'" :size="14" />
    </button>
  </div>
</template>
