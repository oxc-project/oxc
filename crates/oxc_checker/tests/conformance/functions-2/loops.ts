import { Point, make_point } from "./shapes";

export function ok_first_positive(values: number[]): number {
  for (const value of values) {
    if (value > 0) {
      return value;
    }
  }
  return -1;
}

export function bad_loop_may_fall_through(values: number[]): number {
  for (const value of values) {
    return value;
  }
}

export function bad_loop_wrong_return_type(points: Point[]): string {
  for (const point of points) {
    return point;
  }
  return "none";
}

export function bad_while_returns_void_value(limit: number): number {
  let current: number = 0;
  while (current < limit) {
    current += 1;
  }
  return ok_log_progress(current);
}

function ok_log_progress(step: number): void {
  if (step > 100) {
    return;
  }
}

export function ok_origin(): Point {
  return make_point(0, 0);
}
