const _excluded = ["a"];
function f(_ref, _ref2 = void 0, _ref3 = void 0) {
  let { a } = _ref, rest = babelHelpers.objectWithoutProperties(_ref, _excluded);
  let b = _ref2 === void 0 ? a : _ref2;
  let c = _ref3 === void 0 ? arguments[0] : _ref3;
  return ((arguments) => {
    var arguments;
    return [
      arguments[0].a,
      c.a,
      b
    ];
  })(arguments);
}
