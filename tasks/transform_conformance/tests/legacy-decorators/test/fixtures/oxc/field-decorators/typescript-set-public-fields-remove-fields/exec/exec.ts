// Whether a decorated field without an initializer stays an own property of the instance.
const OWN_UNINITIALIZED_FIELD = false;

const log: string[] = [];
const key = (name: string) => (log.push(`key:${name}`), name);
const dec = (name: string) => {
  log.push(`eval:${name}`);
  return (target: any, prop: PropertyKey) => {
    log.push(`apply:${name}:${typeof target === "function" ? "static" : "proto"}:${String(prop)}`);
  };
};
// Defines an accessor on the prototype, which an own field would shadow.
const accessor = (target: any, prop: PropertyKey) => {
  Object.defineProperty(target, prop, {
    get() {
      return "accessor";
    },
    set(_value) {},
    configurable: true,
  });
};

class A {
  @dec("a") [key("a")]: number;
  @dec("b") [key("b")] = 1;
  [key("m")]() {}
  @dec("c") static [key("c")]: number;
  @dec("d") static [key("d")] = 2;
  @dec("e") e() {}
  @accessor field: string;
  constructor(public p = 3) {}
}

expect(log).toEqual([
  "key:a",
  "key:b",
  "key:m",
  "key:c",
  "key:d",
  "eval:a",
  "apply:a:proto:a",
  "eval:b",
  "apply:b:proto:b",
  "eval:e",
  "apply:e:proto:e",
  "eval:c",
  "apply:c:static:c",
  "eval:d",
  "apply:d:static:d",
]);

const a = new A();
expect(a.b).toBe(1);
expect(A.d).toBe(2);
expect(a.p).toBe(3);
expect(Object.hasOwn(a, "field")).toBe(OWN_UNINITIALIZED_FIELD);
expect(a.field).toBe(OWN_UNINITIALIZED_FIELD ? undefined : "accessor");
