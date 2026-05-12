// Repro of issue #21600: parameter property with default value should still
// emit an uninitialized class field declaration to match TypeScript's
// `useDefineForClassFields: true` output.
class Foo {
  constructor(public b1 = 2.1) {
  }
}
