export type Pair = [string, number];
export type Point = [number, number];

export const ok_default_pairs: Pair[] = [
  ["a", 1],
  ["b", 2],
];

export function manhattan(a: Point, b: Point): number {
  return b[0] - a[0] + (b[1] - a[1]);
}
