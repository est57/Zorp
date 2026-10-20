import type { ResponseData } from "@/bindings/ResponseData";
import type { ResponseExample } from "@/bindings/ResponseExample";

/** Shows a saved example through the regular response views. */
export function exampleResponse(example: ResponseExample, url: string): ResponseData {
  return {
    status: example.status,
    statusText: example.statusText,
    headers: example.headers,
    body: { type: "text", text: example.body },
    sizeBytes: new TextEncoder().encode(example.body).length,
    timing: { dnsMs: null, connectMs: null, ttfbMs: 0, downloadMs: 0, totalMs: 0 },
    httpVersion: "",
    url,
    script: { tests: [], logs: [], errors: [], variablesChanged: false },
  };
}

/** Default example name, for example "200 OK". */
export const exampleName = (response: ResponseData) =>
  `${response.status} ${response.statusText}`.trim();
