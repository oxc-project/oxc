const _excluded = ["X"];
(_ref, a = R) => {
	let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref), _ref));
};
(_ref2, { a = { Y } }) => {
	let { X: Y } = _ref2, R = babelHelpers.objectWithoutProperties(_ref2, _excluded);
};
(a = R, _ref3) => {
	let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref3), _ref3));
};
(_ref4, e, c = 2, a = R, f = q) => {
	let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref4), _ref4));
	let q;
};
(_ref5, a = f(R)) => {
	let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref5), _ref5));
};
(_ref6, { [R.key]: a = 42 }) => {
	let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref6), _ref6));
};
(_ref7, { a = { R: b } }) => {
	let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref7), _ref7));
};
(_ref8, { a = (R) => R } = { b: (R) => R }) => {
	let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref8), _ref8));
};
