// Decorators on erased members never run. A reference-free decorator keeps the semantic check
// independent of how TypeScript erasure cleans up references.
abstract class A {
  @((value: any) => value) declare x: number;
  @((value: any) => value) abstract accessor a: number;
  method() {}
}
