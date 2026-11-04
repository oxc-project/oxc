var _staticBlock;
let Foo = (class Foo {
  static bar = ((_value) => (_staticBlock = () => (this.foo = this.bar, this), _value))(42);
}, _staticBlock());
expect(Foo.foo).toBe(42);
