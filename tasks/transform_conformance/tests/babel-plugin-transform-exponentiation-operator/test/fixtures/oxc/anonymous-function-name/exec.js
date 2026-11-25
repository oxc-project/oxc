const seen = [];
Object.defineProperty(Function.prototype, "p", {
  configurable: true,
  get() {
    seen.push(this.name);
    return 2;
  },
  set() {},
});
const orig = Function.prototype.toString;
Function.prototype.toString = function () {
  return `n:${this.name}`;
};
const obj = { "n:": 3 };
try {
  (() => {}).p **= 2;
  obj[() => {}] **= 2;
} finally {
  delete Function.prototype.p;
  Function.prototype.toString = orig;
}
expect(seen).toEqual([""]);
expect(obj["n:"]).toBe(9);
