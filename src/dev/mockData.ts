// Sample data for the development-only mock backend.
import type { CollectionTree } from "@/bindings/CollectionTree";
import type { Environment } from "@/bindings/Environment";
import type { HistoryItem } from "@/bindings/HistoryItem";
import type { RequestSpec } from "@/bindings/RequestSpec";
import { newRequest } from "@/utils/request";

export const workspace = {
  id: "w1",
  name: "Workspace Contoh",
  createdAt: Date.now(),
  folder: "/Users/contoh/api-tim",
};
export const workspaces = [
  workspace,
  { id: "w2", name: "Proyek Pribadi", createdAt: Date.now(), folder: null },
];
export const env: Environment = {
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

export const examples = [
  {
    id: "ex-200",
    requestId: "r-login",
    name: "Login berhasil",
    status: 200,
    statusText: "OK",
    headers: [["content-type", "application/json"]] as [string, string][],
    body: '{\n  "token": "contoh-token",\n  "user": { "id": 41, "name": "Rani Wijaya" }\n}',
    sortOrder: 1,
  },
  {
    id: "ex-401",
    requestId: "r-login",
    name: "Password salah",
    status: 401,
    statusText: "Unauthorized",
    headers: [["content-type", "application/json"]] as [string, string][],
    body: '{\n  "message": "Email atau password salah"\n}',
    sortOrder: 2,
  },
];

export const tree: CollectionTree = {
  examples: examples.map(({ id, requestId, name, status }) => ({ id, requestId, name, status })),
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
    {
      collectionId: "c1",
      folderId: null,
      sortOrder: 1,
      spec: sample("Login", "POST", "{{base_url}}/login", {
        id: "r-login",
        auth: { type: "none" },
        body: {
          type: "json",
          text: '{\n  "email": "rani@contoh.id",\n  "password": "{{login_password}}"\n}',
        },
      }),
    },
  ],
};
export const history: HistoryItem[] = tree.requests.map((r, i) => ({
  id: `h${i}`,
  request: r.spec,
  status: [200, 201, 404][i],
  durationMs: 140 + i * 37,
  sizeBytes: 1240,
  sentAt: Date.now() - i * 3_600_000,
}));

export const body = JSON.stringify({
  data: [{ id: 41, name: "Rani Wijaya", active: true }],
  meta: { page: 2, total: 47 },
});
