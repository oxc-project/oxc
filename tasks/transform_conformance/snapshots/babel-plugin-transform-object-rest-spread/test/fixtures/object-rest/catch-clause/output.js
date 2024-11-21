const _excluded = ["a1"], _excluded2 = ["a2", "b2"], _excluded3 = ["c3"];
var _objectDestructuringEmpty = require("@babel/runtime/helpers/objectDestructuringEmpty");
var _extends = require("@babel/runtime/helpers/extends");
var _objectWithoutProperties = require("@babel/runtime/helpers/objectWithoutProperties");
try {} catch (_ref) {
	let a34 = _extends({}, (_objectDestructuringEmpty(_ref), _ref));
}
try {} catch (_ref2) {
	let { a1 } = _ref2, b1 = _objectWithoutProperties(_ref2, _excluded);
}
try {} catch (_ref3) {
	let { a2, b2 } = _ref3, c2 = _objectWithoutProperties(_ref3, _excluded2);
}
try {} catch (_ref4) {
	let { a2, b2, c2: _ref5 } = _ref4, { c3 } = _ref5, c4 = _objectWithoutProperties(_ref5, _excluded3);
}
try {} catch (a) {}
try {} catch ({ b }) {}
