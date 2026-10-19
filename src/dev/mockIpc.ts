// Development-only backend mock for previewing the UI in a plain browser (?mock in the URL).
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { CollectionTree } from "@/bindings/CollectionTree";
import type { Environment } from "@/bindings/Environment";
import type { HistoryItem } from "@/bindings/HistoryItem";
import type { RequestSpec } from "@/bindings/RequestSpec";
import { newRequest } from "@/utils/request";

const workspace = { id: "w1", name: "Workspace Contoh", createdAt: Date.now() };
const env: Environment = {
  id: "e1",
  name: "Development",
  isActive: true,
  variables: [
    {
      key: "base_url",
      value: "https://api.contoh.dev/v1",
      enabled: true,
      secret: false,
      hasValue: false,
    },
    { key: "token", value: "", enabled: true, secret: true, hasValue: true },
  ],
};
const sample = (
  name: string,
  method: RequestSpec["method"],
  url: string,
  extra: Partial<RequestSpec> = {},
) => newRequest({ name, method, url, auth: { type: "bearer", token: "{{token}}" }, ...extra });

const tree: CollectionTree = {
  collections: [{ id: "c1", name: "API Contoh", auth: null, sortOrder: 1 }],
  folders: [
    { id: "f1", collectionId: "c1", parentId: null, name: "Users", auth: null, sortOrder: 1 },
  ],
  requests: [
    {
      collectionId: "c1",
      folderId: "f1",
      sortOrder: 1,
      spec: sample("Daftar user", "GET", "{{base_url}}/users?page=2&limit=20"),
    },
    {
      collectionId: "c1",
      folderId: "f1",
      sortOrder: 2,
      spec: sample("Buat user", "POST", "{{base_url}}/users", {
        body: { type: "json", text: '{\n  "name": "Rani Wijaya",\n  "role": "admin"\n}' },
      }),
    },
    {
      collectionId: "c1",
      folderId: "f1",
      sortOrder: 3,
      spec: sample("Hapus user", "DELETE", "{{base_url}}/users/{{user_id}}"),
    },
  ],
};
const history: HistoryItem[] = tree.requests.map((r, i) => ({
  id: `h${i}`,
  request: r.spec,
  status: [200, 201, 404][i],
  durationMs: 140 + i * 37,
  sizeBytes: 1240,
  sentAt: Date.now() - i * 3_600_000,
}));

const body = JSON.stringify({
  data: [{ id: 41, name: "Rani Wijaya", active: true }],
  meta: { page: 2, total: 47 },
});

export function installMockBackend() {
  mockWindows("main");
  mockIPC(async (cmd, args) => {
    const a = (args ?? {}) as Record<string, unknown>;
    switch (cmd) {
      case "app_info":
        return { version: "0.1.0", schemaVersion: 1, workspace };
      case "list_tree":
        return structuredClone(tree);
      case "list_environments":
        return [env];
      case "list_history":
        return { items: history, nextCursor: null };
      case "set_node_auth": {
        const node = [...tree.collections, ...tree.folders].find((n) => n.id === a.id);
        if (node) node.auth = a.auth as never;
        return null;
      }
      case "move_node": {
        const t = a.target as { collectionId: string; folderId: string | null; sortOrder: number };
        const node =
          a.kind === "folder"
            ? tree.folders.find((f) => f.id === a.id)
            : tree.requests.find((r) => r.spec.id === a.id);
        if (node && "parentId" in node)
          Object.assign(node, { collectionId: t.collectionId, parentId: t.folderId });
        else if (node) Object.assign(node, { collectionId: t.collectionId, folderId: t.folderId });
        if (node) node.sortOrder = t.sortOrder;
        return null;
      }
      case "set_active_environment":
      case "plugin:event|listen":
        return null;
      case "send_request": {
        const spec = a.spec as RequestSpec;
        if (spec.url.includes("{{user_id}}")) {
          throw {
            kind: "unresolved_variable",
            message: "Variabel tidak ditemukan: user_id",
            names: ["user_id"],
          };
        }
        await new Promise((r) => setTimeout(r, 600));
        return {
          status: 200,
          statusText: "OK",
          headers: [
            ["content-type", "application/json"],
            ["set-cookie", "session=abc; HttpOnly; Secure"],
          ],
          body: { type: "text", text: body },
          sizeBytes: body.length,
          timing: { ttfbMs: 120, downloadMs: 22, totalMs: 142 },
          httpVersion: "HTTP/2.0",
          url: spec.url,
        };
      }
      default:
        throw { kind: "internal", message: `Mock belum mendukung ${cmd}`, names: [] };
    }
  });
}
