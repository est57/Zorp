import { describe, expect, it } from "vitest";
import { fuzzyMatch, highlight } from "./fuzzy";

describe("fuzzyMatch", () => {
  it("matches subsequences and rejects missing characters", () => {
    expect(fuzzyMatch("lsus", "List users")?.indices).toEqual([0, 2, 5, 6]);
    expect(fuzzyMatch("xyz", "List users")).toBeNull();
  });

  it("ranks prefix and word-start matches higher", () => {
    const prefix = fuzzyMatch("user", "Users list")!.score;
    const middle = fuzzyMatch("user", "Delete a user")!.score;
    const scattered = fuzzyMatch("user", "Unused report entries")!.score;
    expect(prefix).toBeGreaterThan(middle);
    expect(middle).toBeGreaterThan(scattered);
  });

  it("matches everything for an empty query", () => {
    expect(fuzzyMatch("  ", "Anything")).toEqual({ score: 0, indices: [] });
  });
});

describe("highlight", () => {
  it("groups consecutive hits", () => {
    expect(highlight("Users", [0, 1])).toEqual([
      { text: "Us", hit: true },
      { text: "ers", hit: false },
    ]);
  });
});
