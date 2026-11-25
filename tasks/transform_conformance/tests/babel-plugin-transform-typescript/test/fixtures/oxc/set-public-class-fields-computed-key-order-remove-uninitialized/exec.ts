const log: string[] = [];
const k = (name: string) => (log.push(name), name);

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
expect(Object.keys(d)).toEqual(["a", "lit", "f", "plain"]);
expect(d).toEqual({ a: 1, lit: 2, f: 4, plain: 6 });
expect(Object.keys(D)).toEqual(["c", "g"]);
expect(D.c).toBe(3);
expect(D.g).toBe(5);
expect(typeof D.prototype.e).toBe("function");
