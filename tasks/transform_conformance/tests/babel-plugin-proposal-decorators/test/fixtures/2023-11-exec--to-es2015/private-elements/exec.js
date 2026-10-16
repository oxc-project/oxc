const log = [];
const dec = (value, ctx) => {
  log.push(`${ctx.kind}:${ctx.name}:${ctx.private}`);
  if (ctx.kind === "method") return function () { return "wrapped:" + value.call(this); };
  if (ctx.kind === "field") return (v) => v * 10;
};

class Foo {
  @dec #field = 1;
  @dec #method() { return "m"; }
  @dec get #getter() { return "g"; }
  @dec accessor #acc = 2;
  @dec static #sMethod() { return "sm"; }
  read() {
    return [this.#field, this.#method(), this.#getter, this.#acc, Foo.#sMethod()];
  }
}

expect(new Foo().read()).toEqual([10, "wrapped:m", "g", 2, "wrapped:sm"]);
expect(log).toEqual([
  "method:#sMethod:true",
  "method:#method:true", "getter:#getter:true", "accessor:#acc:true",
  "field:#field:true",
]);
