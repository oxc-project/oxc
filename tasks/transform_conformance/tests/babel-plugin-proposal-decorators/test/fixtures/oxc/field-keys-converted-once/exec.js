// Field keys of a decorated class are converted once, in element order, while the class is
// defined, even though the fields are defined later.
const log = [];
const key = (name) => ({ toString() { log.push(name); return name; } });
const a = key("a"), b = key("b"), c = key("c"), e = key("e"), f = key("f");
const dec = (value) => value;

class X {
  [a] = 1;
  [b]() {}
  static [c] = 2;
  @dec [e]() {}
  accessor [f] = 3;
  target = new.target;
}
expect(log).toEqual(["a", "b", "c", "e", "f"]);
expect(Object.keys(new X())).toEqual(["a", "target"]);
expect(Object.keys(new X())).toEqual(["a", "target"]);
expect(new X().f).toBe(3);
expect(new X().target).toBeUndefined();
expect(X.c).toBe(2);
expect(log).toEqual(["a", "b", "c", "e", "f"]);
