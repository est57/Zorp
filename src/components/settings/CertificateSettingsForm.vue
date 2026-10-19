<script setup lang="ts">
import type { WorkspaceSettings } from "@/bindings/WorkspaceSettings";
import AppButton from "@/components/ui/AppButton.vue";
import IconButton from "@/components/ui/IconButton.vue";
import { checkboxClass, inputClass } from "@/components/ui/styles";
import { fileName, pickFile } from "@/utils/files";

const settings = defineModel<WorkspaceSettings>({ required: true });
const PEM = [{ name: "Sertifikat PEM", extensions: ["pem", "crt", "cer", "key"] }];

async function addCa() {
  const path = await pickFile(PEM);
  if (path) settings.value.caCertificates.push({ id: crypto.randomUUID(), path, enabled: true });
}

function addClient() {
  settings.value.clientCertificates.push({
    id: crypto.randomUUID(),
    host: "",
    certPath: "",
    keyPath: "",
    enabled: true,
  });
}

async function choose(index: number, field: "certPath" | "keyPath") {
  const path = await pickFile(PEM);
  if (path) settings.value.clientCertificates[index][field] = path;
}

const fileButton = [inputClass, "truncate text-left"];
</script>

<template>
  <div class="grid gap-5 text-xs">
    <section class="grid gap-2">
      <h3 class="font-medium">Sertifikat CA</h3>
      <p class="text-muted">
        Dipercaya selain sertifikat bawaan sistem. Gunakan untuk server internal dengan CA sendiri.
      </p>
      <div
        v-for="(ca, index) in settings.caCertificates"
        :key="ca.id"
        class="flex items-center gap-2"
      >
        <input
          v-model="ca.enabled"
          type="checkbox"
          :class="checkboxClass"
          aria-label="Aktifkan sertifikat CA"
        />
        <span class="min-w-0 flex-1 truncate font-mono select-text" :title="ca.path">{{
          ca.path
        }}</span>
        <IconButton
          icon="trash"
          label="Hapus sertifikat CA"
          @click="settings.caCertificates.splice(index, 1)"
        />
      </div>
      <div><AppButton icon="plus" @click="addCa">Tambah sertifikat CA</AppButton></div>
    </section>

    <section class="grid gap-2">
      <h3 class="font-medium">Sertifikat klien (mTLS)</h3>
      <p class="text-muted">
        Format PEM dengan private key tidak terenkripsi. Host bisa ditulis persis atau memakai
        wildcard seperti *.contoh.com.
      </p>
      <div
        v-for="(cert, index) in settings.clientCertificates"
        :key="cert.id"
        class="grid grid-cols-[20px_minmax(0,1fr)_minmax(0,1fr)_minmax(0,1fr)_28px] items-center gap-2 max-md:grid-cols-[20px_minmax(0,1fr)_28px]"
      >
        <input
          v-model="cert.enabled"
          type="checkbox"
          :class="checkboxClass"
          aria-label="Aktifkan sertifikat klien"
        />
        <input
          v-model="cert.host"
          placeholder="api.contoh.com"
          aria-label="Host"
          :class="[inputClass, 'font-mono']"
        />
        <button
          type="button"
          :class="fileButton"
          :title="cert.certPath"
          class="max-md:col-start-2"
          @click="choose(index, 'certPath')"
        >
          {{ cert.certPath ? fileName(cert.certPath) : "Pilih sertifikat" }}
        </button>
        <button
          type="button"
          :class="fileButton"
          :title="cert.keyPath"
          class="max-md:col-start-2"
          @click="choose(index, 'keyPath')"
        >
          {{ cert.keyPath ? fileName(cert.keyPath) : "Pilih private key" }}
        </button>
        <IconButton
          icon="trash"
          label="Hapus sertifikat klien"
          class="max-md:col-start-3 max-md:row-start-1"
          @click="settings.clientCertificates.splice(index, 1)"
        />
      </div>
      <div><AppButton icon="plus" @click="addClient">Tambah sertifikat klien</AppButton></div>
    </section>
  </div>
</template>
