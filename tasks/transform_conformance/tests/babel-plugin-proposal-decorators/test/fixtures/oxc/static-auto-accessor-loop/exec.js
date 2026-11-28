// Each evaluation of a class with a lowered static auto-accessor gets its own storage, even in a
// brace-less loop body.
const classes = [];
for (let i = 0; i < 2; i++) classes.push(class W { static accessor a = i; static get() { return W.a; } });
expect(classes.map((C) => C.get())).toEqual([0, 1]);
classes[0].a = 5;
expect(classes.map((C) => C.a)).toEqual([5, 1]);

const make = (value) => class { static accessor a = value; };
const [A, B] = [make(1), make(2)];
expect([A.a, B.a]).toEqual([1, 2]);

// A class evaluated repeatedly in a loop header also owns its own accessor storage.
{
  const instances = [];
  let i = 0;
  let C;
  while ((C = class { accessor x = i; }, i++ < 3)) instances.push(new C());
  expect(instances.map((c) => c.x)).toEqual([1, 2, 3]);
}

// A class created by a function in an unbraced loop body owns its storage per call.
{
  const makers = [];
  for (let i = 0; i < 2; i++) makers.push(() => class { accessor x = i; });
  const [A, B] = makers.map((make) => make());
  const A2 = makers[0]();
  expect([new A().x, new B().x, new A2().x]).toEqual([0, 1, 0]);
}
