<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { MockRouteInfo } from "@/bindings/MockRouteInfo";
import { mockRoutes } from "@/api/mock";
import AppButton from "@/components/ui/AppButton.vue";
import ModalDialog from "@/components/ui/ModalDialog.vue";
import { inputClass } from "@/components/ui/styles";
import { useDialogsStore } from "@/stores/dialogs";
import { useMocksStore } from "@/stores/mocks";
import { useToastsStore } from "@/stores/toasts";
import { statusTone } from "@/utils/format";
import { methodColor } from "@/utils/request";
import type { HttpMethod } from "@/bindings/HttpMethod";

const dialogs = useDialogsStore();
const mocks = useMocksStore();
const toasts = useToastsStore();

const routes = ref<MockRouteInfo[]>([]);
const port = ref(4010);
const target = computed(() => dialogs.mockTarget);
const status = computed(() => (target.value ? mocks.statusOf(target.value.collectionId) : null));
const log = computed(() =>
  mocks.log
    .filter((e) => e.collectionId === target.value?.collectionId)
    .slice(-50)
    .reverse(),
);
const open = computed({
  get: () => target.value !== null,
  set: (v) => !v && (dialogs.mockTarget = null),
});

watch(target, async (t) => {
  routes.value = [];
  if (t) routes.value = (await toasts.guard(() => mockRoutes(t.collectionId))) ?? [];
});

async function start() {
  if (!target.value) return;
  const p =
    Number.isInteger(port.value) && port.value > 0 && port.value < 65536 ? port.value : null;
  await toasts.guard(() => mocks.start(target.value!.collectionId, p));
}

async function copy(url: string) {
  try {
    await navigator.clipboard.writeText(url);
    toasts.push("URL mock disalin", "success");
  } catch {
    toasts.push("Gagal menyalin ke clipboard", "danger");
  }
}

const time = (at: number) => new Date(at).toLocaleTimeString("id-ID");
const cell = "border-b border-line px-2 py-1.5";
</script>

<template>
  <ModalDialog
    v-model:open="open"
    wide
    :title="`Mock server: ${target?.name ?? ''}`"
    description="Menyajikan contoh respons yang tersimpan, hanya di komputer ini (127.0.0.1)."
  >
    <div class="grid gap-3 text-xs">
      <div v-if="status" class="flex items-center gap-2">
        <span
          class="rounded-full bg-[color-mix(in_srgb,var(--success)_14%,transparent)] px-2 py-0.5 font-medium text-success"
          >Berjalan</span
        >
        <code class="min-w-0 flex-1 truncate font-mono select-text">{{ status.url }}</code>
        <AppButton icon="copy" @click="copy(status.url)">Salin URL</AppButton>
      </div>
      <div v-else class="flex items-center gap-2">
        <label class="flex items-center gap-2 text-muted"
          >Port
          <input
            v-model.number="port"
            type="number"
            min="1"
            max="65535"
            :class="[inputClass, '!w-24']"
          />
        </label>
        <span class="text-subtle"
          >Base URL request diganti dengan http://127.0.0.1:{{ port }}.</span
        >
      </div>

      <div class="max-h-48 overflow-auto rounded-md border border-line">
        <table class="w-full border-collapse text-left">
          <thead class="bg-surface-2 text-[11px] text-subtle uppercase">
            <tr>
              <th :class="cell">Method</th>
              <th :class="cell">Path</th>
              <th :class="cell">Contoh yang disajikan</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="r in routes" :key="r.method + r.path">
              <td
                :class="[
                  cell,
                  'font-mono text-[11px] font-semibold',
                  methodColor(r.method as HttpMethod),
                ]"
              >
                {{ r.method }}
              </td>
              <td :class="[cell, 'font-mono']">{{ r.path }}</td>
              <td :class="cell">
                <template v-if="r.example">
                  <span class="font-mono font-semibold" :class="statusTone(r.status)">{{
                    r.status
                  }}</span>
                  {{ r.example }}
                </template>
                <span v-else class="text-subtle">Belum ada contoh, membalas 501</span>
              </td>
            </tr>
            <tr v-if="!routes.length">
              <td :class="[cell, 'text-subtle']" colspan="3">
                Koleksi ini belum punya request HTTP.
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="status" class="grid gap-1.5">
        <span class="text-muted">Log request</span>
        <ul
          class="max-h-32 overflow-auto rounded-md border border-line bg-bg p-2 font-mono text-[11px] leading-5 select-text"
        >
          <li v-for="(e, i) in log" :key="i" class="flex gap-3 whitespace-nowrap">
            <span class="text-subtle tabular-nums">{{ time(e.at) }}</span>
            <span class="w-14">{{ e.method }}</span>
            <span class="min-w-0 flex-1 truncate">{{ e.path }}</span>
            <span :class="statusTone(e.status)">{{ e.status }}</span>
            <span class="max-w-40 truncate text-muted">{{ e.example ?? "" }}</span>
          </li>
          <li v-if="!log.length" class="text-subtle">Belum ada request masuk.</li>
        </ul>
      </div>

      <p class="text-[11px] leading-4 text-subtle">
        Path diambil dari URL request setelah base URL; bagian seperti
        <code class="font-mono" v-text="'{{id}}'" /> menjadi parameter. Jika ada beberapa contoh,
        contoh pertama yang dipakai; header <code class="font-mono">x-zorp-example</code> bisa
        memilih contoh lain berdasarkan nama.
      </p>

      <div class="flex justify-end gap-2">
        <AppButton @click="open = false">Tutup</AppButton>
        <AppButton
          v-if="status"
          variant="danger"
          @click="toasts.guard(() => mocks.stop(target!.collectionId))"
          >Hentikan</AppButton
        >
        <AppButton v-else variant="primary" @click="start">Mulai mock server</AppButton>
      </div>
    </div>
  </ModalDialog>
</template>
