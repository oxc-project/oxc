// Nested unions flatten; assignability looks through the nesting.
type AB = "a" | "b";
type CD = "c" | "d";
type ABCD = AB | CD;

declare const ab_value: AB;
declare const abcd_value: ABCD;

const ok_nested_subset: ABCD = ab_value;
const ok_nested_literal: ABCD = "d";
const bad_nested_literal: ABCD = "e";
const bad_unnest: AB | "c" = abcd_value;

type DeepNullable = (string | null) | (number | undefined);
const ok_deep_null: DeepNullable = null;
const ok_deep_num: DeepNullable = 5;
const bad_deep_bool: DeepNullable = true;

export {};
