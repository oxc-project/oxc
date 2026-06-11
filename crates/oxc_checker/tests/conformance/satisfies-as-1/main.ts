import { Theme, Point } from "./theme";

export const ok_theme = {
  primary: "blue",
  mode: "dark",
  spacing: 8,
} satisfies Theme;

// satisfies keeps the literal type when the contextual type is a literal union
export const ok_mode_literal: "dark" = ok_theme.mode;

export const bad_missing_prop = {
  primary: "red",
  mode: "light",
} satisfies Theme;

export const bad_wrong_prop_type = {
  primary: "green",
  mode: "dark",
  spacing: "wide",
} satisfies Theme;

export const bad_excess_prop = {
  x: 1,
  y: 2,
  z: 3,
} satisfies Point;

export const ok_origin = { x: 0, y: 0 } satisfies Point;
export const ok_x_is_number: number = ok_origin.x;
