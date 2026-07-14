function f(g = () => x) {
//                   ^ resolves to the body `var x`: nothing named `x` is
//                     visible when `f`'s parameters are declared, so
//                     resolution falls back to the end-of-program state
  var x;
}
