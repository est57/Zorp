import type { Auth } from "@/bindings/Auth";
import type { Body } from "@/bindings/Body";
import type { HttpMethod } from "@/bindings/HttpMethod";
import type { KeyValue } from "@/bindings/KeyValue";
import type { MultipartPart } from "@/bindings/MultipartPart";
import { METHODS } from "@/utils/request";
import { parseQuery } from "@/utils/url";
import { type Option, readOptions } from "./options";
import { tokenize } from "./tokenize";

export interface CurlRequest {
  method: HttpMethod;
  url: string;
  headers: KeyValue[];
  body: Body;
  /** Null when the command carries no credentials. */
  auth: Auth | null;
  timeoutMs: number | null;
  verifyTls: boolean;
  warnings: string[];
}

const header = (key: string, value: string): KeyValue => ({
  key,
  value,
  enabled: true,
  description: "",
});
const findHeader = (headers: KeyValue[], name: string) =>
  headers.find((h) => h.key.toLowerCase() === name.toLowerCase());

function decodeBase64(value: string): string | null {
  try {
    return atob(value);
  } catch {
    return null;
  }
}

function takeAuth(headers: KeyValue[], options: Option[]): Auth | null {
  const user = options.filter((o) => o.name === "user").at(-1)?.value;
  if (user !== undefined) {
    const index = user.indexOf(":");
    return {
      type: "basic",
      username: index < 0 ? user : user.slice(0, index),
      password: index < 0 ? "" : user.slice(index + 1),
    };
  }
  const authorization = findHeader(headers, "authorization");
  const match = authorization?.value.match(/^(Bearer|Basic)\s+(.+)$/i);
  if (!authorization || !match) return null;
  headers.splice(headers.indexOf(authorization), 1);
  if (match[1].toLowerCase() === "bearer") return { type: "bearer", token: match[2] };
  const decoded = decodeBase64(match[2]) ?? "";
  const index = decoded.indexOf(":");
  return {
    type: "basic",
    username: decoded.slice(0, Math.max(index, 0)),
    password: decoded.slice(index + 1),
  };
}

function formPart(value: string, literal: boolean): MultipartPart {
  const index = value.indexOf("=");
  const key = index < 0 ? value : value.slice(0, index);
  const raw = index < 0 ? "" : value.slice(index + 1);
  if (!literal && raw.startsWith("@"))
    return { type: "file", key, path: raw.slice(1).split(";")[0], enabled: true };
  return {
    type: "text",
    key,
    value: literal ? raw : raw.replace(/;type=[^;]*$/, ""),
    enabled: true,
  };
}

function encodeData(value: string): string {
  const index = value.indexOf("=");
  if (index < 0) return encodeURIComponent(value);
  return `${value.slice(0, index)}=${encodeURIComponent(value.slice(index + 1))}`;
}

function bodyFromData(data: string, contentType: string): Body {
  if (data.startsWith("@")) return { type: "binary", path: data.slice(1) };
  const trimmed = data.trim();
  const looksJson = /^[[{]/.test(trimmed);
  if (contentType.includes("json") || (looksJson && !contentType))
    return { type: "json", text: data };
  const isForm =
    contentType.includes("x-www-form-urlencoded") ||
    (!contentType && /^[^=&\s]+=[^&]*(&[^=&\s]+=[^&]*)*$/.test(trimmed));
  if (isForm) {
    return {
      type: "form_url_encoded",
      fields: parseQuery(trimmed).map((p) => header(p.key, p.value)),
    };
  }
  return { type: "raw", text: data, contentType: contentType || "text/plain" };
}

export function parseCurl(command: string): CurlRequest {
  const args = tokenize(command.trim());
  if (args[0] !== "curl") throw new Error("Perintah harus diawali dengan curl");
  const { options, positionals, unknown } = readOptions(args.slice(1));
  const all = (name: string) => options.filter((o) => o.name === name).map((o) => o.value ?? "");
  const has = (name: string) => options.some((o) => o.name === name);
  const warnings = unknown
    .filter((u) => u.startsWith("--proxy") || u.startsWith("--cert"))
    .map((u) => `Opsi ${u} diabaikan`);
  if (has("proxy") || has("cert"))
    warnings.push("Proxy dan sertifikat klien belum didukung dan diabaikan");

  let url = all("url").at(-1) ?? positionals[0] ?? "";
  const headers = all("header")
    .map((h) => h.split(/:(.*)/s))
    .filter(([key]) => key.trim())
    .map(([key, value = ""]) => header(key.trim(), value.trim()));
  for (const [name, key] of [
    ["user-agent", "User-Agent"],
    ["cookie", "Cookie"],
    ["referer", "Referer"],
  ] as const) {
    const value = all(name).at(-1);
    if (value !== undefined && !findHeader(headers, key)) headers.push(header(key, value));
  }
  const auth = takeAuth(headers, options);

  const json = all("json");
  if (json.length) {
    if (!findHeader(headers, "content-type"))
      headers.push(header("Content-Type", "application/json"));
    if (!findHeader(headers, "accept")) headers.push(header("Accept", "application/json"));
  }
  const data = [
    ...all("data"),
    ...all("data-raw"),
    ...all("data-urlencode").map(encodeData),
    ...json,
  ];
  const forms = [
    ...all("form").map((f) => formPart(f, false)),
    ...all("form-string").map((f) => formPart(f, true)),
  ];

  let body: Body = { type: "none" };
  if (has("get") && data.length) {
    url += (url.includes("?") ? "&" : "?") + data.join("&");
  } else if (forms.length) {
    body = { type: "multipart", parts: forms };
    const contentType = findHeader(headers, "content-type");
    // A multipart Content-Type without the generated boundary would break the request.
    if (contentType) headers.splice(headers.indexOf(contentType), 1);
  } else if (data.length) {
    body = bodyFromData(
      data.join("&"),
      findHeader(headers, "content-type")?.value.toLowerCase() ?? "",
    );
  }

  const explicit = all("request").at(-1)?.toUpperCase();
  const method: HttpMethod =
    explicit && (METHODS as string[]).includes(explicit)
      ? (explicit as HttpMethod)
      : has("head")
        ? "HEAD"
        : body.type !== "none"
          ? "POST"
          : "GET";
  if (explicit && method !== explicit)
    warnings.push(`Method ${explicit} belum didukung, diganti ${method}`);

  const maxTime = Number(all("max-time").at(-1));
  return {
    method,
    url,
    headers,
    body,
    auth,
    timeoutMs: Number.isFinite(maxTime) && maxTime > 0 ? Math.round(maxTime * 1000) : null,
    verifyTls: !has("insecure"),
    warnings,
  };
}
