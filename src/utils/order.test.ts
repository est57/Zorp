import { describe, expect, it } from "vitest";
import { orderAtEnd, orderNextTo } from "./order";

const items = [
  { id: "a", sortOrder: 1 },
  { id: "b", sortOrder: 2 },
  { id: "c", sortOrder: 4 },
];

describe("order helpers", () => {
  it("places items between neighbours", () => {
    expect(orderNextTo(items, "b", "after")).toBe(3);
    expect(orderNextTo(items, "b", "before")).toBe(1.5);
  });

  it("places items at the edges", () => {
    expect(orderNextTo(items, "a", "before")).toBe(0);
    expect(orderNextTo(items, "c", "after")).toBe(5);
  });

  it("appends to the end", () => {
    expect(orderAtEnd(items)).toBe(5);
    expect(orderAtEnd([])).toBe(1);
  });
});
