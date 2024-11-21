({ a1 } = c1);
({ a2, ...b2 } = c2);
(a0, { a2, ...b2 } = c2);
(a0, { a2, ...b2 } = c2, a1);

console.log({ a3, ...b3 } = c3);
console.log((a0, { a3, ...b3 } = c3));
console.log((a0, { a3, ...b3 } = c3, a3));
