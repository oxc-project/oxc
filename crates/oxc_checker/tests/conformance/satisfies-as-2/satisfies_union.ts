import { Shape, Id } from "./unions";

// satisfies picks the matching union arm, so .radius is visible without narrowing
export const ok_circle = { kind: "circle", radius: 2 } satisfies Shape;
export const ok_radius: number = ok_circle.radius;

export const ok_square = { kind: "square", side: 4 } satisfies Shape;

// no union arm has kind "triangle"
export const bad_unknown_kind = { kind: "triangle", sides: 3 } satisfies Shape;

// kind says circle but payload belongs to the square arm
export const bad_mixed_arms = { kind: "circle", side: 4 } satisfies Shape;

export const ok_string_id = "user-1" satisfies Id;
export const ok_number_id = 42 satisfies Id;
export const bad_bool_id = true satisfies Id;
