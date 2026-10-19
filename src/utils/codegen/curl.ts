import type { SnippetRequest } from "./model";

export const shellQuote = (value: string) => `'${value.replace(/'/g, `'\\''`)}'`;

export function toCurl(req: SnippetRequest): string {
  const lines = [`curl${req.method === "GET" ? "" : ` -X ${req.method}`} ${shellQuote(req.url)}`];
  for (const [key, value] of req.headers) lines.push(`-H ${shellQuote(`${key}: ${value}`)}`);
  const body = req.body;
  if (body.kind === "text") lines.push(`--data-raw ${shellQuote(body.text)}`);
  if (body.kind === "form") {
    for (const [key, value] of body.fields)
      lines.push(`--data-urlencode ${shellQuote(`${key}=${value}`)}`);
  }
  if (body.kind === "multipart") {
    for (const p of body.parts) {
      lines.push(
        p.file !== undefined
          ? `-F ${shellQuote(`${p.key}=@${p.file}`)}`
          : `--form-string ${shellQuote(`${p.key}=${p.value}`)}`,
      );
    }
  }
  if (body.kind === "file") lines.push(`--data-binary ${shellQuote(`@${body.path}`)}`);
  return lines.join(" \\\n  ");
}
