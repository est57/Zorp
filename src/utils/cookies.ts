export interface Cookie {
  name: string;
  value: string;
  attributes: string;
}

export function parseSetCookies(headers: [string, string][]): Cookie[] {
  return headers
    .filter(([name]) => name.toLowerCase() === "set-cookie")
    .map(([, raw]) => {
      const [pair, ...attrs] = raw.split(";");
      const index = pair.indexOf("=");
      return {
        name: (index < 0 ? pair : pair.slice(0, index)).trim(),
        value: index < 0 ? "" : pair.slice(index + 1).trim(),
        attributes: attrs
          .map((a) => a.trim())
          .filter(Boolean)
          .join("; "),
      };
    });
}
