var _staticBlock, _staticBlock2;
let Foo = ({ "Foo": class {
  static #_ = _staticBlock = () => ((() => {
    expect(this.name).toBe("Foo");
    if (false) use(this);
  })(), this);
} }["Foo"], _staticBlock());
let obj = { ["someName"]: ({ "someName": class {
  static x = ((_value) => (_staticBlock2 = () => ((() => {
    if (false) use(this);
    expect(this.name).toBe("someName");
  })(), this), _value))(expect(this.name).toBe("someName"));
} }["someName"], _staticBlock2()) };
expect(Foo.name).toBe("Foo");
expect(obj.someName.name).toBe("someName");
