// Examples of incorrect code for no-duplicate-type-constituents rule

type T1 = 'A' | 'A';

type T2 = A | A | B;

type T3 = { a: string } & { a: string };

type T4 = [A, A];

type T5 =
  | 'foo'
  | 'bar'
  | 'foo';