import type { KeyValue } from "@/bindings/KeyValue";

const DISABLED_PREFIX = "//";

export function toBulk(rows: KeyValue[]): string {
  return rows
    .filter((r) => r.key !== "" || r.value !== "")
    .map((r) => `${r.enabled ? "" : DISABLED_PREFIX}${r.key}: ${r.value}`)
    .join("\n");
}

export function fromBulk(text: string, previous: KeyValue[]): KeyValue[] {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const enabled = !line.startsWith(DISABLED_PREFIX);
      const body = enabled ? line : line.slice(DISABLED_PREFIX.length).trim();
      const index = body.indexOf(":");
      const key = (index < 0 ? body : body.slice(0, index)).trim();
      const value = index < 0 ? "" : body.slice(index + 1).trim();
      const description = previous.find((p) => p.key === key)?.description ?? "";
      return { key, value, enabled, description };
    });
}
