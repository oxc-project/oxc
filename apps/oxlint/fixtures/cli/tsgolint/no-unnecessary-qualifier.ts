// Examples of incorrect code for no-unnecessary-qualifier rule

namespace A {
  export type B = number;
  const x: A.B = 3;
}
