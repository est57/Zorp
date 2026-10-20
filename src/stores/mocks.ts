import { defineStore } from "pinia";
import { ref } from "vue";
import type { MockLogEntry } from "@/bindings/MockLogEntry";
import type { MockStatus } from "@/bindings/MockStatus";
import * as api from "@/api/mock";
import { isTauri } from "@/api/invoke";

const MAX_LOG = 200;

export const useMocksStore = defineStore("mocks", () => {
  const running = ref<MockStatus[]>([]);
  const log = ref<MockLogEntry[]>([]);
  let listening = false;

  function listen() {
    if (listening) return;
    listening = true;
    void api.onMockLog((entry) => {
      log.value.push(entry);
      if (log.value.length > MAX_LOG) log.value.splice(0, log.value.length - MAX_LOG);
    });
  }

  async function load() {
    if (!isTauri()) return;
    listen();
    running.value = await api.listMocks();
  }

  const statusOf = (collectionId: string) =>
    running.value.find((m) => m.collectionId === collectionId) ?? null;

  async function start(collectionId: string, port: number | null) {
    listen();
    const status = await api.startMock(collectionId, port);
    running.value = [...running.value.filter((m) => m.collectionId !== collectionId), status];
    return status;
  }

  async function stop(collectionId: string) {
    await api.stopMock(collectionId);
    running.value = running.value.filter((m) => m.collectionId !== collectionId);
  }

  return { running, log, load, statusOf, start, stop };
});
