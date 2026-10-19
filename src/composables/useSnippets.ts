import type { Auth } from "@/bindings/Auth";
import type { RequestSpec } from "@/bindings/RequestSpec";
import { useCollectionsStore } from "@/stores/collections";
import { useEnvironmentsStore } from "@/stores/environments";
import { type SnippetRequest, toSnippetRequest } from "@/utils/codegen";

export function useSnippets() {
  const collections = useCollectionsStore();
  const envs = useEnvironmentsStore();

  /** Auth that applies to a request, following folder and collection inheritance. */
  function effectiveAuth(
    spec: RequestSpec,
    collectionId: string | null,
    folderId: string | null,
  ): Auth {
    if (spec.auth.type !== "inherit") return spec.auth;
    let current = folderId;
    while (current) {
      const folder = collections.tree.folders.find((f) => f.id === current);
      if (folder?.auth && folder.auth.type !== "inherit") return folder.auth;
      current = folder?.parentId ?? null;
    }
    return (
      collections.tree.collections.find((c) => c.id === collectionId)?.auth ?? { type: "none" }
    );
  }

  /** Builds a snippet model; secrets always stay as {{variables}}. */
  function build(
    spec: RequestSpec,
    collectionId: string | null,
    folderId: string | null,
    resolve: boolean,
  ): SnippetRequest {
    const values = new Map(
      (envs.active?.variables ?? [])
        .filter((v) => v.enabled && !v.secret)
        .map((v) => [v.key, v.value]),
    );
    const replace = (text: string) =>
      resolve
        ? text.replace(
            /\{\{\s*([^{}\s]+)\s*\}\}/g,
            (match, name: string) => values.get(name) ?? match,
          )
        : text;
    return toSnippetRequest(spec, effectiveAuth(spec, collectionId, folderId), replace);
  }

  return { build, effectiveAuth };
}
