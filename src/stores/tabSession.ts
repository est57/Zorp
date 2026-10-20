import { watch } from "vue";
import type { Tab } from "./tabs";
import { useTabsStore } from "./tabs";

interface SavedSession {
  tabs: Tab[];
  activeId: string | null;
}

const key = (workspaceId: string) => `zorp.tabs.${workspaceId}`;
const SAVE_DELAY_MS = 400;

/** Workspace whose tabs are on screen; null while switching, so nothing is saved then. */
let current: string | null = null;
let timer: ReturnType<typeof setTimeout> | undefined;

function isTab(value: unknown): value is Tab {
  const t = value as Tab;
  return (
    typeof t?.id === "string" &&
    typeof t.saved === "string" &&
    typeof t.draft?.url === "string" &&
    typeof t.draft?.method === "string"
  );
}

function read(workspaceId: string): SavedSession {
  try {
    const raw = JSON.parse(localStorage.getItem(key(workspaceId)) ?? "null");
    const tabs = Array.isArray(raw?.tabs) ? raw.tabs.filter(isTab) : [];
    return { tabs, activeId: typeof raw?.activeId === "string" ? raw.activeId : null };
  } catch {
    return { tabs: [], activeId: null };
  }
}

/** Writes the open tabs of the workspace on screen right away. */
export function saveTabSession() {
  clearTimeout(timer);
  if (!current) return;
  const tabs = useTabsStore();
  try {
    localStorage.setItem(
      key(current),
      JSON.stringify({ tabs: tabs.tabs, activeId: tabs.activeId } satisfies SavedSession),
    );
  } catch {
    // Storage may be full or unavailable; open tabs are a convenience only.
  }
}

/** Saves the tabs of the workspace being left and stops saving until the next restore. */
export function leaveTabSession() {
  saveTabSession();
  current = null;
}

/** Brings back the tabs last open in a workspace. */
export function restoreTabSession(workspaceId: string) {
  const session = read(workspaceId);
  useTabsStore().replace(session.tabs, session.activeId);
  current = workspaceId;
}

/** Saves open tabs shortly after they change. Call once from the app shell. */
export function watchTabSession() {
  const tabs = useTabsStore();
  watch(
    () => [tabs.tabs, tabs.activeId],
    () => {
      clearTimeout(timer);
      timer = setTimeout(saveTabSession, SAVE_DELAY_MS);
    },
    { deep: true },
  );
  window.addEventListener("beforeunload", saveTabSession);
}
