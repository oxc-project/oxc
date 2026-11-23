const log: string[] = [];
const k = (name: string) => (log.push(name), name);
const getY = () => "yKey";
const bound = "b";

class C {
  "x" = 1;
  1 = 2;
  [getY()] = 4;
  [bound] = 5;
  z = 6;
}

const c = new C();
expect(Object.keys(c)).toEqual(["1", "x", "yKey", "b", "z"]);
expect(c).toEqual({ 1: 2, x: 1, yKey: 4, b: 5, z: 6 });

class D {
  [k("a")] = 1;
  "lit" = 2;
  [k("b")]: number;
  static [k("c")] = 3;
  [k("d")]: number;
  [k("e")]() {}
  [k("f")] = 4;
  static [k("g")] = 5;
  plain = 6;
}

expect(log).toEqual(["a", "b", "c", "d", "e", "f", "g"]);
const d = new D();
expect(log).toEqual(["a", "b", "c", "d", "e", "f", "g"]);
expect(d.a).toBe(1);
expect(d.lit).toBe(2);
expect(d.f).toBe(4);
expect(d.plain).toBe(6);
expect(D.c).toBe(3);
expect(D.g).toBe(5);
expect(typeof D.prototype.e).toBe("function");
expect(Object.keys(d)).not.toContain("undefined");
expect(Object.keys(d)).not.toContain("null");
expect(Object.keys(D)).toEqual(["c", "g"]);
