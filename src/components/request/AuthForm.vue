<script setup lang="ts">
import type { Auth } from "@/bindings/Auth";
import VariableInput from "@/components/ui/VariableInput.vue";
import { inputClass, selectClass } from "@/components/ui/styles";

const props = withDefaults(defineProps<{ inheritLabel?: string; idPrefix?: string }>(), {
  inheritLabel: "",
  idPrefix: "auth",
});
const auth = defineModel<Auth>({ required: true });

const options: { type: Auth["type"]; label: string }[] = [
  ...(props.inheritLabel ? [{ type: "inherit" as const, label: props.inheritLabel }] : []),
  { type: "none", label: "Tanpa auth" },
  { type: "bearer", label: "Bearer token" },
  { type: "basic", label: "Basic auth" },
  { type: "api_key", label: "API key" },
];

function setType(type: Auth["type"]) {
  const defaults: Record<Auth["type"], Auth> = {
    inherit: { type: "inherit" },
    none: { type: "none" },
    bearer: { type: "bearer", token: "" },
    basic: { type: "basic", username: "", password: "" },
    api_key: { type: "api_key", key: "X-API-Key", value: "", location: "header" },
  };
  auth.value = defaults[type];
}

const field = "flex h-7 min-w-0 rounded border border-line bg-bg focus-within:border-accent";
</script>

<template>
  <div
    class="grid max-w-xl grid-cols-[120px_minmax(0,1fr)] items-center gap-x-4 gap-y-2.5 p-3 text-xs max-sm:grid-cols-1"
  >
    <label :for="`${idPrefix}-type`" class="label-caps">Tipe</label>
    <select
      :id="`${idPrefix}-type`"
      :value="auth.type"
      :class="selectClass"
      @change="setType(($event.target as HTMLSelectElement).value as Auth['type'])"
    >
      <option v-for="o in options" :key="o.type" :value="o.type">{{ o.label }}</option>
    </select>

    <template v-if="auth.type === 'bearer'">
      <span class="label-caps">Token</span>
      <div :class="field">
        <VariableInput v-model="auth.token" label="Token" placeholder="{{token}}" />
      </div>
    </template>

    <template v-else-if="auth.type === 'basic'">
      <span class="label-caps">Username</span>
      <div :class="field"><VariableInput v-model="auth.username" label="Username" /></div>
      <label :for="`${idPrefix}-password`" class="label-caps">Password</label>
      <input
        :id="`${idPrefix}-password`"
        v-model="auth.password"
        type="password"
        autocomplete="off"
        :class="inputClass"
      />
    </template>

    <template v-else-if="auth.type === 'api_key'">
      <span class="label-caps">Key</span>
      <div :class="field"><VariableInput v-model="auth.key" label="Nama key" /></div>
      <span class="label-caps">Value</span>
      <div :class="field">
        <VariableInput v-model="auth.value" label="Nilai key" placeholder="{{api_key}}" />
      </div>
      <label :for="`${idPrefix}-location`" class="label-caps">Kirim lewat</label>
      <select :id="`${idPrefix}-location`" v-model="auth.location" :class="selectClass">
        <option value="header">Header</option>
        <option value="query">Query param</option>
      </select>
    </template>

    <p v-if="auth.type !== 'none' && auth.type !== 'inherit'" class="col-span-full text-muted">
      Simpan kredensial sebagai variabel rahasia di environment, lalu rujuk dengan
      <code class="font-mono" v-text="'{{nama}}'" />. Nilai rahasia disimpan di keychain sistem.
    </p>
    <p v-else-if="auth.type === 'inherit'" class="col-span-full text-muted">
      Auth diambil dari folder atau koleksi induk.
    </p>
  </div>
</template>
