const outer = 1;
const foo = function (a, [b], { c }, d = 1, e = a, f = outer, g) {
  a;
};

const unresolved = g;
