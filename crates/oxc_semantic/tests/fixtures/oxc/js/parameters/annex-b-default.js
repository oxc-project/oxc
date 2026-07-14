function f(a = x) {
//             ^ resolves to the block's function `x`, which is hoisted to
//               the function scope (Annex B) and merged with `var x` while
//               the body is visited, after the parameters
  {
    function x() {}
  }
  var x;
}
