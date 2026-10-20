<script setup lang="ts">
import { computed, ref } from "vue";
import AppButton from "@/components/ui/AppButton.vue";
import { modKey } from "@/composables/usePlatform";
import { useRealtimeStore } from "@/stores/realtime";
import type { Tab } from "@/stores/tabs";
import { useToastsStore } from "@/stores/toasts";
import RealtimeLog from "./RealtimeLog.vue";

const props = defineProps<{ tab: Tab }>();
const realtime = useRealtimeStore();
const toasts = useToastsStore();
const draft = ref("");

const connection = computed(() => realtime.get(props.tab.id));
const status = computed(() => connection.value?.status ?? "idle");
const isWebSocket = computed(() => props.tab.draft.mode === "websocket");
const counts = computed(() => {
  const events = connection.value?.events ?? [];
  return {
    in: events.filter((e) => e.kind === "message").length,
    out: events.filter((e) => e.kind === "sent").length,
  };
});
const statusText = computed(
  () =>
    ({
      idle: "Belum terhubung",
      connecting: "Menghubungkan",
      open: "Terhubung",
      closed: "Terputus",
    })[status.value],
);

async function send() {
  if (!draft.value.trim()) return;
  await toasts.guard(() => realtime.send(props.tab.id, draft.value));
  draft.value = "";
}
</script>

<template>
  <section aria-label="Koneksi real-time" class="flex h-full min-h-0 flex-col bg-surface">
    <div class="flex min-h-10 flex-wrap items-center gap-3 px-3 py-1.5 text-xs">
      <span class="label-caps">{{ isWebSocket ? "WebSocket" : "Server-Sent Events" }}</span>
      <span class="inline-flex items-center gap-1.5 font-medium" role="status">
        <span
          class="size-[7px] rounded-full"
          :class="{
            'bg-success': status === 'open',
            'bg-warning': status === 'connecting',
            'bg-subtle': status === 'idle' || status === 'closed',
          }"
        />
        {{ statusText }}
      </span>
      <span class="font-mono text-muted tabular-nums"
        >{{ counts.in }} masuk<template v-if="isWebSocket">
          · {{ counts.out }} keluar</template
        ></span
      >
      <AppButton
        variant="ghost"
        class="ml-auto"
        :disabled="!connection?.events.length"
        @click="realtime.clear(tab.id)"
        >Bersihkan</AppButton
      >
    </div>
    <form v-if="isWebSocket" class="flex gap-2 border-t border-line p-2" @submit.prevent="send">
      <textarea
        v-model="draft"
        aria-label="Pesan WebSocket"
        rows="2"
        spellcheck="false"
        :placeholder="
          status === 'open'
            ? `Tulis pesan, ${modKey} Enter untuk mengirim`
            : 'Hubungkan dulu untuk mengirim pesan'
        "
        class="min-w-0 flex-1 resize-y rounded-md border border-line bg-bg px-2 py-1.5 font-mono text-xs text-fg outline-none select-text focus:border-accent"
        @keydown.meta.enter.prevent.stop="send"
        @keydown.ctrl.enter.prevent.stop="send"
      />
      <AppButton
        type="submit"
        variant="primary"
        class="!h-auto"
        :disabled="status !== 'open' || !draft.trim()"
        >Kirim</AppButton
      >
    </form>
    <div class="min-h-0 flex-1 border-t border-line">
      <RealtimeLog :events="connection?.events ?? []" />
    </div>
  </section>
</template>
