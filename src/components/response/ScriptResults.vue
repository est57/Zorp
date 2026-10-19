<script setup lang="ts">
import type { ScriptReport } from "@/bindings/ScriptReport";

defineProps<{ report: ScriptReport; view: "tests" | "console" }>();
</script>

<template>
  <div v-if="view === 'tests'" class="grid gap-1 p-3 text-xs">
    <p v-if="!report.tests.length" class="text-muted">
      Belum ada tes. Tulis tes di tab Script, misalnya
      <code class="font-mono">pm.test("Status 200", ...)</code>.
    </p>
    <div
      v-for="(t, i) in report.tests"
      :key="i"
      class="grid grid-cols-[64px_minmax(0,1fr)] gap-2 border-b border-line py-1.5"
    >
      <span
        class="font-mono text-[11px] font-semibold"
        :class="t.passed ? 'text-success' : 'text-danger'"
      >
        {{ t.passed ? "LULUS" : "GAGAL" }}
      </span>
      <span class="grid min-w-0 gap-0.5">
        <span class="break-words">{{ t.name }}</span>
        <span v-if="t.error" class="font-mono break-words text-danger select-text">{{
          t.error
        }}</span>
      </span>
    </div>
  </div>
  <div v-else class="grid gap-0.5 p-3 font-mono text-xs select-text">
    <p v-if="!report.logs.length && !report.errors.length" class="font-sans text-muted">
      Tidak ada output. Gunakan <code class="font-mono">console.log()</code> di script untuk
      mencetak nilai.
    </p>
    <p v-for="(e, i) in report.errors" :key="`e${i}`" class="break-words text-danger">{{ e }}</p>
    <p
      v-for="(log, i) in report.logs"
      :key="i"
      class="break-words whitespace-pre-wrap"
      :class="{ 'text-danger': log.level === 'error', 'text-warning': log.level === 'warn' }"
    >
      {{ log.message }}
    </p>
  </div>
</template>
