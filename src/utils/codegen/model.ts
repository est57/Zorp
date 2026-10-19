import type { Auth } from "@/bindings/Auth";
import type { HttpMethod } from "@/bindings/HttpMethod";
import type { RequestSpec } from "@/bindings/RequestSpec";

export type SnippetBody =
  | { kind: "none" }
  | { kind: "text"; text: string }
  | { kind: "form"; fields: [string, string][] }
  | { kind: "multipart"; parts: { key: string; value?: string; file?: string }[] }
  | { kind: "file"; path: string };

export interface SnippetRequest {
  method: HttpMethod;
  url: string;
  headers: [string, string][];
  body: SnippetBody;
}

const base64 = (text: string) => btoa(String.fromCharCode(...new TextEncoder().encode(text)));

/** Flattens a request and its effective auth into what code generators need. */
export function toSnippetRequest(
  spec: RequestSpec,
  auth: Auth,
  resolve: (text: string) => string,
): SnippetRequest {
  const r = (pairs: { key: string; value: string; enabled: boolean }[]) =>
    pairs.filter((p) => p.enabled && p.key).map((p): [string, string] => [r1(p.key), r1(p.value)]);
  const r1 = resolve;

  // The URL already contains every enabled query param.
  let url = r1(spec.url);
  const headers = r(spec.headers);
  const hasHeader = (name: string) => headers.some(([k]) => k.toLowerCase() === name.toLowerCase());

  if (auth.type === "bearer") headers.push(["Authorization", `Bearer ${r1(auth.token)}`]);
  if (auth.type === "basic")
    headers.push(["Authorization", `Basic ${base64(`${r1(auth.username)}:${r1(auth.password)}`)}`]);
  // OAuth tokens live in the keychain and are never written into snippets.
  if (auth.type === "oauth2") headers.push(["Authorization", "Bearer <access_token>"]);
  if (auth.type === "api_key" && auth.location === "header")
    headers.push([r1(auth.key), r1(auth.value)]);
  if (auth.type === "api_key" && auth.location === "query") {
    url += `${url.includes("?") ? "&" : "?"}${encodeURIComponent(r1(auth.key))}=${encodeURIComponent(r1(auth.value))}`;
  }

  const body = spec.body;
  let snippetBody: SnippetBody = { kind: "none" };
  if (body.type === "json" || body.type === "raw") {
    const contentType = body.type === "json" ? "application/json" : body.contentType;
    if (!hasHeader("content-type")) headers.push(["Content-Type", contentType]);
    snippetBody = { kind: "text", text: r1(body.text) };
  } else if (body.type === "form_url_encoded") {
    if (!hasHeader("content-type"))
      headers.push(["Content-Type", "application/x-www-form-urlencoded"]);
    snippetBody = { kind: "form", fields: r(body.fields) };
  } else if (body.type === "multipart") {
    const parts = body.parts
      .filter((p) => p.enabled && p.key)
      .map((p) =>
        p.type === "file"
          ? { key: r1(p.key), file: p.path }
          : { key: r1(p.key), value: r1(p.value) },
      );
    snippetBody = { kind: "multipart", parts };
  } else if (body.type === "binary") {
    snippetBody = { kind: "file", path: body.path };
  }

  return { method: spec.method, url, headers, body: snippetBody };
}
