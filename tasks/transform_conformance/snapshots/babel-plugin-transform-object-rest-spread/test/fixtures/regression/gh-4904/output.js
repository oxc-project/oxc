const _excluded = ["b"];
var _objectWithoutProperties = require("@babel/runtime/helpers/objectWithoutProperties");
const _foo = foo(), { s } = _foo, t = _objectWithoutProperties(_foo, ["s"]);
const _bar = bar(), { s: _ref } = _bar, { q1 } = _ref, q2 = _objectWithoutProperties(_ref, ["q1"]), q3 = _objectWithoutProperties(_bar, ["s"]);
const { a } = foo((_ref2) => {
	let { b } = _ref2, c = _objectWithoutProperties(_ref2, _excluded);
	console.log(b, c);
});
