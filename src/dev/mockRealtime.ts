// Development-only simulation of WebSocket and SSE connections for the ?mock preview.
import { emit } from "@tauri-apps/api/event";
import type { RealtimeEvent } from "@/bindings/RealtimeEvent";
import type { RequestSpec } from "@/bindings/RequestSpec";

const timers = new Map<string, ReturnType<typeof setInterval>>();

function send(
  connectionId: string,
  kind: RealtimeEvent["kind"],
  data: string,
  event: string | null = null,
) {
  const payload: RealtimeEvent = { connectionId, kind, data, event, binary: false, at: Date.now() };
  void emit("realtime://event", payload);
}

export function handleRealtime(
  cmd: string,
  args: Record<string, unknown>,
): { handled: boolean; value?: unknown } {
  const id = args.connectionId as string;
  switch (cmd) {
    case "realtime_connect": {
      const spec = args.spec as RequestSpec;
      setTimeout(() => {
        send(id, "open", "Terhubung (HTTP 101)");
        if (spec.mode === "websocket") {
          send(id, "message", JSON.stringify({ type: "welcome", users: 3 }));
        } else {
          let n = 0;
          timers.set(
            id,
            setInterval(
              () => send(id, "message", JSON.stringify({ price: 189 + (n++ % 5) }), "tick"),
              1000,
            ),
          );
        }
      }, 300);
      return { handled: true, value: null };
    }
    case "realtime_send":
      send(id, "sent", args.text as string);
      setTimeout(() => send(id, "message", `echo: ${args.text}`), 150);
      return { handled: true, value: null };
    case "realtime_close":
      clearInterval(timers.get(id));
      timers.delete(id);
      send(id, "close", "Koneksi ditutup");
      return { handled: true, value: null };
    default:
      return { handled: false };
  }
}
