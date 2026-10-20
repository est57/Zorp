// Helpers for pm.sendRequest: variable filling and Postman request shapes.
pm.variables.replaceIn = (text) =>
  String(text).replace(/\{\{([^{}]+)\}\}/g, (match, name) => {
    const value = pm.variables.get(name.trim());
    return value === undefined ? match : String(value);
  });

/** Accepts a URL string or a Postman request object; variables are filled in. */
function __normalizeRequest(request) {
  const fill = pm.variables.replaceIn;
  if (typeof request === "string") return { method: "GET", url: fill(request), headers: [], body: null };
  const source = request.header ?? request.headers ?? {};
  const headers = Array.isArray(source)
    ? source.filter((h) => !h.disabled).map((h) => [fill(h.key), fill(h.value ?? "")])
    : Object.entries(source).map(([k, v]) => [fill(k), fill(v)]);
  const hasType = () => headers.some(([k]) => k.toLowerCase() === "content-type");
  const b = request.body;
  let body = null;
  if (typeof b === "string") body = b;
  else if (b && b.mode === "raw") body = b.raw ?? "";
  else if (b && b.mode === "urlencoded") {
    body = (b.urlencoded ?? [])
      .filter((p) => !p.disabled)
      .map((p) => `${encodeURIComponent(fill(p.key))}=${encodeURIComponent(fill(p.value ?? ""))}`)
      .join("&");
    if (!hasType()) headers.push(["Content-Type", "application/x-www-form-urlencoded"]);
  } else if (b && typeof b === "object") {
    body = JSON.stringify(b);
    if (!hasType()) headers.push(["Content-Type", "application/json"]);
  }
  if (b && b.mode === "raw" && b.options?.raw?.language === "json" && !hasType())
    headers.push(["Content-Type", "application/json"]);
  const url = typeof request.url === "string" ? request.url : request.url?.raw;
  return {
    method: String(request.method ?? "GET").toUpperCase(),
    url: fill(url ?? ""),
    headers,
    body: body === null ? null : fill(body),
  };
}
