// `declare class X {}` now emits a bare identifier matching tsc/babel. Previously
// OXC wrapped this in the `typeof X !== "undefined"` guard, silently producing
// `Object` when the ambient binding was unfulfilled at runtime. The bare emit
// surfaces unfulfilled ambient declarations as `ReferenceError`, consistent with
// tsc and babel.

declare class Ambient {}

declare function dec(target: any, key: string): void;

class Source {
  @dec value!: Ambient;
}
