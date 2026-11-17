const log = [];
const dec = (value, ctx) => {
  if (ctx.kind === "field") return (v) => v * 2;
  ctx.addInitializer(function () {
    log.push(`${ctx.name}:${this.constructor.name}`);
  });
};

class Base {
  constructor(x) {
    this.x = x;
  }
}

class Derived extends Base {
  @dec y = this.x + 1;
  @dec method() {}
  constructor(x) {
    if (x > 100) {
      super(x - 100);
    } else {
      super(x);
    }
    log.push(`ctor:${this.y}`);
  }
}

expect(new Derived(1).y).toBe(4);
expect(new Derived(101).y).toBe(4);
expect(log).toEqual(["method:Derived", "ctor:4", "method:Derived", "ctor:4"]);
