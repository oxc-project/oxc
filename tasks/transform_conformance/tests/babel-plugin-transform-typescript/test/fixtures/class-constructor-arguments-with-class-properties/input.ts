// A free reference in a field initializer resolves to an outer variable with the
// same name as a constructor parameter property. The constructor parameter gets
// renamed to avoid shadowing the outer variable once the initializer moves into
// the constructor body. `this.x` must still use the original source name, not `_x`.
let x = 10;
class Foo {
  field = x;
  constructor(public x: number) {}
}
