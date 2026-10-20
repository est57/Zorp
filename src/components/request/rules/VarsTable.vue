<script setup lang="ts">
import { computed, ref } from "vue";
import type { RequestSpec } from "@/bindings/RequestSpec";
import type { VarPhase } from "@/bindings/VarPhase";
import type { VarRule } from "@/bindings/VarRule";
import type { VarTarget } from "@/bindings/VarTarget";
import IconButton from "@/components/ui/IconButton.vue";
import VariableInput from "@/components/ui/VariableInput.vue";
import { checkboxClass, selectClass } from "@/components/ui/styles";
import { useRunnerStore } from "@/stores/runner";

const props = defineProps<{ spec: RequestSpec }>();
const runner = useRunnerStore();
const phase = ref<VarPhase>(props.spec.vars?.some((v) => v.phase === "post") ? "post" : "pre");

const all = computed<VarRule[]>(() => props.spec.vars ?? []);
const rows = computed(() => all.value.filter((r) => r.phase === phase.value));
const draft = (): VarRule => ({
  enabled: true,
  phase: phase.value,
  name: "",
  expr: "",
  target: "collection",
});

function update(row: VarRule | null, patch: Partial<VarRule>) {
  props.spec.vars = row
    ? all.value.map((r) => (r === row ? { ...r, ...patch } : r))
    : [...all.value, { ...draft(), ...patch }];
}

/** Values read on the last send; each enabled row takes the next capture with its name. */
const captured = computed(() => {
  const run = runner.get(props.spec.id);
  const list = run.status === "done" ? [...(run.response?.script.captured ?? [])] : [];
  const map = new Map<VarRule, string | null>();
  for (const row of all.value.filter((r) => r.enabled && r.phase === "post" && r.name.trim())) {
    const index = list.findIndex((c) => c.name === row.name.trim());
    if (index >= 0) map.set(row, list.splice(index, 1)[0].value);
  }
  return map;
});
const short = (value: string) => (value.length > 32 ? `${value.slice(0, 32)}…` : value);

const remove = (row: VarRule) => (props.spec.vars = all.value.filter((r) => r !== row));
const segment = (active: boolean) =>
  `rounded px-2.5 py-0.5 font-medium ${active ? "bg-accent-soft text-accent" : "text-muted hover:text-fg"}`;
const cell = "h-8 border-b border-line px-2";
</script>

<template>
  <div class="text-xs">
    <div class="flex items-center gap-3 px-3 py-2">
      <div
        class="flex gap-0.5 rounded-md border border-line p-0.5"
        role="group"
        aria-label="Waktu variabel"
      >
        <button type="button" :class="segment(phase === 'pre')" @click="phase = 'pre'">
          Sebelum request
        </button>
        <button type="button" :class="segment(phase === 'post')" @click="phase = 'post'">
          Setelah respons
        </button>
      </div>
    </div>
    <table class="w-full table-fixed border-collapse">
      <thead class="bg-surface-2 text-[11px] tracking-wide text-subtle uppercase">
        <tr>
          <th class="w-8"></th>
          <th class="w-48 px-2 py-1 text-left font-medium">Nama variabel</th>
          <th class="px-2 py-1 text-left font-medium">
            {{ phase === "pre" ? "Nilai" : "Ambil dari" }}
          </th>
          <th v-if="phase === 'post'" class="w-44 px-2 py-1 text-left font-medium">Simpan ke</th>
          <th v-if="phase === 'post'" class="w-48 px-2 py-1 text-left font-medium max-md:hidden">
            Nilai terakhir
          </th>
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
              aria-label="Aktifkan variabel"
              @change="update(row, { enabled: ($event.target as HTMLInputElement).checked })"
            />
          </td>
          <td :class="cell">
            <div class="flex h-7">
              <VariableInput
                :model-value="row.name"
                label="Nama variabel"
                placeholder="token"
                @update:model-value="(v) => update(i < rows.length ? row : null, { name: v })"
              />
            </div>
          </td>
          <td :class="cell">
            <div class="flex h-7">
              <VariableInput
                :model-value="row.expr"
                :label="phase === 'pre' ? 'Nilai' : 'Ekspresi respons'"
                :placeholder="phase === 'pre' ? '{{userId}}' : 'res.body.token'"
                @update:model-value="(v) => update(i < rows.length ? row : null, { expr: v })"
              />
            </div>
          </td>
          <td v-if="phase === 'post'" :class="cell">
            <select
              :value="row.target"
              :class="[selectClass, 'w-full']"
              aria-label="Simpan ke"
              @change="
                update(i < rows.length ? row : null, {
                  target: ($event.target as HTMLSelectElement).value as VarTarget,
                })
              "
            >
              <option value="collection">Variabel koleksi</option>
              <option value="environment">Environment aktif</option>
            </select>
          </td>
          <td v-if="phase === 'post'" :class="[cell, 'truncate font-mono max-md:hidden']">
            <template v-if="captured.has(row)">
              <span v-if="captured.get(row) === null" class="font-sans text-danger"
                >Tidak ditemukan</span
              >
              <span v-else class="text-muted select-text" :title="captured.get(row) ?? ''">{{
                short(captured.get(row) ?? "")
              }}</span>
            </template>
          </td>
          <td :class="cell">
            <IconButton
              v-if="i < rows.length"
              icon="x"
              label="Hapus variabel"
              @click="remove(row)"
            />
          </td>
        </tr>
      </tbody>
    </table>
    <p class="px-3 py-2 text-[11px] leading-4 text-subtle">
      {{
        phase === "pre"
          ? "Variabel ini hanya berlaku untuk request ini, setara pm.variables.set, dan diisi setelah script pre-request."
          : "Nilai diambil dari respons sebelum test script berjalan, lalu disimpan ke variabel koleksi atau environment aktif."
      }}
    </p>
  </div>
</template>
