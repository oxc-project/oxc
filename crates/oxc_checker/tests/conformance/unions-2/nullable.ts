// Unions with null and undefined under strictNullChecks.
type MaybeName = string | null;
type OptCount = number | undefined;

const ok_null: MaybeName = null;
const ok_name: MaybeName = "alice";
const bad_undef_into_null_union: MaybeName = undefined;

const ok_undef: OptCount = undefined;
const ok_count: OptCount = 7;
const bad_null_into_undef_union: OptCount = null;

declare const maybe_name: MaybeName;
const bad_drop_null: string = maybe_name;
const ok_keep_null: string | null | undefined = maybe_name;

export {};
