<script setup lang="ts">
import { computed } from "vue";
import type { AssertOp } from "@/bindings/AssertOp";
import type { Assertion } from "@/bindings/Assertion";
import type { RequestSpec } from "@/bindings/RequestSpec";
import IconButton from "@/components/ui/IconButton.vue";
import VariableInput from "@/components/ui/VariableInput.vue";
import { checkboxClass, selectClass } from "@/components/ui/styles";
import { useRunnerStore } from "@/stores/runner";

const props = defineProps<{ spec: RequestSpec }>();
const runner = useRunnerStore();

const OPS: { op: AssertOp; label: string }[] = [
  { op: "eq", label: "sama dengan" },
  { op: "neq", label: "tidak sama dengan" },
  { op: "gt", label: "lebih dari" },
  { op: "gte", label: "minimal" },
  { op: "lt", label: "kurang dari" },
  { op: "lte", label: "maksimal" },
  { op: "contains", label: "berisi" },
  { op: "not_contains", label: "tidak berisi" },
  { op: "matches", label: "cocok regex" },
  { op: "exists", label: "ada" },
  { op: "not_exists", label: "tidak ada" },
  { op: "is_type", label: "bertipe" },
];
const noValue = (op: AssertOp) => op === "exists" || op === "not_exists";

const rows = computed<Assertion[]>(() => props.spec.asserts ?? []);
const draft = (): Assertion => ({ enabled: true, expr: "", op: "eq", value: "" });

/**
 * Results of the last send. Assertion results are named "<ekspresi> <operator> <nilai>", so
 * each enabled row takes the next result with its expression and operator.
 */
const results = computed(() => {
  const run = runner.get(props.spec.id);
  const tests = run.status === "done" ? [...(run.response?.script.tests ?? [])] : [];
  const map = new Map<Assertion, (typeof tests)[number]>();
  for (const row of rows.value.filter((r) => r.enabled && r.expr.trim())) {
    const label = OPS.find((o) => o.op === row.op)?.label ?? "";
    const index = tests.findIndex((t) => t.name.startsWith(`${row.expr.trim()} ${label}`));
    if (index >= 0) map.set(row, tests.splice(index, 1)[0]);
  }
  return map;
});

function update(index: number, patch: Partial<Assertion>) {
  const next = [...rows.value];
  if (index === next.length) next.push({ ...draft(), ...patch });
  else next[index] = { ...next[index], ...patch };
  props.spec.asserts = next;
}

const remove = (index: number) => (props.spec.asserts = rows.value.filter((_, i) => i !== index));
const cell = "h-8 border-b border-line px-2";
</script>

<template>
  <div class="text-xs">
    <table class="w-full table-fixed border-collapse">
      <thead class="bg-surface-2 text-[11px] tracking-wide text-subtle uppercase">
        <tr>
          <th class="w-8"></th>
          <th class="px-2 py-1 text-left font-medium">Ekspresi</th>
          <th class="w-40 px-2 py-1 text-left font-medium">Operator</th>
          <th class="px-2 py-1 text-left font-medium">Nilai</th>
          <th class="w-40 px-2 py-1 text-left font-medium max-md:hidden">Hasil</th>
          <th class="w-9"></th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="(row, i) in [...rows, draft()]"
          :key="i"
          :class="i === rows.length && 'opacity-70'"
        >
          <td :class="cell">
            <input
              v-if="i < rows.length"
              type="checkbox"
              :checked="row.enabled"
              :class="checkboxClass"
              aria-label="Aktifkan assert"
              @change="update(i, { enabled: ($event.target as HTMLInputElement).checked })"
            />
          </td>
          <td :class="cell">
            <div class="flex h-7">
              <VariableInput
                :model-value="row.expr"
                label="Ekspresi"
                placeholder="res.status"
                @update:model-value="(v) => update(i, { expr: v })"
              />
            </div>
          </td>
          <td :class="cell">
            <select
              :value="row.op"
              :class="[selectClass, 'w-full']"
              aria-label="Operator"
              @change="update(i, { op: ($event.target as HTMLSelectElement).value as AssertOp })"
            >
              <option v-for="o in OPS" :key="o.op" :value="o.op">{{ o.label }}</option>
            </select>
          </td>
          <td :class="cell">
            <div v-if="!noValue(row.op)" class="flex h-7">
              <VariableInput
                :model-value="row.value"
                label="Nilai"
                :placeholder="row.op === 'is_type' ? 'string' : '200'"
                @update:model-value="(v) => update(i, { value: v })"
              />
            </div>
          </td>
          <td :class="[cell, 'truncate max-md:hidden']">
            <template v-if="results.get(row)">
              <span v-if="results.get(row)?.passed" class="text-success">Lulus</span>
              <span v-else class="text-danger" :title="results.get(row)?.error ?? ''"
                >Gagal: {{ results.get(row)?.error }}</span
              >
            </template>
          </td>
          <td :class="cell">
            <IconButton v-if="i < rows.length" icon="x" label="Hapus assert" @click="remove(i)" />
          </td>
        </tr>
      </tbody>
    </table>
    <p class="px-3 py-2 text-[11px] leading-4 text-subtle">
      Ekspresi: res.status, res.responseTime, res.body.path.ke[0].field, dan res.headers['nama'].
      Nilai boleh memakai variabel. Hasil assert juga tampil di tab Tes pada panel respons dan di
      collection runner.
    </p>
  </div>
</template>
