import type { RequestSpec } from "@/bindings/RequestSpec";
import { paramsFromUrl } from "@/utils/url";
import type { CurlRequest } from "./parse";

/** Replaces the request fields described by a parsed cURL command. */
export function applyCurl(spec: RequestSpec, curl: CurlRequest): void {
  spec.method = curl.method;
  spec.url = curl.url;
  spec.params = paramsFromUrl(curl.url, []);
  spec.headers = curl.headers;
  spec.body = curl.body;
  if (curl.auth) spec.auth = curl.auth;
  if (curl.timeoutMs) spec.settings.timeoutMs = curl.timeoutMs;
  spec.settings.verifyTls = curl.verifyTls;
}

export const looksLikeCurl = (text: string) => /^\s*curl\s/.test(text);
