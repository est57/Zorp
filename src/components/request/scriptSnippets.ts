export interface ScriptSnippet {
  label: string;
  code: string;
}

export const PRE_REQUEST_SNIPPETS: ScriptSnippet[] = [
  { label: "Set variabel environment", code: 'pm.environment.set("nama", "nilai");' },
  {
    label: "Tambah header",
    code: 'pm.request.headers.upsert({ key: "X-Request-Id", value: String(Date.now()) });',
  },
  {
    label: "Variabel untuk request ini",
    code: 'pm.variables.set("timestamp", String(Date.now()));',
  },
];

export const TEST_SNIPPETS: ScriptSnippet[] = [
  {
    label: "Status 200",
    code: 'pm.test("Status 200", () => {\n  pm.response.to.have.status(200);\n});',
  },
  {
    label: "Waktu respons",
    code: 'pm.test("Respons di bawah 500 ms", () => {\n  pm.expect(pm.response.responseTime).to.be.below(500);\n});',
  },
  {
    label: "Cek properti JSON",
    code: 'pm.test("Body memiliki id", () => {\n  pm.expect(pm.response.json()).to.have.property("id");\n});',
  },
  { label: "Simpan token", code: 'pm.environment.set("token", pm.response.json().token);' },
];
