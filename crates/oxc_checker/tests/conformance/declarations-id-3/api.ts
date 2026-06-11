export interface Point {
  readonly x: number;
  readonly y: number;
}

export type Mode = "fast" | "slow";

export enum Direction {
  Up = 1,
  Down = 2,
}

// Clean: literals, as-const, and annotated initializers.
export const ok_origin: Point = { x: 0, y: 0 };
export const ok_mode = "fast";
export const ok_limits = [1, 2, 3] as const;

export function ok_distance_sq(a: Point, b: Point): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  return dx * dx + dy * dy;
}

export class Grid {
  size: number;
  constructor(size: number) {
    this.size = size;
  }
  contains(p: Point): boolean {
    return p.x < this.size && p.y < this.size;
  }
}
