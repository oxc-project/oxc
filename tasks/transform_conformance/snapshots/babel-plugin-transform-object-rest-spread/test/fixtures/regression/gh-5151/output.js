var _objectWithoutProperties = require("@babel/runtime/helpers/objectWithoutProperties");
var _objectDestructuringEmpty = require("@babel/runtime/helpers/objectDestructuringEmpty");
var _extends = require("@babel/runtime/helpers/extends");
const { x } = a, y = _objectWithoutProperties(a, ["x"]), z = foo(y);
const s = _extends({}, (_objectDestructuringEmpty(r), r)), t = foo(s);
var l = foo(), _bar = bar(), { m: _ref } = _bar, { n } = _ref, o = _objectWithoutProperties(_ref, ["n"]), p = _objectWithoutProperties(_bar, ["m"]), q = baz();
