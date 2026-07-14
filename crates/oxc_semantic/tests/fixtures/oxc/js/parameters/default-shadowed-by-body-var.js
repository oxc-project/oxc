let b = 0;
function f(a = b) {
//             ^ resolves to the outer `let b`, not the body `var b`:
//               parameter references bind to bindings visible when the
//               parameter list is declared
  var b;
}
