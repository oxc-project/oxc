var _Foo;
function Bar() {
  return (_target) => {
    console.log(Bar.name);
  };
}
let Foo = _Foo = class Foo {};
Foo = _Foo = babelHelpers.decorate([Bar()], Foo);
babelHelpers.defineProperty(Foo, "foo", `${_Foo.name}`);