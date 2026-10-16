const log = [];
let initThis;
const replace = (cls, ctx) => {
  log.push(`${ctx.kind}:${ctx.name}`);
  ctx.addInitializer(function () {
    initThis = this;
  });
  return class extends cls {
    replaced = true;
  };
};

@replace
class Foo {
  static self = this;
  static named = Foo;
  method() {
    return Foo;
  }
}

expect(new Foo().replaced).toBe(true);
expect(Foo.self).toBe(Foo);
expect(Foo.named).toBe(Foo);
expect(new Foo().method()).toBe(Foo);
expect(initThis).toBe(Foo);
expect(log).toEqual(["class:Foo"]);
