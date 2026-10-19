import { onBeforeUnmount, onMounted } from "vue";

type Handler = (event: KeyboardEvent) => void;

/** Registers shortcuts keyed as "mod+b", where mod is Cmd on macOS and Ctrl elsewhere. */
export function useShortcuts(map: Record<string, Handler>) {
  const isMac = navigator.userAgent.includes("Mac");

  function onKeydown(event: KeyboardEvent) {
    const mod = isMac ? event.metaKey : event.ctrlKey;
    if (!mod) return;
    // Ignore repeats so holding a shortcut does not send the same request many times.
    if (event.repeat) return;
    const key = `mod+${event.shiftKey ? "shift+" : ""}${event.key.toLowerCase()}`;
    const handler = map[key];
    if (!handler) return;
    event.preventDefault();
    handler(event);
  }

  onMounted(() => window.addEventListener("keydown", onKeydown));
  onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));
}
