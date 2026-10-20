import { describe, expect, it } from "vitest";
import { newRequest } from "./request";
import { authCredentials, requestCredentials } from "./credentials";

const row = (key: string, value: string) => ({ key, value, enabled: true, description: "" });

describe("requestCredentials", () => {
  it("flags credentials typed in directly", () => {
    const spec = newRequest({
      auth: { type: "bearer", token: "abc123" },
      headers: [row("X-Api-Key", "k-1"), row("Accept", "application/json")],
      params: [row("access_token", "t")],
      body: { type: "json", text: '{\n  "email": "a@b.c",\n  "password": "rahasia"\n}' },
    });
    expect(requestCredentials(spec)).toEqual([
      "Auth Bearer token",
      "Header X-Api-Key",
      "Param access_token",
      "Body password",
    ]);
  });

  it("ignores values that use variables or are empty", () => {
    const spec = newRequest({
      auth: { type: "basic", username: "ada", password: "{{password}}" },
      headers: [row("Authorization", "Bearer {{token}}"), { ...row("Token", "x"), enabled: false }],
      body: { type: "form_url_encoded", fields: [row("password", "")] },
    });
    expect(requestCredentials(spec)).toEqual([]);
    expect(authCredentials({ type: "inherit" })).toEqual([]);
  });
});
