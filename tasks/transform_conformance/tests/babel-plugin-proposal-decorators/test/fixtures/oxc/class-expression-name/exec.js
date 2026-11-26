const names = [];
const dec = (cls, ctx) => {
  names.push(ctx.name);
  names.push(cls.name);
};

const A = @dec class {};
let B;
B = @dec class {};
const { C = @dec class {} } = {};
const [D = @dec class {}] = [];
const obj = { E: @dec class {} };
class Host {
  F = @dec class {};
  accessor G = @dec class {};
  static H = @dec class {};
}
new Host();
function fn(I = @dec class {}) {}
fn();

expect(names).toEqual([
  "A", "A", "B", "B", "C", "C", "D", "D", "E", "E",
  "H", "H", "F", "F", "G", "G", "I", "I",
]);
