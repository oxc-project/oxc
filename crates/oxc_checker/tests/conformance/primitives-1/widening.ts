export {};

// const keeps the literal type
const narrow_str = "hello";
const ok_literal: "hello" = narrow_str;

// let widens the literal to string
let widened_str = "hello";
const bad_widened: "hello" = widened_str;

// let initialized from a const literal also widens
const narrow_num = 42;
let widened_num = narrow_num;
const bad_from_widened: 42 = widened_num;

// as const blocks widening even through let
let asconst_num = 42 as const;
const ok_asconst: 42 = asconst_num;

// boolean widens under let, but flow analysis narrows this read
let widened_bool = true;
const ok_flow_narrowed: true = widened_bool;

// without a known flow value, boolean does not satisfy a literal
declare let plain_bool: boolean;
const bad_bool_literal: true = plain_bool;

// literal-typed let only accepts members of its union
let flag: "on" | "off" = "on";
flag = "off";
flag = "toggle";

// widened values still flow to their base primitive
const ok_base_str: string = widened_str;
const ok_base_num: number = widened_num;
