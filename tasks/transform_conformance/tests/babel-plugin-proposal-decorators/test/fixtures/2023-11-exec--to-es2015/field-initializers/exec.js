const log = [];
const dec = (tag) => (value, ctx) => {
  expect(value).toBe(undefined);
  log.push(`${tag}:${String(ctx.name)}`);
  ctx.addInitializer(function () {
    log.push(`extra:${tag}`);
  });
  return function (v) {
    log.push(`init:${tag}:${v}`);
    return v === undefined ? tag : v * 10;
  };
};
const key = Symbol("key");

class Foo {
  @dec("a") a = 1;
  @dec("b") b;
  @dec("c") [key] = 3;
  @dec("s") static s = 4;
  plain = this.a + 1;
}

const foo = new Foo();
expect(foo.a).toBe(10);
expect(foo.b).toBe("b");
expect(foo[key]).toBe(30);
expect(foo.plain).toBe(11);
expect(Foo.s).toBe(40);
expect(log).toEqual([
  "s:s", "a:a", "b:b", "c:Symbol(key)",
  "init:s:4", "extra:s",
  "init:a:1", "extra:a", "init:b:undefined", "extra:b", "init:c:3", "extra:c",
]);
