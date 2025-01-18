import nsa from "mod";

namespace N1 {
  // Retain because `onlyRemoveTypeImports` is true
  import Foo = nsa.bar;
  const foo = 0;
}

namespace N2 {
  // Retain because `onlyRemoveTypeImports` is true
  import Foo = nsa.bar;
  const foo: Foo = 0;
}
