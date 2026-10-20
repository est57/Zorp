<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { Auth } from "@/bindings/Auth";
import type { KeyValue } from "@/bindings/KeyValue";
import type { Scripts } from "@/bindings/Scripts";
import { getCollectionVariables, setCollectionVariables, setNodeSettings } from "@/api/collections";
import { isTauri } from "@/api/invoke";
import AuthForm from "@/components/request/AuthForm.vue";
import DocsEditor from "@/components/request/DocsEditor.vue";
import ScriptEditor from "@/components/request/ScriptEditor.vue";
import AppButton from "@/components/ui/AppButton.vue";
import KeyValueTable from "@/components/ui/KeyValueTable.vue";
import ModalDialog from "@/components/ui/ModalDialog.vue";
import SegmentedTabs, { type TabItem } from "@/components/ui/SegmentedTabs.vue";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogsStore } from "@/stores/dialogs";
import { useToastsStore } from "@/stores/toasts";
import { cloneJson } from "@/utils/request";

const dialogs = useDialogsStore();
const collections = useCollectionsStore();
const toasts = useToastsStore();

const section = ref("auth");
const auth = ref<Auth>({ type: "none" });
const scripts = ref<Scripts>({ preRequest: "", test: "" });
const docs = ref<string | undefined>("");
const variables = ref<KeyValue[]>([]);

const target = computed(() => dialogs.nodeTarget);
const isFolder = computed(() => target.value?.kind === "folder");
const node = computed(() => {
  if (!target.value) return null;
  const list = isFolder.value ? collections.tree.folders : collections.tree.collections;
  return list.find((n) => n.id === target.value?.id) ?? null;
});
const collectionId = computed(() => {
  if (!target.value) return null;
  if (!isFolder.value) return target.value.id;
  return collections.tree.folders.find((f) => f.id === target.value?.id)?.collectionId ?? null;
});
const open = computed({
  get: () => node.value !== null,
  set: (v) => !v && (dialogs.nodeTarget = null),
});

const sections = computed<TabItem[]>(() => [
  { id: "auth", label: "Auth" },
  {
    id: "scripts",
    label: "Script",
    badge:
      [scripts.value.preRequest, scripts.value.test].filter((s) => s.trim()).length || undefined,
  },
  ...(isFolder.value
    ? []
    : [{ id: "variables", label: "Variabel", badge: variables.value.length || undefined }]),
  { id: "docs", label: "Docs" },
]);

watch(node, async (current, previous) => {
  if (!current || current.id === previous?.id) return;
  section.value = "auth";
  auth.value = current.auth
    ? cloneJson(current.auth)
    : { type: isFolder.value ? "inherit" : "none" };
  scripts.value = cloneJson(current.scripts);
  docs.value = current.docs;
  variables.value = [];
  if (!isFolder.value && isTauri()) {
    const pairs = await toasts.guard(() => getCollectionVariables(current.id));
    variables.value = (pairs ?? []).map(([key, value]) => ({
      key,
      value,
      enabled: true,
      description: "",
    }));
  }
});

async function save() {
  const t = target.value;
  if (!t) return;
  const ok = await toasts.guard(async () => {
    await collections.setAuth(t.kind, t.id, auth.value);
    await setNodeSettings(t.kind, t.id, scripts.value, docs.value ?? "");
    if (t.kind === "collection") {
      const pairs = variables.value
        .filter((v) => v.enabled && v.key.trim())
        .map((v): [string, string] => [v.key.trim(), v.value]);
      await setCollectionVariables(t.id, pairs);
    }
    await collections.load();
    return true;
  });
  if (!ok) return;
  dialogs.nodeTarget = null;
  toasts.push("Pengaturan disimpan", "success");
}
</script>

<template>
  <ModalDialog
    v-model:open="open"
    wide
    :title="`Pengaturan ${isFolder ? 'folder' : 'koleksi'}: ${node?.name ?? ''}`"
    description="Berlaku untuk semua folder dan request di dalamnya. Script berjalan dari koleksi, lalu folder, lalu request."
  >
    <form class="grid gap-3" @submit.prevent="save">
      <SegmentedTabs v-model="section" :items="sections" label="Bagian pengaturan" />
      <div class="h-72 min-h-0">
        <AuthForm
          v-if="section === 'auth'"
          v-model="auth"
          id-prefix="node-auth"
          :collection-id="collectionId"
          :inherit-label="isFolder ? 'Ikuti koleksi atau folder induk' : ''"
        />
        <ScriptEditor v-else-if="section === 'scripts'" :scripts="scripts" />
        <div v-else-if="section === 'variables'" class="h-full overflow-auto">
          <KeyValueTable v-model="variables" name="Variabel koleksi" />
          <p class="px-3 py-2 text-xs text-subtle">
            Dipakai dengan <code class="font-mono" v-text="'{{nama}}'" /> di semua request koleksi
            ini. Nilai environment yang namanya sama lebih diutamakan. Variabel koleksi ikut
            tersimpan di folder workspace, jadi jangan isi dengan rahasia.
          </p>
        </div>
        <DocsEditor
          v-else
          v-model="docs"
          :hint="`Catatan Markdown untuk ${isFolder ? 'folder' : 'koleksi'} ini.`"
        />
      </div>
      <div class="flex justify-end gap-2">
        <AppButton @click="open = false">Batal</AppButton>
        <AppButton type="submit" variant="primary">Simpan</AppButton>
      </div>
    </form>
  </ModalDialog>
</template>
