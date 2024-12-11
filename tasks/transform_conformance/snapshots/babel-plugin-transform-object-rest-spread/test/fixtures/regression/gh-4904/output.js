const _excluded = ["b"];
const _foo = foo(), { s } = _foo, t = babelHelpers.objectWithoutProperties(_foo, ["s"]);
const _bar = bar(), { s: _ref } = _bar, { q1 } = _ref, q2 = babelHelpers.objectWithoutProperties(_ref, ["q1"]), q3 = babelHelpers.objectWithoutProperties(_bar, ["s"]);
const { a } = foo((_ref2) => {
	let { b } = _ref2, c = babelHelpers.objectWithoutProperties(_ref2, _excluded);
	console.log(b, c);
});
