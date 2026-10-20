<script setup lang="ts">
import { useUpdaterStore } from "@/stores/updater";

const updater = useUpdaterStore();
const link = "font-medium text-accent hover:underline";
</script>

<template>
  <span
    v-if="updater.status === 'available' && updater.update"
    class="flex items-center gap-1.5"
    role="status"
    :title="updater.update.body || undefined"
  >
    Versi {{ updater.update.version }} tersedia.
    <button type="button" :class="link" @click="updater.install()">Perbarui</button>
  </span>
  <span v-else-if="updater.status === 'downloading'" role="status" class="tabular-nums">
    Mengunduh pembaruan {{ updater.percent }}%
  </span>
  <span v-else-if="updater.status === 'ready'" class="flex items-center gap-1.5" role="status">
    Pembaruan siap.
    <button type="button" :class="link" @click="updater.restart()">Mulai ulang</button>
  </span>
</template>
