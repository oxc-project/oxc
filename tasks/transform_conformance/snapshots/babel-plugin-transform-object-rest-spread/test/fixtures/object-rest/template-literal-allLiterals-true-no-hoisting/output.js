var _toPropertyKey = require("@babel/runtime/helpers/toPropertyKey");
var _objectWithoutProperties = require("@babel/runtime/helpers/objectWithoutProperties");
const example = () => {
	const input = {};
	const foo = "foo";
	var _input, _ref = `${foo}_bar`;
	_input = input, {[_ref]: country} = _input, rest = _objectWithoutProperties(_input, [_ref].map(_toPropertyKey)), _input;
};
