const log: string[] = [];
function key(name: string) {
  log.push(`key:${name}`);
  return name;
}
function dec(target: any, prop: string) {
  log.push(`${typeof target === "function" ? "static" : "proto"}:${prop}`);
}

class Foo {
  [key("a")] = 1;
  @dec declare [key("b")]: any;
  @dec static declare [key("c")]: any;
  [key("d")] = 2;
}

class Bar {
  @dec declare [key("e")]: any;
  @dec static declare [key("f")]: any;
}

const foo = new Foo() as any;
expect([foo.a, foo.d, "b" in foo, "c" in Foo]).toEqual([1, 2, false, false]);
expect("e" in new Bar()).toBe(false);
expect(log).toEqual([
  "key:a",
  "key:b",
  "key:c",
  "key:d",
  "proto:b",
  "static:c",
  "key:e",
  "key:f",
  "proto:e",
  "static:f",
]);
