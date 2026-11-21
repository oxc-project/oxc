// Decorated fields without an initializer are removed, but their keys are still evaluated in
// order: in the next computed key, or in a static block when no computed key follows.
class A {
  @dec [a()]: number;
  [b()]() {}
  @dec [c()]: number;
}
