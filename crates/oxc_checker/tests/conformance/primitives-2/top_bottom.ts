export {};

declare const some_any: any;
declare const some_unknown: unknown;
declare const some_never: never;

// any flows everywhere except never
const ok_any_to_string: string = some_any;
const ok_any_to_bigint: bigint = some_any;
const bad_any_to_never: never = some_any;

// unknown absorbs everything but flows only to any/unknown
const ok_unknown_from_literal: unknown = "abc";
const ok_unknown_from_null: unknown = null;
const bad_unknown_to_string: string = some_unknown;
const ok_unknown_to_any: any = some_unknown;

// never flows to everything
const ok_never_to_string: string = some_never;
const ok_never_to_boolean: boolean = some_never;

// nothing concrete flows into never
const bad_number_to_never: never = 1;
