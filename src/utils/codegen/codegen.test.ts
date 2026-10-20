import { describe, expect, it } from "vitest";
import type { RequestSpec } from "@/bindings/RequestSpec";
import { parseCurl } from "@/utils/curl/parse";
import { newRequest } from "@/utils/request";
import { SNIPPET_TARGETS, toSnippetRequest } from "./index";
import { toCurl } from "./curl";

const identity = (text: string) => text;
const vars: Record<string, string> = { base: "https://api.test" };
const resolve = (text: string) => text.replace(/\{\{(\w+)\}\}/g, (m, n) => vars[n] ?? m);

const spec = (overrides: Partial<RequestSpec>) =>
  newRequest({ method: "POST", url: "{{base}}/users?page=1", ...overrides });

describe("code generation", () => {
  it("round-trips a JSON request through cURL", () => {
    const req = toSnippetRequest(
      spec({ body: { type: "json", text: `{"name":"O'Neil"}` } }),
      { type: "bearer", token: "{{token}}" },
      resolve,
    );
    const parsed = parseCurl(toCurl(req));
    expect(parsed.method).toBe("POST");
    expect(parsed.url).toBe("https://api.test/users?page=1");
    expect(parsed.auth).toEqual({ type: "bearer", token: "{{token}}" });
    expect(parsed.body).toEqual({ type: "json", text: `{"name":"O'Neil"}` });
  });

  it("round-trips form and multipart bodies", () => {
    const form = toSnippetRequest(
      spec({
        body: {
          type: "form_url_encoded",
          fields: [{ key: "q", value: "a b&c", enabled: true, description: "" }],
        },
      }),
      { type: "none" },
      identity,
    );
    expect(parseCurl(toCurl(form)).body).toEqual({
      type: "form_url_encoded",
      fields: [{ key: "q", value: "a b&c", enabled: true, description: "" }],
    });

    const multi = toSnippetRequest(
      spec({
        body: {
          type: "multipart",
          parts: [{ type: "file", key: "f", path: "/tmp/a b.png", enabled: true }],
        },
      }),
      { type: "none" },
      identity,
    );
    expect(parseCurl(toCurl(multi)).body).toEqual({
      type: "multipart",
      parts: [{ type: "file", key: "f", path: "/tmp/a b.png", enabled: true }],
    });
  });

  it("applies api key auth to the query and generates every target", () => {
    const req = toSnippetRequest(
      newRequest({ url: "https://x.test/a" }),
      { type: "api_key", key: "api_key", value: "k 1", location: "query" },
      identity,
    );
    expect(req.url).toBe("https://x.test/a?api_key=k%201");
    for (const target of SNIPPET_TARGETS)
      expect(target.generate(req)).toContain("https://x.test/a?api_key=k%201");
  });
});

describe("graphql snippets", () => {
  it("sends the query and parsed variables as JSON", () => {
    const req = toSnippetRequest(
      newRequest({
        method: "POST",
        url: "{{base}}/graphql",
        body: {
          type: "graphql",
          query: "query($id: ID!) { user(id: $id) { name } }",
          variables: '{"id": 7}',
        },
      }),
      { type: "none" },
      resolve,
    );
    expect(req.headers).toContainEqual(["Content-Type", "application/json"]);
    expect(req.body).toEqual({
      kind: "text",
      text: JSON.stringify({
        query: "query($id: ID!) { user(id: $id) { name } }",
        variables: { id: 7 },
      }),
    });
    expect(parseCurl(toCurl(req)).body.type).toBe("json");
  });
});
