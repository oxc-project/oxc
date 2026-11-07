const _excluded = ["X"];
(_ref, _ref2 = void 0) => {
  let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref), _ref));
  let a = _ref2 === void 0 ? R : _ref2;
};
(_ref3, _ref4, ..._unused) => {
  let { X: Y } = _ref3, R = babelHelpers.objectWithoutProperties(_ref3, _excluded);
  let { a = { Y } } = _ref4;
};
(_ref5 = void 0, _ref6) => {
  let a = _ref5 === void 0 ? R : _ref5;
  let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref6), _ref6));
};
(_ref7, _ref8, _ref9 = void 0, _ref10 = void 0, _ref11 = void 0) => {
  let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref7), _ref7));
  let e = _ref8;
  let c = _ref9 === void 0 ? 2 : _ref9;
  let a = _ref10 === void 0 ? R : _ref10;
  let f = _ref11 === void 0 ? q : _ref11;
  return (() => {
    let q;
  })();
};
(_ref12, _ref13 = void 0) => {
  let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref12), _ref12));
  let a = _ref13 === void 0 ? f(R) : _ref13;
};
(_ref14, _ref15, ..._unused2) => {
  let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref14), _ref14));
  let { [R.key]: a = 42 } = _ref15;
};
(_ref16, { a = { R: b } }) => {
  let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref16), _ref16));
};
(_ref17, { a = (R) => R } = { b: (R) => R }) => {
  let R = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref17), _ref17));
};
