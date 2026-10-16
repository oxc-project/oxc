const log = [];
const dec = (tag) => (value, ctx) => {
  log.push(`dec:${tag}`);
  ctx.addInitializer(function () {
    log.push(`extra:${tag}`);
  });
  if (ctx.kind === "field" || ctx.kind === "accessor") {
    const init = (v) => {
      log.push(`init:${tag}`);
      return v;
    };
    return ctx.kind === "field" ? init : { init };
  }
};

@dec("class")
class Foo {
  @dec("method") method() {}
  @dec("static-method") static sMethod() {}
  @dec("field") field = log.push("field-value");
  @dec("static-field") static sField = log.push("static-field-value");
  @dec("accessor") accessor acc = 1;
  @dec("static-accessor") static accessor sAcc = 1;
  static {
    log.push("static-block");
  }
  constructor() {
    log.push("constructor");
  }
}
log.push("after-class");
new Foo();

expect(log.join(",")).toBe([
  "dec:static-method", "dec:static-accessor", "dec:method", "dec:accessor",
  "dec:static-field", "dec:field", "dec:class",
  "extra:static-method",
  "static-field-value", "init:static-field", "extra:static-field",
  "init:static-accessor", "extra:static-accessor",
  "static-block",
  "extra:class",
  "after-class",
  "extra:method",
  "field-value", "init:field", "extra:field",
  "init:accessor", "extra:accessor",
  "constructor",
].join(","));
