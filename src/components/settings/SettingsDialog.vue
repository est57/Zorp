<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { WorkspaceSettings } from "@/bindings/WorkspaceSettings";
import { getSettings, saveSettings } from "@/api/settings";
import AppButton from "@/components/ui/AppButton.vue";
import ModalDialog from "@/components/ui/ModalDialog.vue";
import SegmentedTabs from "@/components/ui/SegmentedTabs.vue";
import { useDialogsStore } from "@/stores/dialogs";
import { useToastsStore } from "@/stores/toasts";
import CertificateSettingsForm from "./CertificateSettingsForm.vue";
import ProxySettingsForm from "./ProxySettingsForm.vue";

const dialogs = useDialogsStore();
const toasts = useToastsStore();
const section = ref("proxy");
const draft = ref<WorkspaceSettings | null>(null);
const sections = [
  { id: "proxy", label: "Proxy" },
  { id: "certificates", label: "Sertifikat" },
];

const open = computed({ get: () => dialogs.settingsOpen, set: (v) => (dialogs.settingsOpen = v) });

watch(open, async (isOpen) => {
  draft.value = isOpen ? ((await toasts.guard(getSettings)) ?? null) : null;
});

async function save() {
  if (!draft.value) return;
  const saved = await toasts.guard(() => saveSettings(draft.value!));
  if (!saved) return;
  open.value = false;
  toasts.push("Pengaturan disimpan dan berlaku untuk request berikutnya", "success");
}
</script>

<template>
  <ModalDialog
    v-model:open="open"
    wide
    title="Pengaturan koneksi"
    description="Berlaku untuk semua request di workspace ini."
  >
    <form v-if="draft" class="grid gap-4" @submit.prevent="save">
      <SegmentedTabs v-model="section" :items="sections" label="Bagian pengaturan" class="!px-0" />
      <ProxySettingsForm v-if="section === 'proxy'" v-model="draft.proxy" />
      <CertificateSettingsForm v-else v-model="draft" />
      <div class="flex justify-end gap-2">
        <AppButton @click="open = false">Batal</AppButton>
        <AppButton type="submit" variant="primary">Simpan</AppButton>
      </div>
    </form>
  </ModalDialog>
</template>
