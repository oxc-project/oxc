import type Foo = require("./foo");
import Bar = require("./bar");

// Type-only aliases resolve in type positions, including qualified names and type queries.
type Plain = Foo;
type Qualified = Foo.Member;
type Query = typeof Foo;
type QualifiedQuery = typeof Foo.member;

// Runtime reads must not resolve to the type-only import.
Foo;
Foo.member;

// Ordinary import-equals aliases remain usable in both value and type positions.
Bar;
type ValueType = Bar;
type ValueQuery = typeof Bar;

export type { Foo };
export { Bar };
