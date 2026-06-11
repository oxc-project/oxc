// Type-literal targets: missing members, optional members, excess properties.
type Point = { x: number; y: number };
type LabeledPoint = { x: number; y: number; label?: string };

const ok_point: Point = { x: 1, y: 2 };

const bad_missing_y: Point = { x: 1 };

const bad_extra_member: Point = { x: 1, y: 2, z: 3 };

const ok_optional_omitted: LabeledPoint = { x: 0, y: 0 };
const ok_optional_present: LabeledPoint = { x: 0, y: 0, label: "origin" };

const bad_optional_wrong_type: LabeledPoint = { x: 0, y: 0, label: 42 };

// Width subtyping through a variable: extra member is fine (no freshness check).
const wide_value = { x: 1, y: 2, z: 3 };
const ok_width_subtyping: Point = wide_value;

export {};
