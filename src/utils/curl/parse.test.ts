import { describe, expect, it } from "vitest";
import { parseCurl } from "./parse";
import { tokenize } from "./tokenize";

describe("tokenize", () => {
  it("handles quotes, escapes and line continuations", () => {
    expect(tokenize(`curl 'a b' "c \\"d\\"" e\\ f \\\n $'x\\ny'`)).toEqual([
      "curl",
      "a b",
      'c "d"',
      "e f",
      "x\ny",
    ]);
  });
});

describe("parseCurl", () => {
  it("parses a JSON POST with bearer auth", () => {
    const req = parseCurl(`curl -X POST 'https://api.test/users?x=1' \\
      -H 'Content-Type: application/json' -H 'Authorization: Bearer abc' \\
      --data-raw '{"name":"Rani"}'`);
    expect(req.method).toBe("POST");
    expect(req.url).toBe("https://api.test/users?x=1");
    expect(req.auth).toEqual({ type: "bearer", token: "abc" });
    expect(req.headers.map((h) => h.key)).toEqual(["Content-Type"]);
    expect(req.body).toEqual({ type: "json", text: '{"name":"Rani"}' });
  });

  it("infers POST and form body from -d", () => {
    const req = parseCurl("curl https://api.test/login -d 'user=rani' -d 'pass=a%20b'");
    expect(req.method).toBe("POST");
    expect(req.body).toEqual({
      type: "form_url_encoded",
      fields: [
        { key: "user", value: "rani", enabled: true, description: "" },
        { key: "pass", value: "a b", enabled: true, description: "" },
      ],
    });
  });

  it("parses multipart forms, basic auth and flags", () => {
    const req = parseCurl(
      "curl -sSLk -u rani:secret -F 'file=@/tmp/a.png' -F note=hi -m 5 https://up.test",
    );
    expect(req.auth).toEqual({ type: "basic", username: "rani", password: "secret" });
    expect(req.body).toEqual({
      type: "multipart",
      parts: [
        { type: "file", key: "file", path: "/tmp/a.png", enabled: true },
        { type: "text", key: "note", value: "hi", enabled: true },
      ],
    });
    expect(req.verifyTls).toBe(false);
    expect(req.timeoutMs).toBe(5000);
  });

  it("moves data to the query with -G and supports attached values", () => {
    const req = parseCurl("curl -G https://s.test/q -d q=zorp -XGET");
    expect(req.method).toBe("GET");
    expect(req.url).toBe("https://s.test/q?q=zorp");
    expect(req.body).toEqual({ type: "none" });
  });

  it("rejects commands that are not curl", () => {
    expect(() => parseCurl("wget https://x.test")).toThrow();
  });
});
