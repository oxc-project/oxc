var _Foo, _Bar, _Throws;
class Foo {}
_Foo = Foo;
values.push(babelHelpers.checkInRHS(_Foo) === _Foo && _b !== void 0);
var _b = { _: 0 };
values.push(babelHelpers.checkInRHS(_Foo) === _Foo && _b !== void 0);
class Bar {}
_Bar = Bar;
values.push(babelHelpers.checkInRHS((calls++, _Bar)) === _Bar && _b2 !== void 0);
var _b2 = { _: void 0 };
values.push(babelHelpers.checkInRHS(_Bar) === _Bar && _b2 !== void 0);
class Throws {}
_Throws = Throws;
babelHelpers.checkInRHS(null) === _Throws && _b3 !== void 0;
var _b3 = { _: void 0 };
