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
  {
    label: "Login otomatis (pm.sendRequest)",
    code: [
      "// Login sekali, lalu pakai ulang token sampai kedaluwarsa.",
      'if (Date.now() > Number(pm.collectionVariables.get("tokenExp") || 0)) {',
      "  const res = pm.sendRequest({",
      '    url: "{{baseUrl}}/login",',
      '    method: "POST",',
      '    body: { email: "{{email}}", password: "{{login_password}}" },',
      "  });",
      '  pm.collectionVariables.set("token", res.json().token);',
      '  pm.collectionVariables.set("tokenExp", Date.now() + 50 * 60 * 1000);',
      "}",
    ].join("\n"),
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
