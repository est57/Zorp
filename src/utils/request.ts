import type { HttpMethod } from "@/bindings/HttpMethod";
import type { KeyValue } from "@/bindings/KeyValue";
import type { RequestSpec } from "@/bindings/RequestSpec";

export const METHODS: HttpMethod[] = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

export const newId = () => crypto.randomUUID();

export const emptyRow = (): KeyValue => ({ key: "", value: "", enabled: true, description: "" });

export function newRequest(overrides: Partial<RequestSpec> = {}): RequestSpec {
  return {
    id: newId(),
    name: "Request baru",
    method: "GET",
    url: "",
    params: [],
    headers: [],
    body: { type: "none" },
    auth: { type: "inherit" },
    settings: { timeoutMs: 30000, followRedirects: true, verifyTls: true },
    ...overrides,
  };
}

// JSON round-trip instead of structuredClone, which rejects Vue reactive proxies.
export const cloneJson = <T>(value: T): T => JSON.parse(JSON.stringify(value));

export const cloneSpec = (spec: RequestSpec): RequestSpec => cloneJson(spec);

export const snapshot = (spec: RequestSpec) => JSON.stringify(spec);

export const methodLabel = (method: HttpMethod) =>
  method === "DELETE" ? "DEL" : method === "OPTIONS" ? "OPT" : method;

export const methodColor = (method: HttpMethod) =>
  (
    ({
      GET: "text-[var(--get)]",
      POST: "text-[var(--post)]",
      PUT: "text-[var(--put)]",
      PATCH: "text-[var(--patch)]",
      DELETE: "text-[var(--delete)]",
      HEAD: "text-muted",
      OPTIONS: "text-muted",
    }) as const
  )[method];
