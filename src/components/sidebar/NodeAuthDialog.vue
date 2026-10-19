<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { Auth } from "@/bindings/Auth";
import AuthForm from "@/components/request/AuthForm.vue";
import AppButton from "@/components/ui/AppButton.vue";
import ModalDialog from "@/components/ui/ModalDialog.vue";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import { useToastsStore } from "@/stores/toasts";
import { cloneJson } from "@/utils/request";

const dialogs = useDialogsStore();
const collections = useCollectionsStore();
const toasts = useToastsStore();

const auth = ref<Auth>({ type: "none" });

const node = computed(() => {
  const target = dialogs.authTarget;
  if (!target) return null;
  const list =
    target.kind === "collection" ? collections.tree.collections : collections.tree.folders;
  return list.find((n) => n.id === target.id) ?? null;
});
const isFolder = computed(() => dialogs.authTarget?.kind === "folder");
const open = computed({
  get: () => node.value !== null,
  set: (v) => !v && (dialogs.authTarget = null),
});

watch(node, (current) => {
  if (current)
    auth.value = current.auth
      ? cloneJson(current.auth)
      : { type: isFolder.value ? "inherit" : "none" };
});

async function save() {
  const target = dialogs.authTarget;
  if (!target) return;
  const ok = await toasts.guard(async () => {
    await collections.setAuth(target.kind, target.id, auth.value);
    return true;
  });
  if (!ok) return;
  dialogs.authTarget = null;
  toasts.push("Auth disimpan", "success");
}
</script>

<template>
  <ModalDialog
    v-model:open="open"
    wide
    :title="`Auth ${isFolder ? 'folder' : 'koleksi'}: ${node?.name ?? ''}`"
    description="Dipakai oleh request di dalamnya yang memilih tipe auth Ikuti folder atau koleksi."
  >
    <form class="grid gap-3" @submit.prevent="save">
      <AuthForm
        v-model="auth"
        id-prefix="node-auth"
        :inherit-label="isFolder ? 'Ikuti koleksi atau folder induk' : ''"
      />
      <div class="flex justify-end gap-2">
        <AppButton @click="open = false">Batal</AppButton>
        <AppButton type="submit" variant="primary">Simpan</AppButton>
      </div>
    </form>
  </ModalDialog>
</template>
