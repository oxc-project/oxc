const orig = Function.prototype.toString;
let i = 0;
Function.prototype.toString = function () {
  return `n:${i++}:${this.name}`;
};
let keys;
try {
  class C {
    [async () => {}] = 1;
    [() => {}] = 2;
    [class {}] = 3;
  }
  keys = Object.keys(new C());
} finally {
  Function.prototype.toString = orig;
}
expect(keys).toEqual(["n:0:", "n:1:", "n:2:"]);
