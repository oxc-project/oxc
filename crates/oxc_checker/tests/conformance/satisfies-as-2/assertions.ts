import { Status } from "./unions";

// `as` changes the checked type of the expression
const raw_value: unknown = "hello";
export const ok_as_from_unknown: string = raw_value as string;

// literal "open" is comparable to Status, so the assertion is allowed
export const ok_as_status: Status = "open" as Status;

// allowed: the operand widens to string, which overlaps Status
export const ok_as_status_widened = "pending" as Status;

const plain_number = 12;
// number and string do not overlap, direct assertion is rejected
export const bad_as_string = plain_number as string;

// round-tripping through unknown silences the overlap check
export const ok_double_assertion: string = plain_number as unknown as string;

// the assertion really changes the type seen by later checks
const reinterpreted = "5" as unknown as number;
export const bad_use_after_assertion: string = reinterpreted;

interface Wide {
  a: number;
  b?: string;
}
const wide_obj: Wide = { a: 1 };
// asserting to a structurally overlapping type is fine
export const ok_as_narrower = wide_obj as { a: number };
