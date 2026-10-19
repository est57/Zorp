<script setup lang="ts">
import type { RequestSpec } from "@/bindings/RequestSpec";
import { checkboxClass, inputClass } from "@/components/ui/styles";

const props = defineProps<{ spec: RequestSpec }>();

function setTimeoutMs(value: string) {
  const ms = Math.round(Number(value));
  if (Number.isFinite(ms)) props.spec.settings.timeoutMs = Math.min(600_000, Math.max(1, ms));
}
</script>

<template>
  <div class="grid max-w-xl gap-3 p-3 text-xs">
    <label class="grid grid-cols-[160px_120px] items-center gap-4 max-sm:grid-cols-1">
      <span class="label-caps">Batas waktu (ms)</span>
      <input
        type="number"
        min="1"
        max="600000"
        :value="spec.settings.timeoutMs"
        :class="inputClass"
        @change="setTimeoutMs(($event.target as HTMLInputElement).value)"
      />
    </label>
    <label class="flex items-center gap-2.5">
      <input v-model="spec.settings.followRedirects" type="checkbox" :class="checkboxClass" />
      Ikuti redirect secara otomatis (maksimal 10)
    </label>
    <label class="flex items-center gap-2.5">
      <input v-model="spec.settings.verifyTls" type="checkbox" :class="checkboxClass" />
      Verifikasi sertifikat TLS
    </label>
    <p v-if="!spec.settings.verifyTls" class="text-warning">
      Verifikasi TLS nonaktif. Gunakan hanya untuk server pengembangan yang Anda percaya.
    </p>
  </div>
</template>
