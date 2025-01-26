import nsa from "mod";

let N1;
(function(_N) {
  // Retain because `onlyRemoveTypeImports` is true
  var Foo = nsa.bar;
  const foo = 0;
})(N1 || (N1 = {}));

let N2;
(function(_N2) {
  // Retain because `onlyRemoveTypeImports` is true
  var Foo = nsa.bar;
  const foo = 0;
})(N2 || (N2 = {}));
