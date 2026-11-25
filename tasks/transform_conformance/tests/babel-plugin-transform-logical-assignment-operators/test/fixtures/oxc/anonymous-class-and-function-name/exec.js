expect(
  (class {
    static get p() {
      return this.name || "empty";
    }
  }).p ||= 1,
).toBe("empty");
const orig = Function.prototype.toString;
Function.prototype.toString = function () {
  return `n:${this.name}`;
};
const obj = {};
try {
  obj[() => {}] ??= 1;
} finally {
  Function.prototype.toString = orig;
}
expect(Object.keys(obj)).toEqual(["n:"]);
