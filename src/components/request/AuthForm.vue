<script setup lang="ts">
import type { Auth } from "@/bindings/Auth";
import SecretField from "@/components/ui/SecretField.vue";
import VariableInput from "@/components/ui/VariableInput.vue";
import OAuth2Fields from "./OAuth2Fields.vue";
import SignedAuthFields from "./SignedAuthFields.vue";
import { selectClass } from "@/components/ui/styles";

const props = withDefaults(
  defineProps<{ inheritLabel?: string; idPrefix?: string; collectionId?: string | null }>(),
  { inheritLabel: "", idPrefix: "auth", collectionId: null },
);
const auth = defineModel<Auth>({ required: true });

const options: { type: Auth["type"]; label: string }[] = [
  ...(props.inheritLabel ? [{ type: "inherit" as const, label: props.inheritLabel }] : []),
  { type: "none", label: "Tanpa auth" },
  { type: "bearer", label: "Bearer token" },
  { type: "basic", label: "Basic auth" },
  { type: "api_key", label: "API key" },
  { type: "oauth2", label: "OAuth 2.0" },
  { type: "oauth1", label: "OAuth 1.0" },
  { type: "jwt", label: "JWT Bearer" },
  { type: "digest", label: "Digest auth" },
  { type: "aws_sigv4", label: "AWS Signature v4" },
];

function setType(type: Auth["type"]) {
  const defaults: Record<Auth["type"], Auth> = {
    inherit: { type: "inherit" },
    none: { type: "none" },
    bearer: { type: "bearer", token: "" },
    basic: { type: "basic", username: "", password: "" },
    api_key: { type: "api_key", key: "X-API-Key", value: "", location: "header" },
    oauth2: {
      type: "oauth2",
      grant: "authorization_code",
      authUrl: "",
      tokenUrl: "",
      clientId: "",
      clientSecret: "",
      scope: "",
      usePkce: true,
      clientAuth: "basic",
      callbackPort: 0,
      tokenKey: crypto.randomUUID(),
    },
    digest: { type: "digest", username: "", password: "" },
    jwt: {
      type: "jwt",
      algorithm: "HS256",
      secret: "",
      secretBase64: false,
      payload: '{\n  "sub": ""\n}',
      headers: "",
      location: "header",
      param: "Authorization",
      prefix: "Bearer",
    },
    aws_sigv4: {
      type: "aws_sigv4",
      accessKey: "",
      secretKey: "",
      sessionToken: "",
      region: "",
      service: "",
    },
    oauth1: {
      type: "oauth1",
      consumerKey: "",
      consumerSecret: "",
      token: "",
      tokenSecret: "",
      signatureMethod: "HMAC-SHA1",
      realm: "",
    },
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
      <SecretField v-model="auth.token" label="Token" placeholder="{{token}}" />
    </template>

    <template v-else-if="auth.type === 'basic'">
      <span class="label-caps">Username</span>
      <div :class="field"><VariableInput v-model="auth.username" label="Username" /></div>
      <label :for="`${idPrefix}-password`" class="label-caps">Password</label>
      <SecretField
        v-model="auth.password"
        label="Password"
        placeholder="{{password}}"
        :input-id="`${idPrefix}-password`"
      />
    </template>

    <template v-else-if="auth.type === 'api_key'">
      <span class="label-caps">Key</span>
      <div :class="field"><VariableInput v-model="auth.key" label="Nama key" /></div>
      <span class="label-caps">Value</span>
      <SecretField v-model="auth.value" label="Nilai key" placeholder="{{api_key}}" />
      <label :for="`${idPrefix}-location`" class="label-caps">Kirim lewat</label>
      <select :id="`${idPrefix}-location`" v-model="auth.location" :class="selectClass">
        <option value="header">Header</option>
        <option value="query">Query param</option>
      </select>
    </template>

    <SignedAuthFields
      v-else-if="
        auth.type === 'digest' ||
        auth.type === 'jwt' ||
        auth.type === 'aws_sigv4' ||
        auth.type === 'oauth1'
      "
      v-model="auth"
      :id-prefix="idPrefix"
    />

    <OAuth2Fields
      v-else-if="auth.type === 'oauth2'"
      :model-value="auth"
      :collection-id="collectionId"
      :id-prefix="idPrefix"
    />

    <p v-if="auth.type !== 'none' && auth.type !== 'inherit'" class="col-span-full text-muted">
      Simpan kredensial sebagai variabel rahasia di environment, lalu rujuk dengan
      <code class="font-mono" v-text="'{{nama}}'" />. Nilai rahasia disimpan di keychain sistem.
    </p>
    <p v-else-if="auth.type === 'inherit'" class="col-span-full text-muted">
      Auth diambil dari folder atau koleksi induk.
    </p>
  </div>
</template>
