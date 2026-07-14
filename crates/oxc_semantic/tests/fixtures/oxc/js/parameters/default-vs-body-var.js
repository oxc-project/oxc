function f(a = b) {
//             ^ resolves to the body `var b`: no `b` is visible when the
//               parameters are declared, so resolution falls back to the
//               state after the whole program has been visited
  var b;
}
