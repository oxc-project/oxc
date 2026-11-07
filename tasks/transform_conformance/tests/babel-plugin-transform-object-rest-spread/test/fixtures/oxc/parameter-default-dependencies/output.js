const _excluded = ["a"], _excluded2 = ["a"], _excluded3 = ["a"], _excluded4 = ["a"], _excluded5 = ["a"], _excluded6 = ["a"], _excluded7 = ["a"], _excluded8 = ["a"], _excluded9 = ["a"], _excluded10 = ["a"], _excluded11 = ["a"], _excluded12 = ["a"], _excluded13 = ["a"];
function f(_ref, _ref2 = void 0) {
  let { a } = _ref, x = babelHelpers.objectWithoutProperties(_ref, _excluded);
  let b = _ref2 === void 0 ? a : _ref2;
  return b;
}
const arrow = (_ref3, _ref4 = void 0) => {
  let { a } = _ref3, rest = babelHelpers.objectWithoutProperties(_ref3, _excluded2);
  let b = _ref4 === void 0 ? a : _ref4;
  return [b, rest];
};
function ordered(_ref5, _ref6 = void 0, _ref7 = void 0, ..._ref8) {
  let { a } = _ref5, rest = babelHelpers.objectWithoutProperties(_ref5, _excluded3);
  let b = _ref6 === void 0 ? a : _ref6;
  let { c } = _ref7 === void 0 ? { c: b } : _ref7;
  let tail = _ref8;
  return [c, tail];
}
function forward(_ref9 = void 0, _ref10) {
  let b = _ref9 === void 0 ? a : _ref9;
  let { a } = _ref10, rest = babelHelpers.objectWithoutProperties(_ref10, _excluded4);
  return b;
}
function nested(_ref11, _ref12, ..._unused) {
  let { a } = _ref11, rest = babelHelpers.objectWithoutProperties(_ref11, _excluded5);
  let { b = a } = _ref12;
  return b;
}
function computed(_ref13, _ref14, ..._unused2) {
  let { a } = _ref13, rest = babelHelpers.objectWithoutProperties(_ref13, _excluded6);
  let { [a]: b } = _ref14;
  return b;
}
function closure(_ref15, _ref16 = void 0, _ref17 = void 0) {
  let { a } = _ref15, rest = babelHelpers.objectWithoutProperties(_ref15, _excluded7);
  let selected = _ref16 === void 0 ? a : _ref16;
  let read = _ref17 === void 0 ? { ["read"]: () => a }.read : _ref17;
  return read();
}
const outside = 7;
function shadowed(_ref18, _ref19 = void 0, _ref20 = void 0) {
  let { a } = _ref18, rest = babelHelpers.objectWithoutProperties(_ref18, _excluded8);
  let b = _ref19 === void 0 ? a : _ref19;
  let read = _ref20 === void 0 ? { ["read"]: () => outside }.read : _ref20;
  return (() => {
    let outside = 9;
    return read();
  })();
}
function redeclared(_ref21, _ref22 = void 0, _ref23 = void 0) {
  let { a } = _ref21, rest = babelHelpers.objectWithoutProperties(_ref21, _excluded9);
  let selected = _ref22 === void 0 ? a : _ref22;
  let read = _ref23 === void 0 ? { ["read"]: () => a }.read : _ref23;
  return ((a) => {
    var a = 2;
    return [a, read()];
  })(a);
}
function callTime(_ref24, _ref25 = void 0, _ref26 = void 0) {
  let _arguments = arguments;
  let { a } = _ref24, rest = babelHelpers.objectWithoutProperties(_ref24, _excluded10);
  let selected = _ref25 === void 0 ? a : _ref25;
  let read = _ref26 === void 0 ? { ["read"]: () => a }.read : _ref26;
  return function* () {
    yield read();
  }.apply(this, arguments);
}
function bodyEval(_ref27, _ref28 = void 0) {
  let { a } = _ref27, rest = babelHelpers.objectWithoutProperties(_ref27, _excluded11);
  let b = _ref28 === void 0 ? a : _ref28;
  return (() => {
    let hidden = 3;
    return eval("hidden") + b;
  })();
}
function shadowAsync(_ref29, _ref30 = void 0, _ref31 = void 0, _ref32 = void 0) {
  try {
    let { a } = _ref29, rest = babelHelpers.objectWithoutProperties(_ref29, _excluded12);
    let selected = _ref30 === void 0 ? a : _ref30;
    let read = _ref31 === void 0 ? { ["read"]: () => a }.read : _ref31;
    let other = _ref32 === void 0 ? { ["other"]: () => outside }.other : _ref32;
    return (async () => {
      let outside = 2;
      return 3;
    })();
  } catch (_error) {
    return (async () => {
      throw _error;
    })();
  }
}
function argumentIdentity(_ref33, _ref34 = void 0, _ref35 = void 0) {
  let _arguments2 = arguments;
  let { a } = _ref33, rest = babelHelpers.objectWithoutProperties(_ref33, _excluded13);
  let b = _ref34 === void 0 ? a : _ref34;
  let saved = _ref35 === void 0 ? _arguments2 : _ref35;
  return function* () {
    yield saved === _arguments2;
  }.apply(this, arguments);
}
