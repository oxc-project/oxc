// Deeply nested qualified name with shadowing

namespace A {
  export namespace B {
    export type C<T> = T;
  }
}

// A.B.C should reference the namespace A, not the type parameter A
type Test<A> = A.B.C<A>;
