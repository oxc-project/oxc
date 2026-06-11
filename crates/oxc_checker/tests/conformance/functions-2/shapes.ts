export interface Point {
  x: number;
  y: number;
}

export function make_point(x: number, y: number): Point {
  return { x, y };
}

export function bad_point_missing_field(x: number): Point {
  return { x };
}

export function ok_scale(point: Point, factor: number): Point {
  return { x: point.x * factor, y: point.y * factor };
}
