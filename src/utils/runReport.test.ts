import { describe, expect, it } from "vitest";
import { reportHtml, reportJson, type RunReport } from "./runReport";

const report: RunReport = {
  name: "API <Echecklist>",
  finishedAt: "20 Okt 2026 17.00",
  summary: { total: 2, iterations: 2, passed: 1, failed: 1, durationMs: 420, cancelled: false },
  items: [
    {
      index: 1,
      total: 2,
      iteration: 1,
      iterationLabel: "a@contoh.id",
      requestId: "r1",
      name: "Login",
      method: "POST",
      status: 200,
      durationMs: 210,
      tests: [{ name: "status 200", passed: true, error: null }],
      error: null,
      passed: true,
    },
    {
      index: 2,
      total: 2,
      iteration: 2,
      iterationLabel: "<b>x</b>",
      requestId: "r1",
      name: "Login",
      method: "POST",
      status: 401,
      durationMs: 210,
      tests: [{ name: "status 200", passed: false, error: "expected 401 to equal 200" }],
      error: null,
      passed: false,
    },
  ],
};

describe("run report", () => {
  it("escapes every value in HTML", () => {
    const html = reportHtml(report);
    expect(html).toContain("API &lt;Echecklist&gt;");
    expect(html).toContain("&lt;b&gt;x&lt;/b&gt;");
    expect(html).not.toContain("<b>x</b>");
    expect(html).toContain("1 lulus, 1 gagal dari 2 request");
  });

  it("keeps all results in JSON", () => {
    expect(JSON.parse(reportJson(report)).items).toHaveLength(2);
  });
});
