const log: string[] = [];
const k = (name: string) => (log.push(name), name);

class Foo {
  [k("a")] = 1;
  declare [k("f")]: any;
  static declare [k("s")]: any;
  declare x: number;
  [k("g")]: any;
  [k("h")] = 2;
}

expect(log).toEqual(["a", "g", "h"]);
const foo = new Foo();
expect(Object.keys(foo)).toEqual(["a", "h"]);
expect(Object.keys(Foo)).toEqual([]);
