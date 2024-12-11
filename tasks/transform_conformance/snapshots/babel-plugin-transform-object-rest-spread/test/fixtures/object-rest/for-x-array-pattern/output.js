const _excluded = ["a"], _excluded2 = ["a"], _excluded3 = ["a"];
for (const [_ref] of []) {
	let { a } = _ref, b = babelHelpers.objectWithoutProperties(_ref, _excluded);
}
for (var _ref2 of []) {
	var _ref3;
	var _ref4;
	[_ref3] = _ref2, _ref4 = _ref3, {a} = _ref4, b = babelHelpers.objectWithoutProperties(_ref4, _excluded2), _ref4;
}
async function a() {
	for await (var _ref5 of []) {
		var _ref6;
		var _ref7;
		[_ref6] = _ref5, _ref7 = _ref6, {a} = _ref7, b = babelHelpers.objectWithoutProperties(_ref7, _excluded3), _ref7;
	}
}
for ([{a}] in {}) {}
for ([{a}] of []) {}
async function a() {
	for await ([{a}] of []) {}
}
for ([a,...b] in {}) {}
for ([a,...b] of []) {}
async function a() {
	for await ([a,...b] of []) {}
}
