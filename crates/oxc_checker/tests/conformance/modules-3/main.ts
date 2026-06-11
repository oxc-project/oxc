import * as valsNs from "./reexport";
// ok: type-only import through the re-export chain
import type { Label, TagLabel } from "./reexport";
// bad: name never exported anywhere in the chain -> TS2305
import { bad_ghost } from "./reexport";

const ok_label: Label = "tag";
const ok_tag: TagLabel = ok_label;

// ok: namespace value access and qualified type use
const ok_described: string = valsNs.ok_describe(ok_tag);
export type QualifiedLabel = valsNs.Label;

// bad: property missing on the namespace -> TS2339
const bad_member: number = valsNs.bad_missingValue;
// bad: qualified type name missing on the namespace -> TS2694
type bad_qualified = valsNs.MissingType;

export const ok_sum: number =
  valsNs.ok_total + valsNs.ok_count + ok_described.length;
