var _staticBlock;
let Foo = (class Foo {
  static #bar = 21;
  static qux = ((_value) => (_staticBlock = () => (this.qux2 = this.qux, this), _value))(((() => {
    this.foo = this.#bar;
    this.qux1 = this.qux;
  })(), (() => {
    if (foo) bar;
  })(), 21));
}, _staticBlock());
