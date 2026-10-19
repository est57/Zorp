import type { SnippetRequest } from "./model";

const str = JSON.stringify;
const indent = (text: string, spaces: number) => text.replace(/\n/g, `\n${" ".repeat(spaces)}`);
const headerObject = (headers: [string, string][], omit?: string) =>
  `{\n${headers
    .filter(([k]) => k.toLowerCase() !== omit)
    .map(([k, v]) => `  ${str(k)}: ${str(v)},`)
    .join("\n")}\n}`;

/** Statements that prepare the body and the expression that references it. */
function bodySetup(req: SnippetRequest): { setup: string[]; expr: string | null } {
  const body = req.body;
  if (body.kind === "text") return { setup: [], expr: str(body.text) };
  if (body.kind === "form") {
    return {
      setup: [],
      expr: `new URLSearchParams(${indent(str(Object.fromEntries(body.fields), null, 2), 0)})`,
    };
  }
  if (body.kind === "multipart") {
    const setup = ["const form = new FormData();"];
    for (const p of body.parts) {
      setup.push(
        p.file !== undefined
          ? `form.append(${str(p.key)}, await fs.openAsBlob(${str(p.file)}));`
          : `form.append(${str(p.key)}, ${str(p.value)});`,
      );
    }
    if (body.parts.some((p) => p.file !== undefined)) setup.unshift('import fs from "node:fs";');
    return { setup, expr: "form" };
  }
  if (body.kind === "file")
    return { setup: ['import fs from "node:fs";'], expr: `await fs.openAsBlob(${str(body.path)})` };
  return { setup: [], expr: null };
}

export function toFetch(req: SnippetRequest): string {
  const { setup, expr } = bodySetup(req);
  const omit = req.body.kind === "multipart" ? "content-type" : undefined;
  const options = [`  method: ${str(req.method)},`];
  if (req.headers.length) options.push(`  headers: ${indent(headerObject(req.headers, omit), 2)},`);
  if (expr) options.push(`  body: ${indent(expr, 2)},`);
  return [
    ...setup,
    ...(setup.length ? [""] : []),
    `const response = await fetch(${str(req.url)}, {\n${options.join("\n")}\n});`,
    "",
    "console.log(response.status, await response.text());",
  ].join("\n");
}

export function toAxios(req: SnippetRequest): string {
  const { setup, expr } = bodySetup(req);
  const omit = req.body.kind === "multipart" ? "content-type" : undefined;
  const config = [`  method: ${str(req.method.toLowerCase())},`, `  url: ${str(req.url)},`];
  if (req.headers.length) config.push(`  headers: ${indent(headerObject(req.headers, omit), 2)},`);
  if (expr) config.push(`  data: ${indent(expr, 2)},`);
  return [
    'import axios from "axios";',
    ...setup,
    "",
    `const response = await axios.request({\n${config.join("\n")}\n});`,
    "",
    "console.log(response.status, response.data);",
  ].join("\n");
}
