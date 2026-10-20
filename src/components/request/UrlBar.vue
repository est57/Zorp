<script setup lang="ts">
import { computed, ref } from "vue";
import type { HttpMethod } from "@/bindings/HttpMethod";
import type { RequestSpec } from "@/bindings/RequestSpec";
import AppButton from "@/components/ui/AppButton.vue";
import VariableInput from "@/components/ui/VariableInput.vue";
import { useDialogsStore } from "@/stores/dialogs";
import { useToastsStore } from "@/stores/toasts";
import { applyCurl, looksLikeCurl } from "@/utils/curl/apply";
import { parseCurl } from "@/utils/curl/parse";
import type { ConnectionStatus } from "@/stores/realtime";
import { METHODS, specColor } from "@/utils/request";
import { paramsFromUrl } from "@/utils/url";

const props = defineProps<{
  spec: RequestSpec;
  loading: boolean;
  dirty: boolean;
  /** Connection state for WebSocket and SSE requests. */
  connection: ConnectionStatus;
}>();
const emit = defineEmits<{ send: []; cancel: []; save: []; code: [] }>();
const dialogs = useDialogsStore();
const toasts = useToastsStore();

const urlInput = ref<InstanceType<typeof VariableInput>>();

const realtime = computed(() => props.spec.mode !== "http");
const connected = computed(() => props.connection === "open" || props.connection === "connecting");

/** One select covers HTTP methods and the real-time modes. */
const protocol = computed({
  get: () => (props.spec.mode === "http" ? props.spec.method : props.spec.mode),
  set: (value: string) => {
    if (value === "websocket" || value === "sse") {
      props.spec.mode = value;
    } else {
      props.spec.mode = "http";
      props.spec.method = value as HttpMethod;
    }
  },
});

const placeholder = computed(() =>
  props.spec.mode === "websocket"
    ? "wss://contoh.com/socket atau {{ws_url}}"
    : "https://api.contoh.com/v1/users atau {{base_url}}/users",
);

function onUrl(url: string) {
  props.spec.url = url;
  props.spec.params = paramsFromUrl(url, props.spec.params);
}

async function onPaste(event: ClipboardEvent) {
  const text = event.clipboardData?.getData("text") ?? "";
  if (!looksLikeCurl(text)) return;
  event.preventDefault();
  const ok = await dialogs.confirm(
    "Tempel sebagai request cURL?",
    "Method, URL, header, body, dan auth request ini akan diganti sesuai perintah cURL.",
    "Impor",
    false,
  );
  if (!ok) return;
  try {
    const parsed = parseCurl(text);
    applyCurl(props.spec, parsed);
    toasts.push(
      parsed.warnings[0] ?? "Request diisi dari perintah cURL",
      parsed.warnings.length ? "danger" : "success",
    );
  } catch (error) {
    toasts.push(
      `cURL tidak bisa dibaca: ${error instanceof Error ? error.message : error}`,
      "danger",
    );
  }
}

defineExpose({ focus: () => urlInput.value?.focus() });
</script>

<template>
  <div class="flex min-w-0 gap-2 px-3 py-2.5">
    <div
      class="flex h-9 min-w-0 flex-1 items-stretch overflow-hidden rounded-md border border-line bg-bg focus-within:border-accent"
    >
      <select
        v-model="protocol"
        aria-label="Method atau protokol"
        class="w-[92px] shrink-0 cursor-pointer border-r border-line bg-transparent px-2.5 font-mono text-xs font-bold outline-none max-sm:w-[76px]"
        :class="specColor(spec)"
      >
        <optgroup label="HTTP" class="text-fg">
          <option v-for="method in METHODS" :key="method" :value="method">{{ method }}</option>
        </optgroup>
        <optgroup label="Real-time" class="text-fg">
          <option value="websocket">WS</option>
          <option value="sse">SSE</option>
        </optgroup>
      </select>
      <VariableInput
        ref="urlInput"
        input-id="request-url"
        :model-value="spec.url"
        label="URL"
        size="md"
        :placeholder="placeholder"
        @update:model-value="onUrl"
        @enter="emit('send')"
        @paste="onPaste"
      />
    </div>
    <AppButton
      v-if="realtime"
      :variant="connected ? 'danger' : 'primary'"
      class="!h-9 w-[104px] max-sm:w-20"
      @click="emit('send')"
    >
      <span
        v-if="connection === 'connecting'"
        class="size-3 animate-spin rounded-full border-2 border-current border-r-transparent"
      />
      {{ connected ? "Putuskan" : "Hubungkan" }}
    </AppButton>
    <AppButton
      v-else-if="loading"
      variant="danger"
      class="!h-9 w-[88px] max-sm:w-16"
      @click="emit('cancel')"
    >
      <span class="size-3 animate-spin rounded-full border-2 border-current border-r-transparent" />
      Batal
    </AppButton>
    <AppButton
      v-else
      variant="primary"
      class="!h-9 w-[88px] text-[13px] max-sm:w-16"
      @click="emit('send')"
    >
      Kirim
    </AppButton>
    <AppButton
      v-if="spec.mode !== 'websocket'"
      icon="code"
      class="!h-9 !w-9 !px-0"
      aria-label="Lihat kode"
      title="Lihat kode"
      @click="emit('code')"
    />
    <AppButton
      icon="save"
      class="!h-9 max-md:!w-9 max-md:!px-0"
      :aria-label="dirty ? 'Simpan perubahan' : 'Simpan'"
      @click="emit('save')"
    >
      <span class="max-md:hidden">Simpan</span>
    </AppButton>
  </div>
</template>
