var _Foo;
function Bar() {
  return (_target) => {
    console.log(Bar.name);
  };
}
let Foo = class Foo {
  static {
    _Foo = this;
  }
  static foo = `${_Foo.name}`;
};
Foo = _Foo = babelHelpers.decorate([Bar()], Foo);
