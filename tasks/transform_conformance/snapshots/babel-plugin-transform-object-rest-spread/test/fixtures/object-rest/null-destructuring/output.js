const _excluded = ["x"];
var _objectDestructuringEmpty = require("@babel/runtime/helpers/objectDestructuringEmpty");
var _extends = require("@babel/runtime/helpers/extends");
var _objectWithoutProperties = require("@babel/runtime/helpers/objectWithoutProperties");
expect(() => {
	var _ref = null, x = _extends({}, (_objectDestructuringEmpty(_ref), _ref));
}).toThrow(/null/);
expect(() => {
	var _ref2 = null, { x } = _ref2, y = _objectWithoutProperties(_ref2, _excluded);
}).toThrow(/null/);
