function f(x) {}
f(1);
g();
let N;
(function (_N) {
  const x = _N.x = 1;
})(N || (N = {}));
