<script setup lang="ts">
import { ref, watch } from "vue";
import type { Environment } from "@/bindings/Environment";
import AppButton from "@/components/ui/AppButton.vue";
import IconButton from "@/components/ui/IconButton.vue";
import ModalDialog from "@/components/ui/ModalDialog.vue";
import { checkboxClass, inputClass } from "@/components/ui/styles";
import { useEnvironmentsStore } from "@/stores/environments";
import { useToastsStore } from "@/stores/toasts";
import { cloneJson } from "@/utils/request";

const props = defineProps<{ environment: Environment | null; global?: boolean }>();
const open = defineModel<boolean>("open", { required: true });
const envs = useEnvironmentsStore();
const toasts = useToastsStore();

const draft = ref<Environment>({ id: "", name: "", variables: [], isActive: false });

watch(open, (isOpen) => {
  if (!isOpen) return;
  draft.value = props.environment
    ? cloneJson(props.environment)
    : { id: crypto.randomUUID(), name: "", variables: [], isActive: false };
  if (draft.value.variables.length === 0) addRow();
});

function addRow() {
  draft.value.variables.push({ key: "", value: "", enabled: true, secret: false, hasValue: false });
}

async function save() {
  const saved = await toasts.guard(() =>
    props.global ? envs.saveGlobals(draft.value) : envs.save(draft.value),
  );
  if (!saved) return;
  open.value = false;
  toasts.push(
    props.global ? "Variabel global disimpan" : `Environment "${saved.name}" disimpan`,
    "success",
  );
}
</script>

<template>
  <ModalDialog
    v-model:open="open"
    wide
    :title="global ? 'Variabel global' : environment ? 'Edit environment' : 'Environment baru'"
    :description="
      global
        ? 'Berlaku di semua koleksi tanpa memilih environment. Koleksi, environment, dan data iterasi dengan nama sama lebih diutamakan. Nilai rahasia disimpan di keychain sistem.'
        : 'Variabel dirujuk dengan {{nama}}. Nilai rahasia disimpan di keychain sistem, bukan di database.'
    "
  >
    <form class="grid gap-3 text-xs" @submit.prevent="save">
      <label v-if="!global" class="grid gap-1.5">
        <span class="text-muted">Nama</span>
        <input
          v-model="draft.name"
          required
          maxlength="200"
          placeholder="Development"
          :class="[inputClass, '!h-8 !text-[13px]']"
        />
      </label>
      <div class="grid gap-1.5">
        <div
          class="grid grid-cols-[20px_minmax(0,1fr)_minmax(0,1.4fr)_64px_28px] gap-2 text-subtle max-sm:grid-cols-[20px_minmax(0,1fr)_minmax(0,1fr)_28px]"
        >
          <span /><span class="label-caps">Variabel</span><span class="label-caps">Nilai</span>
          <span class="label-caps max-sm:hidden">Rahasia</span><span />
        </div>
        <div
          v-for="(v, index) in draft.variables"
          :key="index"
          class="grid grid-cols-[20px_minmax(0,1fr)_minmax(0,1.4fr)_64px_28px] items-center gap-2 max-sm:grid-cols-[20px_minmax(0,1fr)_minmax(0,1fr)_28px]"
        >
          <input
            v-model="v.enabled"
            type="checkbox"
            :class="checkboxClass"
            aria-label="Aktifkan variabel"
          />
          <input
            v-model="v.key"
            placeholder="base_url"
            aria-label="Nama variabel"
            :class="[inputClass, 'font-mono']"
          />
          <input
            v-model="v.value"
            :type="v.secret ? 'password' : 'text'"
            autocomplete="off"
            :placeholder="v.secret && v.hasValue ? 'Tersimpan di keychain' : 'Nilai'"
            :title="v.secret && v.hasValue ? 'Isi untuk mengganti nilai yang tersimpan' : undefined"
            aria-label="Nilai variabel"
            :class="[inputClass, 'font-mono']"
          />
          <label class="flex items-center gap-1.5 max-sm:hidden">
            <input
              v-model="v.secret"
              type="checkbox"
              :class="checkboxClass"
              aria-label="Simpan sebagai rahasia"
            />
          </label>
          <IconButton icon="x" label="Hapus variabel" @click="draft.variables.splice(index, 1)" />
        </div>
        <div>
          <AppButton variant="ghost" icon="plus" @click="addRow">Tambah variabel</AppButton>
        </div>
      </div>
      <div class="flex justify-end gap-2">
        <AppButton @click="open = false">Batal</AppButton>
        <AppButton type="submit" variant="primary">Simpan</AppButton>
      </div>
    </form>
  </ModalDialog>
</template>
