export {};

// Every assignment in this fixture is valid; expected.txt must be empty.
const ok_string: string = "text";
const ok_number: number = 3.14;
const ok_boolean: boolean = false;
const ok_bigint: bigint = 10n;

// Literal types flow into their widened primitives
const lit_num = 7;
const ok_widen_num: number = lit_num;
const lit_tmpl = `quoted`;
const ok_widen_str: string = lit_tmpl;

// Literal unions accept each member and widen to the base
let state: "idle" | "busy" = "idle";
state = "busy";
const ok_state_str: string = state;

// Nullable unions under strict
const maybe_str: string | undefined = undefined;
const ok_maybe: string | undefined = maybe_str;
const ok_nullable_num: number | null = null;

// any, unknown, and never legal flows
declare const loose: any;
const ok_from_any: number = loose;
const ok_to_unknown: unknown = loose;
declare function fail(): never;
const ok_never_branch: string = false ? fail() : "set";

// void
function ok_void_fn(): void {}
const ok_void_val: void = undefined;
ok_void_fn();
