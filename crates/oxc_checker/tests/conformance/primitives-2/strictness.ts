export {};

// Under strictNullChecks null/undefined stay out of other types
const bad_null_to_string: string = null;
const bad_undef_to_number: number = undefined;
const ok_null: null = null;
const ok_undef: undefined = undefined;
const ok_nullable_union: string | null = null;
const ok_optional_union: number | undefined = undefined;

// void accepts undefined but not null under strict
const ok_void_from_undef: void = undefined;
const bad_void_from_null: void = null;

// void does not flow back to undefined
declare const some_void: void;
const bad_undef_from_void: undefined = some_void;
const ok_unknown_from_void: unknown = some_void;
