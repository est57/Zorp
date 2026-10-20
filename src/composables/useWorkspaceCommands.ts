import { computed } from "vue";
import { useWorkspacesStore } from "@/stores/workspaces";
import type { Command } from "./useCommands";
import { useWorkspaceActions } from "./useWorkspaceActions";

/** Palette commands for switching, creating and opening workspaces. */
export function useWorkspaceCommands() {
  const workspaces = useWorkspacesStore();
  const actions = useWorkspaceActions();

  return computed<Command[]>(() => [
    ...workspaces.items
      .filter((w) => w.id !== workspaces.activeId)
      .map((w): Command => ({
        id: `workspace:${w.id}`,
        group: "Workspace",
        label: `Pindah ke workspace ${w.name}`,
        detail: w.folder ?? undefined,
        run: () => actions.switchTo(w.id),
      })),
    { id: "workspace-new", group: "Workspace", label: "Workspace baru", run: actions.create },
    {
      id: "workspace-open",
      group: "Workspace",
      label: "Buka folder workspace",
      run: actions.openFolder,
    },
  ]);
}
