import type { SnippetRequest } from "./model";

const str = JSON.stringify;
const dict = (pairs: [string, string][]) =>
  pairs.length ? `{\n${pairs.map(([k, v]) => `    ${str(k)}: ${str(v)},`).join("\n")}\n}` : "{}";

export function toPython(req: SnippetRequest): string {
  const body = req.body;
  const headers =
    body.kind === "multipart"
      ? req.headers.filter(([k]) => k.toLowerCase() !== "content-type")
      : req.headers;
  const lines = ["import requests", "", `url = ${str(req.url)}`, `headers = ${dict(headers)}`];
  const args = ["url", "headers=headers"];

  if (body.kind === "text") {
    lines.push(`payload = ${str(body.text)}`);
    args.push('data=payload.encode("utf-8")');
  } else if (body.kind === "form") {
    lines.push(`payload = ${dict(body.fields)}`);
    args.push("data=payload");
  } else if (body.kind === "multipart") {
    const texts = body.parts
      .filter((p) => p.file === undefined)
      .map((p): [string, string] => [p.key, p.value ?? ""]);
    const files = body.parts.filter((p) => p.file !== undefined);
    lines.push(`payload = ${dict(texts)}`);
    args.push("data=payload");
    if (files.length) {
      lines.push(
        `files = {\n${files.map((p) => `    ${str(p.key)}: open(${str(p.file)}, "rb"),`).join("\n")}\n}`,
      );
      args.push("files=files");
    }
  } else if (body.kind === "file") {
    lines.push(`payload = open(${str(body.path)}, "rb")`);
    args.push("data=payload");
  }

  lines.push(
    "",
    `response = requests.request(${str(req.method)}, ${args.join(", ")})`,
    "print(response.status_code, response.text)",
  );
  return lines.join("\n");
}
