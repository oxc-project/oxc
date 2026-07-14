let b = 0;
function outer() {
  function f(a = b) {}
  //             ^ resolves to the module-level `let b`: `outer`'s `var b`
  //               below is not yet declared when `f`'s parameters are
  //               declared, and an early match freezes the whole chain
  var b = 1;
}
