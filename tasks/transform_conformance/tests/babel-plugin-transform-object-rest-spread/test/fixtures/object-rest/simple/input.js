let { ...a0 } = foo;
let { ...b0 } = foo();

let { c0, c1, ...c2 } = foo;
let { d0, d1, ...d2 } = foo();

let bar, { e0, ...e1 } = foo(), baz;

for (let bar, { f0, ...f1 } = foo(), baz;;){}
for (let { g0, ...g1 } in foo){}
for (let { h0, ...h1 } of foo){}

for (let bar, { i0, ...i1 } = ioo(), baz;;);
for (let { j0, ...j1 } in foo);
for (let { k0, ...k1 } of foo);

for (let bar, { l0, ...l1 } = loo(), baz;;) void 0;
for (let { m0, ...m1 } in foo) void 0;
for (let { n0, ...n1 } of foo) void 0;

let { [key]: x, ...rest } = foo;
let { [key1]: x2, ...rest2 } = foo();
