import { describe, expect, it } from "vitest";
import { createSSRApp, h } from "vue";
import { renderToString } from "vue/server-renderer";
import { renderMarkdown } from "./markdown";

const html = (source: string) =>
  renderToString(createSSRApp({ render: () => h(() => renderMarkdown(source)) }));

describe("renderMarkdown", () => {
  it("renders common markdown", async () => {
    const out = await html(
      "# Login\n\nKirim `email` dan **password**.\n\n| A | B |\n|---|---|\n| 1 | 2 |",
    );
    expect(out).toContain("<h1>Login</h1>");
    expect(out).toContain("<code>email</code>");
    expect(out).toContain("<strong>password</strong>");
    expect(out).toContain("<td>2</td>");
  });

  it("never outputs raw html or unsafe links", async () => {
    const out = await html(
      '<img src=x onerror="alert(1)"> [klik](javascript:alert(1)) [web](https://contoh.id)',
    );
    expect(out).not.toContain("<img");
    expect(out).not.toMatch(/href="javascript/i);
    expect(out).toContain('href="https://contoh.id"');
    expect(out).toContain("&lt;img");
  });
});
