// Statement-level type-only re-export
export type { Foo } from "./foo";

// Mixed specifier-level type export
export { type Foo, Bar } from "./foo";

// String-named exports
export { "default" as Foo } from "./foo";
export { Foo as "non-identifier" } from "./foo";

// Re-export with import attributes
export { data } from "./data.json" with { type: "json" };
export * from "./data.json" with { type: "json" };
export * as data from "./data.json" with { type: "json" };
