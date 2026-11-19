// ClassDefinitionEvaluation (tc39/ecma262#2417): static private methods are installed on the
// original class before class decorators run; static fields, static accessor storage, static
// blocks and extra initializers then use the replacement.
let Original;
function replace(C) {
  Original = C;
  return class Replacement extends C {};
}
let extraThis;
function extra(_, ctx) {
  ctx.addInitializer(function () {
    extraThis = this;
  });
}
let hasP, hasField, hasAccessor, blockThis;

@replace
class A {
  static #p() { return 1; }
  static get #g() { return 2; }
  static #field = 3;
  static accessor #a = 4;
  @extra static #r() { return 5; }
  static callInner() { return A.#p(); }
  static callThis() { return this.#p(); }
  static getter() { return A.#g; }
  static decorated() { return A.#r(); }
  static field() { return A.#field; }
  static {
    blockThis = this;
    hasP = (o) => #p in o;
    hasField = (o) => #field in o;
    hasAccessor = (o) => #a in o;
  }
}

expect(A).not.toBe(Original);
expect(Object.getPrototypeOf(A)).toBe(Original);
// Step 122: static private methods and accessors live on the original class only.
expect(() => A.callInner()).toThrow(TypeError);
expect(() => A.callThis()).toThrow(TypeError);
expect(() => A.getter()).toThrow(TypeError);
expect(() => A.decorated()).toThrow(TypeError);
expect(Original.callThis()).toBe(1);
expect(hasP(A)).toBe(false);
expect(hasP(Original)).toBe(true);
expect(hasAccessor(Original)).toBe(true);
// Steps 131-132: static method extra initializers see the replacement.
expect(extraThis).toBe(A);
// Steps 136-140: static fields and static blocks use the replacement.
expect(A.field()).toBe(3);
expect(hasField(A)).toBe(true);
expect(hasField(Original)).toBe(false);
expect(blockThis).toBe(A);
