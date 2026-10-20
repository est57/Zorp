import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { Environment } from "@/bindings/Environment";
import * as api from "@/api/environments";
import { isTauri } from "@/api/invoke";

export const useEnvironmentsStore = defineStore("environments", () => {
  const items = ref<Environment[]>([]);
  /** Workspace-wide variables, below every other layer. */
  const globals = ref<Environment | null>(null);

  const active = computed(() => items.value.find((e) => e.isActive) ?? null);

  /** Names that resolve from the globals or the active environment, for variable highlighting. */
  const names = computed(
    () =>
      new Set(
        [...(globals.value?.variables ?? []), ...(active.value?.variables ?? [])]
          .filter((v) => v.enabled && (!v.secret || v.hasValue))
          .map((v) => v.key),
      ),
  );

  async function load() {
    if (!isTauri()) return;
    [items.value, globals.value] = await Promise.all([api.listEnvironments(), api.getGlobals()]);
  }

  async function saveGlobals(next: Environment) {
    globals.value = await api.saveGlobals(next);
    return globals.value;
  }

  async function save(environment: Environment) {
    const saved = await api.saveEnvironment(environment);
    const index = items.value.findIndex((e) => e.id === saved.id);
    if (index >= 0) items.value[index] = saved;
    else items.value.push(saved);
    items.value.sort((a, b) => a.name.localeCompare(b.name));
    return saved;
  }

  async function remove(id: string) {
    await api.deleteEnvironment(id);
    items.value = items.value.filter((e) => e.id !== id);
  }

  async function activate(id: string | null) {
    await api.setActiveEnvironment(id);
    items.value.forEach((e) => (e.isActive = e.id === id));
  }

  return { items, globals, active, names, load, save, saveGlobals, remove, activate };
});
