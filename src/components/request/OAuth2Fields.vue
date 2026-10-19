<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { OAuth2Config } from "@/bindings/OAuth2Config";
import type { TokenStatus } from "@/bindings/TokenStatus";
import { cancelOAuth, clearOAuthToken, getOAuthToken, oauthTokenStatus } from "@/api/oauth";
import { isTauri, toApiError } from "@/api/invoke";
import AppButton from "@/components/ui/AppButton.vue";
import VariableInput from "@/components/ui/VariableInput.vue";
import { checkboxClass, inputClass, selectClass } from "@/components/ui/styles";
import { useToastsStore } from "@/stores/toasts";

const config = defineModel<OAuth2Config>({ required: true });
const props = defineProps<{ collectionId: string | null; idPrefix: string }>();
const toasts = useToastsStore();

const status = ref<TokenStatus | null>(null);
const busy = ref(false);
const field = "flex h-7 min-w-0 rounded border border-line bg-bg focus-within:border-accent";
const isCode = computed(() => config.value.grant === "authorization_code");
const redirect = computed(
  () => `http://127.0.0.1:${config.value.callbackPort || "<port acak>"}/callback`,
);

const statusText = computed(() => {
  const s = status.value;
  if (!s?.hasToken) return "Belum ada token.";
  if (s.expiresAt === null) return "Token tersedia, tanpa waktu kedaluwarsa.";
  const time = new Date(s.expiresAt).toLocaleString("id-ID", {
    dateStyle: "medium",
    timeStyle: "short",
  });
  const prefix = s.expiresAt < Date.now() ? "Token kedaluwarsa sejak" : "Token berlaku sampai";
  return `${prefix} ${time}${s.hasRefreshToken ? ", diperbarui otomatis" : ""}.`;
});

async function refreshStatus() {
  if (isTauri()) status.value = await oauthTokenStatus(config.value.tokenKey).catch(() => null);
}
watch(() => config.value.tokenKey, refreshStatus, { immediate: true });

async function fetchToken() {
  busy.value = true;
  try {
    status.value = await getOAuthToken(config.value, props.collectionId);
    toasts.push("Token OAuth berhasil diambil", "success");
  } catch (error) {
    const apiError = toApiError(error);
    if (apiError.kind !== "cancelled") toasts.push(apiError.message, "danger");
  } finally {
    busy.value = false;
  }
}

async function clearToken() {
  await toasts.guard(() => clearOAuthToken(config.value.tokenKey));
  await refreshStatus();
}

const id = (name: string) => `${props.idPrefix}-${name}`;
</script>

<template>
  <label :for="id('grant')" class="label-caps">Grant</label>
  <select :id="id('grant')" v-model="config.grant" :class="selectClass">
    <option value="authorization_code">Authorization Code</option>
    <option value="client_credentials">Client Credentials</option>
  </select>
  <template v-if="isCode">
    <span class="label-caps">Auth URL</span>
    <div :class="field">
      <VariableInput
        v-model="config.authUrl"
        label="Auth URL"
        placeholder="https://contoh.com/oauth/authorize"
      />
    </div>
  </template>
  <span class="label-caps">Token URL</span>
  <div :class="field">
    <VariableInput
      v-model="config.tokenUrl"
      label="Token URL"
      placeholder="https://contoh.com/oauth/token"
    />
  </div>
  <span class="label-caps">Client ID</span>
  <div :class="field"><VariableInput v-model="config.clientId" label="Client ID" /></div>
  <span class="label-caps">Client secret</span>
  <div :class="field">
    <VariableInput
      v-model="config.clientSecret"
      label="Client secret"
      placeholder="{{client_secret}}"
    />
  </div>
  <span class="label-caps">Scope</span>
  <div :class="field">
    <VariableInput v-model="config.scope" label="Scope" placeholder="read write" />
  </div>
  <label :for="id('client-auth')" class="label-caps">Kirim kredensial</label>
  <select :id="id('client-auth')" v-model="config.clientAuth" :class="selectClass">
    <option value="basic">Header Basic auth</option>
    <option value="body">Di body request</option>
  </select>
  <template v-if="isCode">
    <span class="label-caps">PKCE</span>
    <label class="flex items-center gap-2">
      <input v-model="config.usePkce" type="checkbox" :class="checkboxClass" />
      Gunakan PKCE (S256)
    </label>
    <label :for="id('port')" class="label-caps">Port callback</label>
    <div class="grid gap-1">
      <input
        :id="id('port')"
        v-model.number="config.callbackPort"
        type="number"
        min="0"
        max="65535"
        :class="[inputClass, 'w-28']"
      />
      <span class="text-subtle"
        >Redirect URI: <code class="font-mono select-text">{{ redirect }}</code
        >. Isi 0 untuk port acak.</span
      >
    </div>
  </template>
  <span class="label-caps">Token</span>
  <div class="flex flex-wrap items-center gap-2">
    <AppButton v-if="busy && isCode" variant="danger" @click="cancelOAuth()"
      >Batalkan login</AppButton
    >
    <AppButton v-else variant="primary" :disabled="busy" @click="fetchToken">{{
      busy ? "Mengambil token" : "Ambil token"
    }}</AppButton>
    <AppButton v-if="status?.hasToken" variant="ghost" icon="trash" @click="clearToken"
      >Hapus token</AppButton
    >
    <span class="text-muted" role="status">{{
      busy && isCode ? "Selesaikan login di browser." : statusText
    }}</span>
  </div>
</template>
