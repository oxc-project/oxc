import type { Lookup } from "./shapes";

// Constraint violated at the extends clause (TS2344).
export interface BadNumericLookup extends Lookup<number> {
  extra: boolean;
}

// Clean: constraint satisfied, V falls back to its default.
export interface OkNamedLookup extends Lookup<"named"> {
  extra: boolean;
}
