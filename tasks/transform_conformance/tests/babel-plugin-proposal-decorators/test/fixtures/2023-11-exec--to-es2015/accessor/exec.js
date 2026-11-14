const log = [];
const dec = (target, ctx) => {
  log.push(`${ctx.kind}:${ctx.name}:${ctx.static}`);
  return {
    get() { return target.get.call(this) * 2; },
    set(v) { target.set.call(this, v + 1); },
    init(v) { return v + 100; },
  };
};

class Foo {
  @dec accessor x = 1;
  @dec static accessor x = 2;
  accessor plain = 3;
  static accessor sPlain = 4;
}

const foo = new Foo();
expect(foo.x).toBe(202);
foo.x = 5;
expect(foo.x).toBe(12);
expect(Foo.x).toBe(204);
expect(foo.plain).toBe(3);
foo.plain = 30;
expect(foo.plain).toBe(30);
expect(Foo.sPlain).toBe(4);
expect(Object.keys(foo)).toEqual([]);
expect(typeof Object.getOwnPropertyDescriptor(Foo.prototype, "x").get).toBe("function");
expect(log).toEqual(["accessor:x:true", "accessor:x:false"]);
