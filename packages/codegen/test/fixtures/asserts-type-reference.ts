declare let x: any;
type asserts = string;

type A1 = (asserts) extends string ? 1 : 2;
type A2 = keyof (asserts) extends string ? 1 : 2;
type A3 = number | (asserts) extends string ? 1 : 2;
type A4 = number & (asserts) extends string ? 1 : 2;
type A5 = (asserts)[] extends string ? 1 : 2;
type A6 = asserts<string> extends string ? 1 : 2;

type M1 = { [K in (asserts) as K]: 1 };
type M2 = { [K in (asserts)]: 1 };
type M3 = { [K in () => (asserts) as K]: 1 };
type M4 = { [K in string extends 1 ? 2 : (asserts) as K]: 1 };

const e1 = (x satisfies (asserts)) in x;
const e2 = (x as (asserts)) instanceof Object;
const e3 = x as (asserts);
for ((x as (asserts)) of x);
