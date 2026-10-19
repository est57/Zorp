<script setup lang="ts">
import { computed, ref, watch } from "vue";
import AppButton from "@/components/ui/AppButton.vue";
import ModalDialog from "@/components/ui/ModalDialog.vue";
import { useDialogsStore } from "@/stores/dialogs";
import { useTabsStore } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";
import { applyCurl } from "@/utils/curl/apply";
import { parseCurl } from "@/utils/curl/parse";
import { newRequest } from "@/utils/request";

const dialogs = useDialogsStore();
const tabs = useTabsStore();
const toasts = useToastsStore();

const text = ref("");
const error = ref("");
const open = computed({
  get: () => dialogs.curlImportOpen,
  set: (v) => (dialogs.curlImportOpen = v),
});

watch(open, (isOpen) => {
  if (isOpen) {
    text.value = "";
    error.value = "";
  }
});

function submit() {
  try {
    const parsed = parseCurl(text.value);
    const spec = newRequest({ name: "Request dari cURL" });
    applyCurl(spec, parsed);
    tabs.openNew(spec);
    open.value = false;
    for (const warning of parsed.warnings) toasts.push(warning, "danger");
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}
</script>

<template>
  <ModalDialog
    v-model:open="open"
    wide
    title="Impor dari cURL"
    description="Tempel perintah cURL. Hasilnya dibuka sebagai tab baru."
  >
    <form class="grid gap-3 text-xs" @submit.prevent="submit">
      <textarea
        v-model="text"
        aria-label="Perintah cURL"
        spellcheck="false"
        required
        placeholder="curl -X POST https://api.contoh.com/users -H 'Content-Type: application/json' -d '{&quot;name&quot;:&quot;Rani&quot;}'"
        class="min-h-40 w-full resize-y rounded-md border border-line bg-bg p-3 font-mono text-xs leading-5 text-fg outline-none select-text focus:border-accent"
      />
      <p v-if="error" class="text-danger" role="alert">{{ error }}</p>
      <div class="flex justify-end gap-2">
        <AppButton @click="open = false">Batal</AppButton>
        <AppButton type="submit" variant="primary">Impor</AppButton>
      </div>
    </form>
  </ModalDialog>
</template>
