import { defineStore } from "pinia";
import { reactive } from "vue";
import type { RealtimeEvent } from "@/bindings/RealtimeEvent";
import { onRealtimeEvent, realtimeClose, realtimeConnect, realtimeSend } from "@/api/realtime";
import { toApiError } from "@/api/invoke";
import { cloneSpec, newId } from "@/utils/request";
import type { Tab } from "./tabs";

export type ConnectionStatus = "idle" | "connecting" | "open" | "closed";

export interface Connection {
  id: string;
  status: ConnectionStatus;
  events: RealtimeEvent[];
}

/** Older events are dropped so long sessions stay responsive. */
const MAX_EVENTS = 1000;

export const useRealtimeStore = defineStore("realtime", () => {
  const connections = reactive<Record<string, Connection>>({});
  const tabByConnection = new Map<string, string>();
  let listening = false;

  function push(tabId: string, event: RealtimeEvent) {
    const connection = connections[tabId];
    if (!connection) return;
    connection.events.push(event);
    if (connection.events.length > MAX_EVENTS)
      connection.events.splice(0, connection.events.length - MAX_EVENTS);
    if (event.kind === "open") connection.status = "open";
    if (event.kind === "close" || event.kind === "error") connection.status = "closed";
  }

  function listen() {
    if (listening) return;
    listening = true;
    void onRealtimeEvent((event) => {
      const tabId = tabByConnection.get(event.connectionId);
      if (tabId && connections[tabId]?.id === event.connectionId) push(tabId, event);
    });
  }

  const get = (tabId: string): Connection | undefined => connections[tabId];
  const isActive = (tabId: string) =>
    ["connecting", "open"].includes(connections[tabId]?.status ?? "idle");

  async function connect(tab: Tab) {
    listen();
    const id = newId();
    tabByConnection.set(id, tab.id);
    const previous = connections[tab.id]?.events ?? [];
    connections[tab.id] = { id, status: "connecting", events: previous };
    try {
      await realtimeConnect(id, cloneSpec(tab.draft));
    } catch (error) {
      const message = toApiError(error).message;
      push(tab.id, {
        connectionId: id,
        kind: "error",
        data: message,
        event: null,
        binary: false,
        at: Date.now(),
      });
    }
  }

  async function send(tabId: string, text: string) {
    const connection = connections[tabId];
    if (connection?.status === "open") await realtimeSend(connection.id, text);
  }

  async function disconnect(tabId: string) {
    const connection = connections[tabId];
    if (connection && isActive(tabId)) await realtimeClose(connection.id);
  }

  function clear(tabId: string) {
    if (connections[tabId]) connections[tabId].events = [];
  }

  return { get, isActive, connect, send, disconnect, clear };
});
