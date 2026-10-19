import { describe, expect, it } from "vitest";
import { paramsFromUrl, parseQuery, splitUrl, urlFromParams } from "./url";
import { tokenize } from "./variables";

const row = (key: string, value: string, enabled = true, description = "") => ({
  key,
  value,
  enabled,
  description,
});

describe("url utils", () => {
  it("splits base, query and hash", () => {
    expect(splitUrl("{{base}}/a?x=1#top")).toEqual({
      base: "{{base}}/a",
      query: "x=1",
      hash: "#top",
    });
  });

  it("parses encoded query values", () => {
    expect(parseQuery("q=a%20b&flag&x=1+2")).toEqual([
      { key: "q", value: "a b" },
      { key: "flag", value: "" },
      { key: "x", value: "1 2" },
    ]);
  });

  it("keeps disabled params and descriptions when syncing from url", () => {
    const previous = [row("page", "1", true, "Halaman"), row("debug", "1", false)];
    expect(paramsFromUrl("/u?page=2&limit=5", previous)).toEqual([
      row("page", "2", true, "Halaman"),
      row("limit", "5"),
      row("debug", "1", false),
    ]);
  });

  it("builds url from enabled params and preserves variables", () => {
    const params = [row("token", "{{token}}"), row("q", "a&b"), row("off", "1", false)];
    expect(urlFromParams("{{base}}/s?old=1", params)).toBe("{{base}}/s?token={{token}}&q=a%26b");
  });
});

describe("tokenize", () => {
  it("splits text into variable segments", () => {
    expect(tokenize("{{base}}/users/{{ id }}")).toEqual([
      { text: "{{base}}", variable: "base" },
      { text: "/users/" },
      { text: "{{ id }}", variable: "id" },
    ]);
  });
});
