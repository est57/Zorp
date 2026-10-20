// Development-only backend mock for previewing the UI in a plain browser (?mock in the URL).
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { RequestSpec } from "@/bindings/RequestSpec";
import { body, env, history, tree, workspace, workspaces } from "./mockData";
import { handleRealtime } from "./mockRealtime";

export function installMockBackend() {
  mockWindows("main");
  mockIPC(
    async (cmd, args) => {
      const a = (args ?? {}) as Record<string, unknown>;
      const realtime = handleRealtime(cmd, a);
      if (realtime.handled) return realtime.value;
      switch (cmd) {
        case "app_info":
          return { version: "0.1.0", schemaVersion: 1, workspace };
        case "list_workspaces":
          return { activeId: workspace.id, workspaces };
        case "sync_status":
          return { folder: workspace.folder, problems: [] };
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
          const t = a.target as {
            collectionId: string;
            folderId: string | null;
            sortOrder: number;
          };
          const node =
            a.kind === "folder"
              ? tree.folders.find((f) => f.id === a.id)
              : tree.requests.find((r) => r.spec.id === a.id);
          if (node && "parentId" in node)
            Object.assign(node, { collectionId: t.collectionId, parentId: t.folderId });
          else if (node)
            Object.assign(node, { collectionId: t.collectionId, folderId: t.folderId });
          if (node) node.sortOrder = t.sortOrder;
          return null;
        }
        case "list_cookies":
          return [
            {
              domain: "api.contoh.dev",
              path: "/",
              name: "session",
              value: "abc123",
              expiresAt: null,
              secure: true,
              httpOnly: true,
              hostOnly: true,
            },
            {
              domain: "contoh.dev",
              path: "/",
              name: "theme",
              value: "dark",
              expiresAt: Date.now() + 86_400_000,
              secure: false,
              httpOnly: false,
              hostOnly: false,
            },
          ];
        case "run_collection":
          return { total: 3, passed: 2, failed: 1, durationMs: 412, cancelled: false };
        case "cancel_run":
          return null;
        case "get_settings":
          return {
            proxy: {
              enabled: false,
              url: "",
              username: "",
              password: "",
              hasPassword: false,
              bypass: "localhost",
            },
            caCertificates: [],
            clientCertificates: [],
          };
        case "save_settings":
          return a.settings;
        case "delete_cookie":
        case "clear_cookies":
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
            timing: { dnsMs: 12, connectMs: 48, ttfbMs: 120, downloadMs: 22, totalMs: 142 },
            httpVersion: "HTTP/2.0",
            url: spec.url,
            script: {
              tests: spec.scripts.test.trim()
                ? [
                    { name: "Status 200", passed: true, error: null },
                    {
                      name: "Respons di bawah 100 ms",
                      passed: false,
                      error: "expected 142 to be below 100",
                    },
                  ]
                : [],
              logs: spec.scripts.test.trim() ? [{ level: "log", message: "total user: 47" }] : [],
              errors: [],
              variablesChanged: false,
            },
          };
        }
        default:
          throw { kind: "internal", message: `Mock belum mendukung ${cmd}`, names: [] };
      }
    },
    { shouldMockEvents: true },
  );
}
