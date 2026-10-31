var _staticBlock;
let Foo = (class Foo {
  static #_() {}
  static #_2 = _staticBlock = () => (this.foo = this.#_, this);
}, _staticBlock());
expect(Foo.foo).toBe(42);
