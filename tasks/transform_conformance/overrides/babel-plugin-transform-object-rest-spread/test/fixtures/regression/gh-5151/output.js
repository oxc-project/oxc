const { x } = a, y = babelHelpers.objectWithoutProperties(a, ["x"]), z = foo(y);
const s = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(r), r)), t = foo(s);
var l = foo(), _bar = bar(), { m: _ref } = _bar, { n } = _ref, o = babelHelpers.objectWithoutProperties(_ref, ["n"]), p = babelHelpers.objectWithoutProperties(_bar, ["m"]), q = baz();
