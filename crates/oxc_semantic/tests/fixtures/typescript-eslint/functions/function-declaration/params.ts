const outer = 1;
function foo(a, [b], { c }, d = 1, e = a, f = outer, g) {
  a;
}

const unresolved = g;
