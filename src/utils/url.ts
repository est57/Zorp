import type { KeyValue } from "@/bindings/KeyValue";

const decode = (value: string) => {
  try {
    return decodeURIComponent(value.replace(/\+/g, " "));
  } catch {
    return value;
  }
};

// Keep {{variables}} readable while escaping characters that would break the query string.
const encode = (value: string) => value.replace(/[&#=+%\s]/g, (c) => encodeURIComponent(c));

export function splitUrl(url: string): { base: string; query: string; hash: string } {
  const hashIndex = url.indexOf("#");
  const hash = hashIndex >= 0 ? url.slice(hashIndex) : "";
  const withoutHash = hashIndex >= 0 ? url.slice(0, hashIndex) : url;
  const queryIndex = withoutHash.indexOf("?");
  if (queryIndex < 0) return { base: withoutHash, query: "", hash };
  return { base: withoutHash.slice(0, queryIndex), query: withoutHash.slice(queryIndex + 1), hash };
}

export function parseQuery(query: string): { key: string; value: string }[] {
  if (!query) return [];
  return query
    .split("&")
    .filter(Boolean)
    .map((pair) => {
      const index = pair.indexOf("=");
      return index < 0
        ? { key: decode(pair), value: "" }
        : { key: decode(pair.slice(0, index)), value: decode(pair.slice(index + 1)) };
    });
}

/** Rebuilds params from the URL, keeping disabled rows and descriptions of matching keys. */
export function paramsFromUrl(url: string, previous: KeyValue[]): KeyValue[] {
  const parsed = parseQuery(splitUrl(url).query);
  const pool = previous.filter((p) => p.enabled);
  const enabled = parsed.map(({ key, value }) => {
    const match = pool.findIndex((p) => p.key === key);
    const description = match >= 0 ? pool.splice(match, 1)[0].description : "";
    return { key, value, enabled: true, description };
  });
  return [...enabled, ...previous.filter((p) => !p.enabled)];
}

export function urlFromParams(url: string, params: KeyValue[]): string {
  const { base, hash } = splitUrl(url);
  const query = params
    .filter((p) => p.enabled && p.key !== "")
    .map((p) => (p.value === "" ? encode(p.key) : `${encode(p.key)}=${encode(p.value)}`))
    .join("&");
  return `${base}${query ? `?${query}` : ""}${hash}`;
}
