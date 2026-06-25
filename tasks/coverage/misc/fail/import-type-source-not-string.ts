// String literal expected in import type source
type A = typeof import(`react`);
type B = typeof import(A);
type C = typeof import(`${A} ${B}`);
type D = typeof import(typeof import('react'));
