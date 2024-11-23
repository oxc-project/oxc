var _objectWithoutProperties = require("@babel/runtime/helpers/objectWithoutProperties");
const defunct = { outer: { inner: {
	three: "three",
	four: "four"
} } };
const { outer: { inner: _ref } } = defunct, { three } = _ref, other = _objectWithoutProperties(_ref, ["three"]);
