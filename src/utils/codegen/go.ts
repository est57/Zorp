import type { SnippetRequest } from "./model";

const str = JSON.stringify;

function bodyCode(req: SnippetRequest): { imports: string[]; setup: string[]; reader: string } {
  const body = req.body;
  if (body.kind === "text")
    return { imports: ["strings"], setup: [], reader: `strings.NewReader(${str(body.text)})` };
  if (body.kind === "form") {
    const setup = [
      "form := url.Values{}",
      ...body.fields.map(([k, v]) => `form.Add(${str(k)}, ${str(v)})`),
    ];
    return { imports: ["net/url", "strings"], setup, reader: "strings.NewReader(form.Encode())" };
  }
  if (body.kind === "file") {
    return {
      imports: ["os"],
      setup: [
        `file, err := os.Open(${str(body.path)})`,
        "if err != nil {",
        "\tpanic(err)",
        "}",
        "defer file.Close()",
      ],
      reader: "file",
    };
  }
  if (body.kind === "multipart") {
    const setup = ["body := &bytes.Buffer{}", "writer := multipart.NewWriter(body)"];
    for (const p of body.parts) {
      if (p.file === undefined) {
        setup.push(`writer.WriteField(${str(p.key)}, ${str(p.value)})`);
      } else {
        setup.push(
          "{",
          `\tfile, err := os.Open(${str(p.file)})`,
          "\tif err != nil {",
          "\t\tpanic(err)",
          "\t}",
          `\tpart, _ := writer.CreateFormFile(${str(p.key)}, filepath.Base(${str(p.file)}))`,
          "\tio.Copy(part, file)",
          "\tfile.Close()",
          "}",
        );
      }
    }
    setup.push("writer.Close()");
    const hasFile = body.parts.some((p) => p.file !== undefined);
    return {
      imports: ["bytes", "mime/multipart", ...(hasFile ? ["os", "path/filepath"] : [])],
      setup,
      reader: "body",
    };
  }
  return { imports: [], setup: [], reader: "nil" };
}

export function toGo(req: SnippetRequest): string {
  const { imports, setup, reader } = bodyCode(req);
  const multipart = req.body.kind === "multipart";
  const headers = req.headers.filter(([k]) => !(multipart && k.toLowerCase() === "content-type"));
  const all = [...new Set(["fmt", "io", "net/http", ...imports])].sort();
  const lines = [
    "package main",
    "",
    `import (\n${all.map((i) => `\t${str(i)}`).join("\n")}\n)`,
    "",
    "func main() {",
    ...setup.map((l) => `\t${l}`),
    `\treq, err := http.NewRequest(${str(req.method)}, ${str(req.url)}, ${reader})`,
    "\tif err != nil {",
    "\t\tpanic(err)",
    "\t}",
    ...headers.map(([k, v]) => `\treq.Header.Set(${str(k)}, ${str(v)})`),
    ...(multipart ? ['\treq.Header.Set("Content-Type", writer.FormDataContentType())'] : []),
    "",
    "\tres, err := http.DefaultClient.Do(req)",
    "\tif err != nil {",
    "\t\tpanic(err)",
    "\t}",
    "\tdefer res.Body.Close()",
    "\tdata, _ := io.ReadAll(res.Body)",
    "\tfmt.Println(res.Status, string(data))",
    "}",
  ];
  return lines.join("\n");
}
