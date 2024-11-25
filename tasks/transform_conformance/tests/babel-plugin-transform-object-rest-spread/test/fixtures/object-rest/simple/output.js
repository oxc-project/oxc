const _excluded = ["f0"], _excluded2 = ["g0"], _excluded3 = ["h0"], _excluded4 = ["i0"], _excluded5 = ["j0"], _excluded6 = ["k0"], _excluded7 = ["l0"], _excluded8 = ["m0"], _excluded9 = ["n0"];
let a0 = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(foo), foo));
let _foo = foo(), b0 = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_foo), _foo));
let { c0, c1 } = foo, c2 = babelHelpers.objectWithoutProperties(foo, ["c0", "c1"]);
let _foo2 = foo(), { d0, d1 } = _foo2, d2 = babelHelpers.objectWithoutProperties(_foo2, ["d0", "d1"]);
let bar, _foo3 = foo(), { e0 } = _foo3, e1 = babelHelpers.objectWithoutProperties(_foo3, ["e0"]), baz;
for (let bar, _foo4 = foo(), { f0 } = _foo4, f1 = babelHelpers.objectWithoutProperties(_foo4, _excluded), baz;;) {}
for (let _ref in foo) {
	let { g0 } = _ref, g1 = babelHelpers.objectWithoutProperties(_ref, _excluded2);
}
for (let _ref2 of foo) {
	let { h0 } = _ref2, h1 = babelHelpers.objectWithoutProperties(_ref2, _excluded3);
}
for (let bar, _ioo = ioo(), { i0 } = _ioo, i1 = babelHelpers.objectWithoutProperties(_ioo, _excluded4), baz;;);
for (let _ref3 in foo) {
	let { j0 } = _ref3, j1 = babelHelpers.objectWithoutProperties(_ref3, _excluded5);
}
for (let _ref4 of foo) {
	let { k0 } = _ref4, k1 = babelHelpers.objectWithoutProperties(_ref4, _excluded6);
}
for (let bar, _loo = loo(), { l0 } = _loo, l1 = babelHelpers.objectWithoutProperties(_loo, _excluded7), baz;;) void 0;
for (let _ref5 in foo) {
	let { m0 } = _ref5, m1 = babelHelpers.objectWithoutProperties(_ref5, _excluded8);
	void 0;
}
for (let _ref6 of foo) {
	let { n0 } = _ref6, n1 = babelHelpers.objectWithoutProperties(_ref6, _excluded9);
	void 0;
}
let _key = key, { [_key]: x } = foo, rest = babelHelpers.objectWithoutProperties(foo, [_key].map(babelHelpers.toPropertyKey));
let _foo5 = foo(), _key2 = key1, { [_key2]: x2 } = _foo5, rest2 = babelHelpers.objectWithoutProperties(_foo5, [_key2].map(babelHelpers.toPropertyKey));
