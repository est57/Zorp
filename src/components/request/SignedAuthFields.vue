<script setup lang="ts">
import { computed, ref } from "vue";
import type { Auth } from "@/bindings/Auth";
import type { JwtAlgorithm } from "@/bindings/JwtAlgorithm";
import AppIcon from "@/components/ui/AppIcon.vue";
import CodeEditor from "@/components/ui/CodeEditor.vue";
import SecretField from "@/components/ui/SecretField.vue";
import VariableInput from "@/components/ui/VariableInput.vue";
import { checkboxClass, selectClass } from "@/components/ui/styles";

type Signed = Extract<Auth, { type: "digest" | "jwt" | "aws_sigv4" | "oauth1" }>;
const auth = defineModel<Signed>({ required: true });
defineProps<{ idPrefix: string }>();

const ALGORITHMS: JwtAlgorithm[] = [
  "HS256",
  "HS384",
  "HS512",
  "RS256",
  "RS384",
  "RS512",
  "ES256",
  "ES384",
];
const usesPem = computed(() => auth.value.type === "jwt" && !auth.value.algorithm.startsWith("HS"));
const showKey = ref(false);
const field = "flex h-7 min-w-0 rounded border border-line bg-bg focus-within:border-accent";
const editor =
  "h-28 min-w-0 overflow-hidden rounded border border-line bg-bg focus-within:border-accent";
</script>

<template>
  <template v-if="auth.type === 'digest'">
    <span class="label-caps">Username</span>
    <div :class="field"><VariableInput v-model="auth.username" label="Username" /></div>
    <span class="label-caps">Password</span>
    <SecretField v-model="auth.password" label="Password" placeholder="{{password}}" />
    <p class="col-span-full text-muted">
      Request dikirim sekali tanpa kredensial; jika server meminta Digest, Zorp menjawabnya
      otomatis.
    </p>
  </template>

  <template v-else-if="auth.type === 'jwt'">
    <label :for="`${idPrefix}-jwt-alg`" class="label-caps">Algoritma</label>
    <select :id="`${idPrefix}-jwt-alg`" v-model="auth.algorithm" :class="selectClass">
      <option v-for="a in ALGORITHMS" :key="a" :value="a">{{ a }}</option>
    </select>
    <span class="label-caps self-start pt-1.5">{{ usesPem ? "Private key" : "Secret" }}</span>
    <div v-if="usesPem" class="relative">
      <textarea
        v-model="auth.secret"
        aria-label="Private key PEM"
        spellcheck="false"
        placeholder="-----BEGIN PRIVATE KEY-----"
        class="h-24 w-full resize-y rounded border border-line bg-bg p-2 pr-8 font-mono text-xs outline-none focus:border-accent"
        :class="
          !showKey && auth.secret && !auth.secret.includes('{{') && '[-webkit-text-security:disc]'
        "
      />
      <button
        type="button"
        class="absolute top-1.5 right-1.5 text-subtle hover:text-fg"
        :aria-label="showKey ? 'Sembunyikan private key' : 'Tampilkan private key'"
        @click="showKey = !showKey"
      >
        <AppIcon :name="showKey ? 'eyeOff' : 'eye'" :size="14" />
      </button>
    </div>
    <div v-else class="grid gap-1.5">
      <SecretField v-model="auth.secret" label="Secret" placeholder="{{jwt_secret}}" />
      <label class="flex items-center gap-2 text-muted">
        <input v-model="auth.secretBase64" type="checkbox" :class="checkboxClass" />
        Secret dalam bentuk base64
      </label>
    </div>
    <span class="label-caps self-start pt-1.5">Payload</span>
    <div :class="editor">
      <CodeEditor v-model="auth.payload" language="json" label="Payload JWT" />
    </div>
    <span class="label-caps self-start pt-1.5">Header tambahan</span>
    <div :class="editor">
      <CodeEditor v-model="auth.headers" language="json" label="Header JWT tambahan" />
    </div>
    <label :for="`${idPrefix}-jwt-loc`" class="label-caps">Kirim lewat</label>
    <select :id="`${idPrefix}-jwt-loc`" v-model="auth.location" :class="selectClass">
      <option value="header">Header</option>
      <option value="query">Query param</option>
    </select>
    <span class="label-caps">{{ auth.location === "header" ? "Nama header" : "Nama param" }}</span>
    <div :class="field"><VariableInput v-model="auth.param" label="Nama header atau param" /></div>
    <template v-if="auth.location === 'header'">
      <span class="label-caps">Prefix</span>
      <div :class="field">
        <VariableInput v-model="auth.prefix" label="Prefix token" placeholder="Bearer" />
      </div>
    </template>
    <p class="col-span-full text-muted">
      Token ditandatangani ulang setiap kali request dikirim; klaim iat ditambahkan jika belum ada.
    </p>
  </template>

  <template v-else-if="auth.type === 'aws_sigv4'">
    <span class="label-caps">Access key</span>
    <div :class="field">
      <VariableInput v-model="auth.accessKey" label="Access key" placeholder="{{aws_access_key}}" />
    </div>
    <span class="label-caps">Secret key</span>
    <SecretField v-model="auth.secretKey" label="Secret key" placeholder="{{aws_secret_key}}" />
    <span class="label-caps">Session token</span>
    <SecretField v-model="auth.sessionToken" label="Session token" placeholder="Opsional" />
    <span class="label-caps">Region</span>
    <div :class="field">
      <VariableInput v-model="auth.region" label="Region" placeholder="ap-southeast-1" />
    </div>
    <span class="label-caps">Service</span>
    <div :class="field">
      <VariableInput v-model="auth.service" label="Service" placeholder="execute-api" />
    </div>
  </template>

  <template v-else-if="auth.type === 'oauth1'">
    <span class="label-caps">Consumer key</span>
    <div :class="field"><VariableInput v-model="auth.consumerKey" label="Consumer key" /></div>
    <span class="label-caps">Consumer secret</span>
    <SecretField
      v-model="auth.consumerSecret"
      label="Consumer secret"
      placeholder="{{consumer_secret}}"
    />
    <span class="label-caps">Access token</span>
    <div :class="field">
      <VariableInput v-model="auth.token" label="Access token" placeholder="Opsional" />
    </div>
    <span class="label-caps">Token secret</span>
    <SecretField v-model="auth.tokenSecret" label="Token secret" placeholder="Opsional" />
    <label :for="`${idPrefix}-oauth1-method`" class="label-caps">Tanda tangan</label>
    <select :id="`${idPrefix}-oauth1-method`" v-model="auth.signatureMethod" :class="selectClass">
      <option value="HMAC-SHA1">HMAC-SHA1</option>
      <option value="HMAC-SHA256">HMAC-SHA256</option>
      <option value="PLAINTEXT">PLAINTEXT</option>
    </select>
    <span class="label-caps">Realm</span>
    <div :class="field">
      <VariableInput v-model="auth.realm" label="Realm" placeholder="Opsional" />
    </div>
  </template>
</template>
