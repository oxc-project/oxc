const _excluded = ["a"], _excluded2 = ["a"], _excluded3 = ["a"];
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
function generator(_ref4, _ref5 = void 0) {
  let _arguments = arguments;
  let { a } = _ref4, rest = babelHelpers.objectWithoutProperties(_ref4, _excluded2);
  let b = _ref5 === void 0 ? a : _ref5;
  let _param = _arguments;
  return function* () {
    var arguments = _param;
    var arguments;
    yield [arguments[0].a, b];
  }.apply(this, arguments);
}
function bound(_ref6, _ref7 = void 0) {
  let _arguments2 = arguments;
  let { a } = _ref6, rest = babelHelpers.objectWithoutProperties(_ref6, _excluded3);
  let _arguments3 = _ref7 === void 0 ? a : _ref7;
  return function* () {
    yield _arguments3;
  }.apply(this, arguments);
}
