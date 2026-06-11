export interface Point {
  x: number;
  y: number;
}

export const ok_origin: Point = { x: 0, y: 0 };

export function ok_makePoint(x: number, y: number): Point {
  return { x, y };
}
