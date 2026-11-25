type F = () => number;
type C = new () => object;
type A = abstract new () => object;
type Predicate = (x: unknown) => x is number;
function f(): number { return 1; }
const arrow = (): number => 1;
const object = { method(): number { return 1; } };
