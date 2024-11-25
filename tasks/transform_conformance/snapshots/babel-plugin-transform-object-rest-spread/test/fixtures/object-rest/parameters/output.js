const _excluded = ["a1"], _excluded2 = ["a2", "b2"], _excluded3 = ["a5"], _excluded4 = ["a3"], _excluded5 = ["ba1"], _excluded6 = ["a3", "b2"], _excluded7 = ["ba1"], _excluded8 = ["a1"], _excluded9 = ["a1"];
var _objectDestructuringEmpty = require("@babel/runtime/helpers/objectDestructuringEmpty");
var _extends = require("@babel/runtime/helpers/extends");
var _objectWithoutProperties = require("@babel/runtime/helpers/objectWithoutProperties");
function a(_ref) {
	let a34 = _extends({}, (_objectDestructuringEmpty(_ref), _ref));
}
function a2(_ref2) {
	let { a1 } = _ref2, b1 = _objectWithoutProperties(_ref2, _excluded);
}
function a3(_ref3) {
	let { a2, b2 } = _ref3, c2 = _objectWithoutProperties(_ref3, _excluded2);
}
function a4(_ref4, _ref5) {
	let { a5 } = _ref5, c5 = _objectWithoutProperties(_ref5, _excluded3);
	let { a3 } = _ref4, c3 = _objectWithoutProperties(_ref4, _excluded4);
}
function a5(_ref6) {
	let { a3, b2: _ref7 } = _ref6, { ba1 } = _ref7, ba2 = _objectWithoutProperties(_ref7, _excluded5), c3 = _objectWithoutProperties(_ref6, _excluded6);
}
function a6(_ref8) {
	let { a3, b2: _ref9 } = _ref8, { ba1 } = _ref9, ba2 = _objectWithoutProperties(_ref9, _excluded7);
}
function a7(_ref10 = {}) {
	let { a1 = 1 } = _ref10, b1 = _objectWithoutProperties(_ref10, _excluded8);
}
function a8([_ref11]) {
	let a1 = _extends({}, (_objectDestructuringEmpty(_ref11), _ref11));
}
function a9([_ref12]) {
	let { a1 } = _ref12, a2 = _objectWithoutProperties(_ref12, _excluded9);
}
function a10([a1, _ref13]) {
	let a2 = _extends({}, (_objectDestructuringEmpty(_ref13), _ref13));
}
function b(a) {}
function b2(a, ...b) {}
function b3({ b }) {}
