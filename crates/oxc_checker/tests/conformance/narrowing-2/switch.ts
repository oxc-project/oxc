// Discriminated union switch narrowing: exhaustive switches are silent.
import type { Shape } from "./shapes";

export function ok_area(shape: Shape): number {
  switch (shape.kind) {
    case "circle":
      return shape.radius * shape.radius * 3;
    case "square":
      return shape.side * shape.side;
    case "triangle":
      return (shape.base * shape.height) / 2;
  }
}

export function ok_describe(shape: Shape): string {
  switch (shape.kind) {
    case "circle":
      return "circle " + shape.radius;
    case "square":
      return "square " + shape.side;
    case "triangle":
      return "triangle " + shape.base;
    default: {
      const ok_exhaustive: never = shape;
      return ok_exhaustive;
    }
  }
}

export function bad_access_without_switch(shape: Shape): number {
  return shape.radius;
}

export function bad_wrong_member_in_case(shape: Shape): number {
  switch (shape.kind) {
    case "circle":
      return shape.side;
    default:
      return 0;
  }
}
