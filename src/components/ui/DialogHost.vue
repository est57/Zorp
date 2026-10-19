<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useDialogsStore } from "@/stores/dialogs";
import AppButton from "./AppButton.vue";
import ModalDialog from "./ModalDialog.vue";
import { inputClass } from "./styles";

const dialogs = useDialogsStore();
const value = ref("");

const open = computed({
  get: () => dialogs.current !== null,
  set: (next) => {
    if (!next) settle(false);
  },
});

watch(
  () => dialogs.current,
  (current) => {
    if (current?.kind === "prompt") value.value = current.value;
  },
);

function settle(confirmed: boolean) {
  const current = dialogs.current;
  if (!current) return;
  dialogs.close();
  if (current.kind === "prompt")
    current.resolve(confirmed && value.value.trim() ? value.value.trim() : null);
  else current.resolve(confirmed);
}
</script>

<template>
  <ModalDialog
    v-if="dialogs.current"
    v-model:open="open"
    :title="dialogs.current.title"
    :description="dialogs.current.kind === 'confirm' ? dialogs.current.message : undefined"
  >
    <form class="grid gap-4" @submit.prevent="settle(true)">
      <label v-if="dialogs.current.kind === 'prompt'" class="grid gap-1.5 text-xs">
        <span class="text-muted">{{ dialogs.current.label }}</span>
        <input
          id="dialog-prompt"
          v-model="value"
          :class="[inputClass, '!h-8 !text-[13px]']"
          maxlength="200"
          autofocus
        />
      </label>
      <div class="flex justify-end gap-2">
        <AppButton @click="settle(false)">Batal</AppButton>
        <AppButton
          type="submit"
          :variant="
            dialogs.current.kind === 'confirm' && dialogs.current.danger ? 'danger' : 'primary'
          "
        >
          {{ dialogs.current.confirmText }}
        </AppButton>
      </div>
    </form>
  </ModalDialog>
</template>
