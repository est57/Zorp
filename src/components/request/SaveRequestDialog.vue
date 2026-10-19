<script setup lang="ts">
import { computed, ref, watch } from "vue";
import AppButton from "@/components/ui/AppButton.vue";
import ModalDialog from "@/components/ui/ModalDialog.vue";
import { inputClass, selectClass } from "@/components/ui/styles";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import { useTabsStore } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";

const dialogs = useDialogsStore();
const tabs = useTabsStore();
const collections = useCollectionsStore();
const toasts = useToastsStore();

const name = ref("");
const collectionId = ref("");
const folderId = ref("");
const newCollection = ref("Koleksi Saya");

const tab = computed(() => tabs.tabs.find((t) => t.id === dialogs.saveTabId) ?? null);
const open = computed({
  get: () => tab.value !== null,
  set: (v) => !v && (dialogs.saveTabId = null),
});
const folders = computed(() =>
  collections.tree.folders.filter((f) => f.collectionId === collectionId.value),
);

watch(tab, (current) => {
  if (!current) return;
  name.value = current.draft.name;
  collectionId.value = collections.tree.collections[0]?.id ?? "";
  folderId.value = "";
});

async function submit() {
  const current = tab.value;
  if (!current) return;
  const saved = await toasts.guard(async () => {
    const target =
      collectionId.value || (await collections.createCollection(newCollection.value)).id;
    current.draft.name = name.value;
    return collections.saveRequest(current.draft, target, folderId.value || null);
  });
  if (!saved) return;
  tabs.markSaved(saved);
  dialogs.saveTabId = null;
  toasts.push("Request disimpan", "success");
}
</script>

<template>
  <ModalDialog
    v-model:open="open"
    title="Simpan request"
    description="Pilih koleksi tempat request disimpan."
  >
    <form class="grid gap-3 text-xs" @submit.prevent="submit">
      <label class="grid gap-1.5">
        <span class="text-muted">Nama request</span>
        <input v-model="name" required maxlength="200" :class="[inputClass, '!h-8 !text-[13px]']" />
      </label>
      <label v-if="collections.tree.collections.length" class="grid gap-1.5">
        <span class="text-muted">Koleksi</span>
        <select v-model="collectionId" :class="[selectClass, '!h-8']" @change="folderId = ''">
          <option v-for="c in collections.tree.collections" :key="c.id" :value="c.id">
            {{ c.name }}
          </option>
        </select>
      </label>
      <label v-else class="grid gap-1.5">
        <span class="text-muted">Nama koleksi baru</span>
        <input
          v-model="newCollection"
          required
          maxlength="200"
          :class="[inputClass, '!h-8 !text-[13px]']"
        />
      </label>
      <label v-if="folders.length" class="grid gap-1.5">
        <span class="text-muted">Folder (opsional)</span>
        <select v-model="folderId" :class="[selectClass, '!h-8']">
          <option value="">Tanpa folder</option>
          <option v-for="f in folders" :key="f.id" :value="f.id">{{ f.name }}</option>
        </select>
      </label>
      <div class="mt-1 flex justify-end gap-2">
        <AppButton @click="open = false">Batal</AppButton>
        <AppButton type="submit" variant="primary">Simpan</AppButton>
      </div>
    </form>
  </ModalDialog>
</template>
