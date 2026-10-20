import type { RunItemResult } from "@/bindings/RunItemResult";
import type { RunSummary } from "@/bindings/RunSummary";

export interface RunReport {
  name: string;
  finishedAt: string;
  summary: RunSummary;
  items: RunItemResult[];
}

const escape = (text: string) =>
  text.replace(
    /[&<>"']/g,
    (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!,
  );

export const reportJson = (report: RunReport) => JSON.stringify(report, null, 2);

/** A self-contained HTML page; every value is escaped, so request names cannot inject markup. */
export function reportHtml(report: RunReport): string {
  const s = report.summary;
  const rows = report.items
    .map((item) => {
      const tests = item.tests
        .map(
          (t) =>
            `<li class="${t.passed ? "ok" : "bad"}">${escape(t.name)}${t.error ? `: ${escape(t.error)}` : ""}</li>`,
        )
        .join("");
      return `<tr class="${item.passed ? "ok" : "bad"}"><td>${item.iteration}${item.iterationLabel ? ` <small>${escape(item.iterationLabel)}</small>` : ""}</td><td>${item.passed ? "LULUS" : "GAGAL"}</td><td>${escape(item.method)} ${escape(item.name)}</td><td>${item.status ?? "-"}</td><td>${item.durationMs ?? "-"} ms</td><td>${item.error ? `<p>${escape(item.error)}</p>` : ""}<ul>${tests}</ul></td></tr>`;
    })
    .join("\n");
  return `<!doctype html>
<html lang="id"><head><meta charset="utf-8"><title>Laporan ${escape(report.name)}</title>
<style>body{font:13px system-ui,sans-serif;margin:24px;color:#161922}table{border-collapse:collapse;width:100%}td,th{border:1px solid #dadde5;padding:6px 8px;text-align:left;vertical-align:top}th{background:#eef0f4}.ok td:nth-child(2){color:#16803c}.bad td:nth-child(2){color:#c92a2a}li.bad{color:#c92a2a}ul{margin:0;padding-left:16px}small{color:#5a6172}</style>
</head><body>
<h1>Laporan runner: ${escape(report.name)}</h1>
<p>${escape(report.finishedAt)} · ${s.iterations} iterasi · ${s.passed} lulus, ${s.failed} gagal dari ${s.total} request · ${s.durationMs} ms${s.cancelled ? " · dibatalkan" : ""}</p>
<table><thead><tr><th>Iterasi</th><th>Hasil</th><th>Request</th><th>Status</th><th>Durasi</th><th>Detail</th></tr></thead>
<tbody>
${rows}
</tbody></table>
</body></html>
`;
}
