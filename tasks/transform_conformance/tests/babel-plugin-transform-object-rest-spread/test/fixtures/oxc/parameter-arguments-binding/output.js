const _excluded = ["a"], _excluded2 = ["a"], _excluded3 = ["a"], _excluded4 = ["a"], _excluded5 = ["a"];
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
function lexicalLet(_ref4, _ref5 = void 0) {
  let { a } = _ref4, rest = babelHelpers.objectWithoutProperties(_ref4, _excluded2);
  let b = _ref5 === void 0 ? a : _ref5;
  let arguments = 1;
  return arguments;
}
function lexicalConst(_ref6, _ref7 = void 0, _ref8 = void 0) {
  let { a } = _ref6, rest = babelHelpers.objectWithoutProperties(_ref6, _excluded3);
  let b = _ref7 === void 0 ? a : _ref7;
  let c = _ref8 === void 0 ? arguments[0] : _ref8;
  return (() => {
    const arguments = 1;
    return [
      arguments,
      b,
      c.a
    ];
  })();
}
function lexicalDestructured(_ref9, _ref10 = void 0, _ref11 = void 0) {
  let { a } = _ref9, rest = babelHelpers.objectWithoutProperties(_ref9, _excluded4);
  let b = _ref10 === void 0 ? a : _ref10;
  let c = _ref11 === void 0 ? arguments[0] : _ref11;
  return (() => {
    let { value: arguments } = { value: 1 };
    return [
      arguments,
      b,
      c.a
    ];
  })();
}
function bodyFunction(_ref12, _ref13 = void 0, _ref14 = void 0) {
  let { a } = _ref12, rest = babelHelpers.objectWithoutProperties(_ref12, _excluded5);
  let b = _ref13 === void 0 ? a : _ref13;
  let c = _ref14 === void 0 ? arguments[0] : _ref14;
  return (() => {
    function arguments() {
      return 1;
    }
    return [
      arguments(),
      b,
      c.a
    ];
  })();
}
