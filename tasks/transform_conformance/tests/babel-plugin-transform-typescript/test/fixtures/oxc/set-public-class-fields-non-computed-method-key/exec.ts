const log: string[] = [];
const k = (name: string) => (log.push(name), name);

class C {
  static [k("a")] = 1;
  "m"() {
    return "m";
  }
  1() {
    return 1;
  }
  get "g"() {
    return 2;
  }
  static "s"() {
    return "s";
  }
  [k("b")] = 3;
}

expect(log).toEqual(["a", "b"]);
expect(C.a).toBe(1);
expect(C.s()).toBe("s");
const c = new C();
expect(c.m()).toBe("m");
expect(c[1]()).toBe(1);
expect(c.g).toBe(2);
expect(c.b).toBe(3);
expect(Object.keys(c)).toEqual(["b"]);
expect(log).toEqual(["a", "b"]);
