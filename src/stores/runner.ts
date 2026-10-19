import { defineStore } from "pinia";
import { reactive } from "vue";
import type { ProgressPhase } from "@/bindings/ProgressPhase";
import type { ResponseData } from "@/bindings/ResponseData";
import { cancelRequest, onProgress, sendRequest } from "@/api/http";
import { ApiError, toApiError } from "@/api/invoke";
import { cloneSpec, newId } from "@/utils/request";
import { useHistoryStore } from "./history";
import type { Tab } from "./tabs";

export type RunStatus = "idle" | "loading" | "done" | "error" | "cancelled";

export interface RunState {
  status: RunStatus;
  runId?: string;
  startedAt?: number;
  finishedAt?: number;
  phase?: ProgressPhase;
  bytes?: number;
  response?: ResponseData;
  error?: ApiError;
}

const IDLE: RunState = Object.freeze({ status: "idle" });

export const useRunnerStore = defineStore("runner", () => {
  const runs = reactive<Record<string, RunState>>({});
  const tabByRun = new Map<string, string>();
  let listening = false;

  function listen() {
    if (listening) return;
    listening = true;
    void onProgress((event) => {
      const tabId = tabByRun.get(event.requestId);
      const run = tabId ? runs[tabId] : undefined;
      if (run?.status !== "loading") return;
      run.phase = event.phase;
      run.bytes = event.bytes;
    });
  }

  const get = (tabId: string): RunState => runs[tabId] ?? IDLE;

  async function send(tab: Tab) {
    if (get(tab.id).status === "loading") return;
    listen();
    const runId = newId();
    tabByRun.set(runId, tab.id);
    runs[tab.id] = { status: "loading", runId, startedAt: Date.now(), phase: "sending", bytes: 0 };
    const current = () => runs[tab.id]?.runId === runId;
    try {
      const response = await sendRequest(runId, cloneSpec(tab.draft));
      if (current()) runs[tab.id] = { status: "done", runId, response, finishedAt: Date.now() };
    } catch (raw) {
      const error = toApiError(raw);
      if (current()) {
        const started = runs[tab.id].startedAt;
        runs[tab.id] =
          error.kind === "cancelled"
            ? { status: "cancelled", runId, startedAt: started, finishedAt: Date.now() }
            : { status: "error", runId, error, finishedAt: Date.now() };
      }
    } finally {
      tabByRun.delete(runId);
      void useHistoryStore().load();
    }
  }

  async function cancel(tabId: string) {
    const run = runs[tabId];
    if (run?.status === "loading" && run.runId) await cancelRequest(run.runId);
  }

  function forget(tabId: string) {
    delete runs[tabId];
  }

  return { get, send, cancel, forget };
});
