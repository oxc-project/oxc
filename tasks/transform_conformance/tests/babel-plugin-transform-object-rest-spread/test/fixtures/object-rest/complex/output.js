let [_ref] = z, a0 = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref), _ref));
let [_ref2, _ref3] = z, b0 = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref2), _ref2)), b1 = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref3), _ref3));
let { c0: _ref4 } = foo, c1 = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref4), _ref4));
let { d0: _ref5 } = foo, { d1 } = _ref5, d2 = babelHelpers.objectWithoutProperties(_ref5, ["d1"]);
let { e0: _ref7 } = foo, { e1: _ref6 } = _ref7, e4 = babelHelpers.objectWithoutProperties(_ref7, ["e1"]), { e2 } = _ref6, e3 = babelHelpers.objectWithoutProperties(_ref6, ["e2"]);
let { f0: _ref9 } = foo, { f1: _ref8 } = _ref9, f4 = babelHelpers.objectWithoutProperties(_ref9, ["f1"]), { f2 } = _ref8, f3 = babelHelpers.objectWithoutProperties(_ref8, ["f2"]), f5 = babelHelpers.objectWithoutProperties(foo, ["f0"]);
let [{ g0: _ref11 }, { g5: [_ref13] }] = goo, { g1: _ref10 } = _ref11, g4 = babelHelpers.objectWithoutProperties(_ref11, ["g1"]), { g2 } = _ref10, g3 = babelHelpers.objectWithoutProperties(_ref10, ["g2"]), { g6: [_ref12] } = _ref13, g9 = babelHelpers.objectWithoutProperties(_ref13, ["g6"]), { g7 } = _ref12, g8 = babelHelpers.objectWithoutProperties(_ref12, ["g7"]);
