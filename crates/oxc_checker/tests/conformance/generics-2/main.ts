import type { Container, Lookup } from "./shapes";

// Clean: generic interface instantiated and implemented correctly.
const ok_container: Container<string> = {
  content: "ready",
  replace(next: string): void {},
};

const ok_lookup_default: Lookup<"name"> = { key: "name", value: 3 };
const ok_lookup_explicit: Lookup<"age", boolean> = { key: "age", value: true };

// Member mismatch after instantiation.
const bad_content: Container<number> = {
  content: "oops",
  replace(next: number): void {},
};

const bad_method: Container<number> = {
  content: 1,
  replace(next: string): void {},
};

const bad_lookup_key: Lookup<"name"> = { key: "title", value: 3 };
const bad_lookup_value: Lookup<"name", string> = { key: "name", value: 7 };

// Missing type arguments entirely (TS2314).
const bad_arity: Container = { content: 0, replace(next: number): void {} };

export const generics_two_content: string = ok_container.content;
