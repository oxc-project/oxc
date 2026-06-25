class A {}
let Foo = class Foo {
  static Error1 = class extends Error {};
  static Error2 = class extends Error {};
  constructor(a) {}
};
Foo = babelHelpers.decorate([dec(), babelHelpers.decorateMetadata("design:paramtypes", [typeof A === "undefined" ? Object : A])], Foo);
export { Foo };
