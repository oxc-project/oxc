let [{...a0}] = z

let [{...b0}, {...b1}] = z

let { c0: { ...c1 } } = foo;

let { d0: { d1, ...d2 } } = foo;

let { e0: { e1: { e2, ...e3 }, ...e4 } } = foo;

let { f0: { f1: { f2, ...f3 }, ...f4 }, ...f5 } = foo;

let [{ g0: { g1: { g2, ...g3 }, ...g4 } }, { g5: [{ g6: [{ g7, ...g8 }], ...g9 }] }] = goo;
