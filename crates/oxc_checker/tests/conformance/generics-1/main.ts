import type { Boxed, KeyHolder, Pair, WithDefault } from "./types";

// Clean instantiations: silent where tsc is silent.
const ok_pair: Pair<number, string> = { first: 1, second: "two" };
const ok_default: WithDefault = { item: "uses the string default" };
const ok_explicit_default: WithDefault<number> = { item: 5 };
const ok_key: KeyHolder<"id"> = { key: "id" };

// Wrong number of type arguments (TS2314).
type BadTooFew = Pair<number>;
type BadTooMany = Boxed<number, string>;

// Member mismatch after instantiation (TS2322 / TS2741).
const bad_pair_first: Pair<number, string> = { first: "one", second: "two" };
const bad_boxed_missing: Boxed<number> = {};
const bad_default_item: WithDefault = { item: 42 };

// Constraint violated on the type alias (TS2344).
type BadConstraint = KeyHolder<number>;

export const generics_one_total: number =
  ok_pair.first + ok_explicit_default.item;
