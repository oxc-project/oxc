// Negative test for the bare-identifier predicate: `const C = class {}` produces
// a `Variable` symbol (not `Class`), and not `Import`. The bare-emit path must
// not fire here — the wrapped `typeof === "function"` guard is emitted instead,
// matching tsc/babel.

const C = class {
  value = 1;
};

declare function dec(target: any, key: string): void;

class Source {
  @dec x!: C;
}
