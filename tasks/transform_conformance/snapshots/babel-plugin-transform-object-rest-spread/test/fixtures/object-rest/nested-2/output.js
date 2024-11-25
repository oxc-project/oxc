var _objectWithoutProperties = require("@babel/runtime/helpers/objectWithoutProperties");
const test = { foo: { bar: { baz: { a: {
	x: 1,
	y: 2,
	z: 3
} } } } };
const { foo: { bar: { baz: { a: _ref } } } } = test, { x } = _ref, other = _objectWithoutProperties(_ref, ["x"]);
