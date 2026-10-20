import { computed } from "vue";
import { useCollectionsStore } from "@/stores/collections";
import { useWorkspacesStore } from "@/stores/workspaces";
import { authCredentials, requestCredentials } from "@/utils/credentials";

export interface CredentialWarning {
  id: string;
  name: string;
  places: string[];
}

/**
 * Credentials typed in directly in a workspace that is synced to a folder, where they
 * would end up in Git. Values stored as secret variables are not affected.
 */
export function useCredentialWarnings() {
  const collections = useCollectionsStore();
  const workspaces = useWorkspacesStore();

  const items = computed<CredentialWarning[]>(() => {
    if (!workspaces.active?.folder) return [];
    const tree = collections.tree;
    return [
      ...tree.collections.map((c) => ({ id: c.id, name: c.name, places: authCredentials(c.auth) })),
      ...tree.folders.map((f) => ({ id: f.id, name: f.name, places: authCredentials(f.auth) })),
      ...tree.requests.map((r) => ({
        id: r.spec.id,
        name: r.spec.name,
        places: requestCredentials(r.spec),
      })),
    ].filter((w) => w.places.length > 0);
  });

  const byId = computed(() => new Map(items.value.map((w) => [w.id, w])));

  /** Tooltip text for a tree row, or undefined when the node is clean. */
  function warningFor(id: string): string | undefined {
    const warning = byId.value.get(id);
    if (!warning) return undefined;
    return `Kredensial tertulis langsung dan ikut tersimpan di folder: ${warning.places.join(", ")}. Gunakan variabel rahasia, misalnya {{token}}.`;
  }

  return { items, warningFor };
}
