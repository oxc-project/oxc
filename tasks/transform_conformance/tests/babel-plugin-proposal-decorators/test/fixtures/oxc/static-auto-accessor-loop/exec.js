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
