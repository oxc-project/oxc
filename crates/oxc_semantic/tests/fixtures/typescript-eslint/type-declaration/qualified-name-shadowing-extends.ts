// Namespace used in extends clause with shadowing type parameter

namespace T {
  export class C<U = number> {
    prop: U;
  }
}

// T.C should reference the namespace T, not the type parameter T
class Foo<T> extends T.C {
  prop2: T;
}
