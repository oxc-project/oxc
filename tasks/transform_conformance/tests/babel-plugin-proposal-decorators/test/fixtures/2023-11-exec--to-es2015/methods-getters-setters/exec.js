const log = [];
const dec = (value, ctx) => {
  log.push(`${ctx.kind}:${String(ctx.name)}:${ctx.static}:${ctx.private}`);
  if (ctx.kind === "method") return function (...args) { return "wrapped:" + value.apply(this, args); };
  if (ctx.kind === "getter") return function () { return value.call(this) + 1; };
  if (ctx.kind === "setter") return function (v) { value.call(this, v * 2); };
};
const sym = Symbol("sym");

class Foo {
  #v = 1;
  @dec method() { return "m"; }
  @dec static sMethod() { return "sm"; }
  @dec [sym]() { return "symbol"; }
  @dec get value() { return this.#v; }
  @dec set value(v) { this.#v = v; }
  @dec static get sValue() { return 10; }
}

const foo = new Foo();
expect(foo.method()).toBe("wrapped:m");
expect(Foo.sMethod()).toBe("wrapped:sm");
expect(foo[sym]()).toBe("wrapped:symbol");
foo.value = 5;
expect(foo.value).toBe(11);
expect(Foo.sValue).toBe(11);
expect(log).toEqual([
  "method:sMethod:true:false",
  "getter:sValue:true:false",
  "method:method:false:false",
  "method:Symbol(sym):false:false",
  "getter:value:false:false",
  "setter:value:false:false",
]);
