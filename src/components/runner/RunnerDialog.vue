<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import type { RunItemResult } from "@/bindings/RunItemResult";
import type { RunSummary } from "@/bindings/RunSummary";
import { save as saveFile } from "@tauri-apps/plugin-dialog";
import { cancelRun, onRunnerItem, runCollection, saveRunReport } from "@/api/runner";
import AppButton from "@/components/ui/AppButton.vue";
import ModalDialog from "@/components/ui/ModalDialog.vue";
import { checkboxClass, inputClass } from "@/components/ui/styles";
import { useDialogsStore } from "@/stores/dialogs";
import { useEnvironmentsStore } from "@/stores/environments";
import { useToastsStore } from "@/stores/toasts";
import { formatDuration } from "@/utils/format";
import { newId } from "@/utils/request";
import { reportHtml, reportJson } from "@/utils/runReport";
import RunDataPicker from "./RunDataPicker.vue";
import RunResults from "./RunResults.vue";

const dialogs = useDialogsStore();
const toasts = useToastsStore();
const envs = useEnvironmentsStore();

const delayMs = ref(0);
const stopOnFailure = ref(false);
const dataPath = ref<string | null>(null);
const iterations = ref(1);
const runId = ref<string | null>(null);
const items = ref<RunItemResult[]>([]);
const summary = ref<RunSummary | null>(null);
const unlisten = onRunnerItem((event) => {
  if (event.runId === runId.value) items.value.push(event.item);
});
onBeforeUnmount(() => void unlisten.then((stop) => stop()));

const target = computed(() => dialogs.runnerTarget);
const running = computed(() => runId.value !== null && summary.value === null);
const open = computed({
  get: () => target.value !== null,
  set: (v) => {
    if (v) return;
    if (running.value && runId.value) void cancelRun(runId.value);
    dialogs.runnerTarget = null;
  },
});
const progress = computed(() =>
  items.value[0] ? (items.value.length / items.value[0].total) * 100 : 0,
);

watch(target, () => {
  runId.value = null;
  items.value = [];
  summary.value = null;
  dataPath.value = null;
  iterations.value = 1;
});

async function exportReport() {
  if (!summary.value || !target.value) return;
  const path = await saveFile({
    defaultPath: `laporan-${target.value.name.replace(/[^\w-]+/g, "-").toLowerCase()}.html`,
    filters: [
      { name: "Laporan HTML", extensions: ["html"] },
      { name: "Laporan JSON", extensions: ["json"] },
    ],
  });
  if (!path) return;
  const report = {
    name: target.value.name,
    finishedAt: new Date().toLocaleString("id-ID"),
    summary: summary.value,
    items: items.value,
  };
  const contents = path.toLowerCase().endsWith(".json") ? reportJson(report) : reportHtml(report);
  const ok = await toasts.guard(() => saveRunReport(path, contents).then(() => true));
  if (ok) toasts.push("Laporan disimpan", "success");
}

async function start() {
  const t = target.value;
  if (!t) return;
  const id = newId();
  runId.value = id;
  items.value = [];
  summary.value = null;
  const options = {
    delayMs: Math.max(0, Math.round(delayMs.value) || 0),
    stopOnFailure: stopOnFailure.value,
    dataPath: dataPath.value,
  };
  const result = await toasts.guard(() => runCollection(id, t.collectionId, t.folderId, options));
  if (runId.value !== id) return;
  if (result) summary.value = result;
  else runId.value = null;
  void envs.load();
}
</script>

<template>
  <ModalDialog
    v-model:open="open"
    wide
    :title="`Jalankan ${target?.name ?? ''}`"
    description="Request dijalankan berurutan sesuai sidebar, lengkap dengan script dan tes. Hasilnya tidak masuk riwayat."
  >
    <div class="grid gap-3 text-xs">
      <div class="flex flex-wrap items-center gap-x-5 gap-y-2">
        <label class="flex items-center gap-2">
          Jeda antar request
          <input
            v-model.number="delayMs"
            type="number"
            min="0"
            max="60000"
            :disabled="running"
            :class="[inputClass, '!w-24']"
          />
          ms
        </label>
        <label class="flex items-center gap-2">
          <input
            v-model="stopOnFailure"
            type="checkbox"
            :disabled="running"
            :class="checkboxClass"
          />
          Berhenti saat ada yang gagal
        </label>
      </div>
      <RunDataPicker
        v-if="!summary && !running"
        v-model="dataPath"
        :disabled="running"
        @rows="(n) => (iterations = n)"
      />

      <div
        v-if="runId"
        class="h-1 overflow-hidden rounded-full bg-surface-2"
        role="progressbar"
        :aria-valuenow="Math.round(progress)"
      >
        <div
          class="h-full bg-accent transition-[width]"
          :style="{ width: `${summary ? 100 : progress}%` }"
        />
      </div>
      <p v-if="summary" class="font-medium" role="status">
        <span class="text-success">{{ summary.passed }} lulus</span>,
        <span :class="summary.failed ? 'text-danger' : 'text-muted'"
          >{{ summary.failed }} gagal</span
        >
        dari {{ summary.total }} request<span v-if="summary.iterations > 1">
          dalam {{ summary.iterations }} iterasi</span
        >, {{ formatDuration(summary.durationMs)
        }}<span v-if="summary.cancelled">, dibatalkan</span>.
      </p>
      <RunResults v-if="items.length" :items="items" :grouped="Boolean(dataPath)" />

      <div class="flex justify-end gap-2">
        <AppButton v-if="running" variant="danger" @click="runId && cancelRun(runId)"
          >Batalkan</AppButton
        >
        <template v-else>
          <AppButton v-if="summary" icon="download" class="mr-auto" @click="exportReport"
            >Ekspor laporan</AppButton
          >
          <AppButton @click="open = false">Tutup</AppButton>
          <AppButton variant="primary" @click="start">{{
            summary ? "Jalankan lagi" : dataPath ? `Mulai ${iterations} iterasi` : "Mulai"
          }}</AppButton>
        </template>
      </div>
    </div>
  </ModalDialog>
</template>
