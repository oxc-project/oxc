var _C;
class C {}
_C = C;
babelHelpers.defineProperty(C, "foo", babelHelpers.superPropGet(_C, "method", _C, 2)([]));
babelHelpers.defineProperty(C, "bar", babelHelpers.superPropGet(_C, "method", _C, 2)([1]));
babelHelpers.defineProperty(C, "bar", babelHelpers.superPropGet(_C, "method", _C, 2)([1, 2, 3]));
