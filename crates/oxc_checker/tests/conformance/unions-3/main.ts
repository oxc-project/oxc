import { Ok, Result, describe_result } from "./result";

const ok_value: Result = { tag: "ok", value: 1 };
const ok_err: Result = { tag: "err", message: "boom" };

// A union member upcasts to the union.
const ok_member: Ok = { tag: "ok", value: 2 };
const ok_upcast: Result = ok_member;

// Widening the union with null is fine when the target includes null.
const ok_nullable: Result | null = null;
const ok_nullable_value: Result | null = ok_value;

// Property access on a union yields the union of property types.
const ok_tag: "ok" | "err" = ok_value.tag;

// Literal union argument flows into a union-typed parameter.
const ok_described: string = describe_result(ok_err);

export const exported_result: Result = ok_value;
