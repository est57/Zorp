export interface Ordered {
  id: string;
  sortOrder: number;
}

export type DropZone = "before" | "after" | "inside";

export const orderAtEnd = (siblings: Ordered[]) =>
  Math.max(0, ...siblings.map((s) => s.sortOrder)) + 1;

/** Sort order that places an item before or after `targetId` among sorted siblings. */
export function orderNextTo(
  siblings: Ordered[],
  targetId: string,
  zone: "before" | "after",
): number {
  const index = siblings.findIndex((s) => s.id === targetId);
  if (index < 0) return orderAtEnd(siblings);
  const at = zone === "before" ? index : index + 1;
  const prev = siblings[at - 1]?.sortOrder;
  const next = siblings[at]?.sortOrder;
  if (prev === undefined && next === undefined) return 1;
  if (prev === undefined) return next! - 1;
  if (next === undefined) return prev + 1;
  return (prev + next) / 2;
}
