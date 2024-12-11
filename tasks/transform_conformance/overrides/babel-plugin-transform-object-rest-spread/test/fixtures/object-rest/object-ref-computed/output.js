var key, x, y, z;
key = 1;
var _ = { 1: {
	a: 1,
	y: 1
} }, { [key++]: _ref } = _, { y } = _ref, x = babelHelpers.objectWithoutProperties(_ref, ["y"]);
expect(x).toEqual({ a: 1 });
expect(key).toBe(2);
expect(y).toBe(1);
key = 1;
var _$ = {
	2: {
		y: 2,
		z: 3
	},
	3: {
		y: 2,
		z: 3
	}
}, { [++key]: _ref2, [++key]: _ref3 } = _$, { y } = _ref2, rest_y = babelHelpers.objectWithoutProperties(_ref2, ["y"]), { z } = _ref3, rest_z = babelHelpers.objectWithoutProperties(_ref3, ["z"]);
expect(y).toBe(2);
expect(rest_y).toEqual({ z: 3 });
expect(z).toBe(3);
expect(rest_z).toEqual({ y: 2 });
expect(key).toBe(3);
