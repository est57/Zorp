// Development-only answers for newer commands in the ?mock preview.
import { examples } from "./mockData";

export function handleFeature(
  cmd: string,
  a: Record<string, unknown>,
): { handled: boolean; value?: unknown } {
  const value = (() => {
    switch (cmd) {
      case "preview_openapi":
        return {
          title: "Swagger Petstore",
          version: "1.0.27",
          format: "OpenAPI 3.0",
          endpoints: 19,
          groups: [
            { name: "pet", count: 8 },
            { name: "store", count: 4 },
            { name: "user", count: 7 },
          ],
          auth: ["petstore_auth", "api_key"],
          servers: ["https://petstore3.swagger.io/api/v3"],
          samples: [
            { method: "PUT", name: "Update an existing pet", group: "pet" },
            { method: "POST", name: "Add a new pet to the store", group: "pet" },
            { method: "GET", name: "Finds Pets by status", group: "pet" },
          ],
        };
      case "mock_routes":
        return [
          {
            method: "POST",
            path: "/login",
            requestName: "Login",
            example: "Login berhasil",
            status: 200,
          },
          {
            method: "GET",
            path: "/users",
            requestName: "Daftar user",
            example: null,
            status: null,
          },
        ];
      case "list_mocks":
        return [];
      case "start_mock":
        return { collectionId: a.collectionId, port: 4010, url: "http://127.0.0.1:4010" };
      case "stop_mock":
        return null;
      case "get_globals":
        return {
          id: "w1",
          name: "Global",
          isActive: false,
          variables: [
            { key: "appVersion", value: "2.4.0", enabled: true, secret: false, hasValue: false },
          ],
        };
      case "save_globals":
        return a.globals;
      case "get_collection_variables":
        return [["base_url", "https://api.contoh.id"]];
      case "set_node_settings":
      case "set_collection_variables":
        return null;
      case "get_example":
        return examples.find((e) => e.id === a.id);
      case "save_example":
        return { ...(a.example as object), id: "ex-new", sortOrder: 3 };
      case "preview_run_data":
        return {
          columns: ["email", "outletId", "login_password"],
          rows: [
            ["leader.a@outlet.contoh.id", "242", "rahasia-1"],
            ["leader.b@outlet.contoh.id", "118", "rahasia-2"],
            ["staff.c@outlet.contoh.id", "305", "rahasia-3"],
          ],
          total: 3,
        };
      default:
        return undefined;
    }
  })();
  return { handled: value !== undefined, value };
}
