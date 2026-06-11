export interface Point {
  x: number;
  y: number;
}

export type ID = string | number;

export const ORIGIN: Point = { x: 0, y: 0 };

export const VERSION = 2;

export function distance(a: Point, b: Point): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  return dx * dx + dy * dy;
}
