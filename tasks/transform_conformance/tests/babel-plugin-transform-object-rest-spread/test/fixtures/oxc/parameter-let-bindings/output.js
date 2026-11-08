const _excluded = ["a"], _excluded2 = ["a"], _excluded3 = ["a"], _excluded4 = ["a"], _excluded5 = ["a"], _excluded6 = ["a"], _excluded7 = ["a"], _excluded8 = ["a"], _excluded9 = ["a"], _excluded10 = ["a"], _excluded11 = ["a"];
function f(_ref, _ref2 = void 0, _ref3 = void 0) {
  let { a } = _ref, rest = babelHelpers.objectWithoutProperties(_ref, _excluded);
  let b = _ref2 === void 0 ? a : _ref2;
  let _let = _ref3 === void 0 ? 3 : _ref3;
  return {
    let: _let,
    b
  };
}
function named(_ref4, _ref5 = void 0, _ref6 = void 0) {
  let { a } = _ref4, rest = babelHelpers.objectWithoutProperties(_ref4, _excluded2);
  let b = _ref5 === void 0 ? a : _ref5;
  let _let2 = _ref6 === void 0 ? { ["let"]: () => a }.let : _ref6;
  return _let2.name;
}
function destructured(_ref7, _ref8 = void 0, _ref9 = void 0) {
  let { a } = _ref7, rest = babelHelpers.objectWithoutProperties(_ref7, _excluded3);
  let b = _ref8 === void 0 ? a : _ref8;
  let { let: _let3 = 3 } = _ref9 === void 0 ? {} : _ref9;
  return _let3;
}
function rest(_ref10, _ref11 = void 0, ..._ref12) {
  let { a } = _ref10, other = babelHelpers.objectWithoutProperties(_ref10, _excluded4);
  let b = _ref11 === void 0 ? a : _ref11;
  let _let4 = _ref12;
  return _let4;
}
function before(_ref13, _ref14, _ref15 = void 0) {
  let _let5 = _ref13;
  let { a } = _ref14, rest = babelHelpers.objectWithoutProperties(_ref14, _excluded5);
  let b = _ref15 === void 0 ? a : _ref15;
  return _let5;
}
function tdz(_ref16, _ref17 = void 0, _ref18 = void 0) {
  let { a } = _ref16, rest = babelHelpers.objectWithoutProperties(_ref16, _excluded6);
  let b = _ref17 === void 0 ? a : _ref17;
  let _let6 = _ref18 === void 0 ? _let6 : _ref18;
  return _let6;
}
function copied(_ref19, _ref20 = void 0, _ref21 = void 0) {
  let { a } = _ref19, rest = babelHelpers.objectWithoutProperties(_ref19, _excluded7);
  let b = _ref20 === void 0 ? a : _ref20;
  let _let7 = _ref21 === void 0 ? 3 : _ref21;
  return ((let) => {
    var let = 4;
    return let;
  })(_let7);
}
function nested(_ref22, _ref23 = void 0, _ref24 = void 0) {
  let { a } = _ref22, rest = babelHelpers.objectWithoutProperties(_ref22, _excluded8);
  let b = _ref23 === void 0 ? a : _ref23;
  let { let: _let8 = { ["let"]: function() {} }.let } = _ref24 === void 0 ? {} : _ref24;
  return _let8.name;
}
function array(_ref25, _ref26 = void 0, _ref27 = void 0) {
  let { a } = _ref25, rest = babelHelpers.objectWithoutProperties(_ref25, _excluded9);
  let b = _ref26 === void 0 ? a : _ref26;
  let [_let9 = { ["let"]: () => a }.let] = _ref27 === void 0 ? [] : _ref27;
  return _let9.name;
}
function klass(_ref28, _ref29 = void 0, _ref30 = void 0) {
  let { a } = _ref28, rest = babelHelpers.objectWithoutProperties(_ref28, _excluded10);
  let b = _ref29 === void 0 ? a : _ref29;
  let { let: _let10 = { ["let"]: class {
    static inferred() {
      return this.name;
    }
  } }.let } = _ref30 === void 0 ? {} : _ref30;
  return [_let10.name, _let10.inferred()];
}
function assignments(_ref31, _ref32 = void 0, _ref33) {
  let { a } = _ref31, rest = babelHelpers.objectWithoutProperties(_ref31, _excluded11);
  let b = _ref32 === void 0 ? a : _ref32;
  let _let11 = _ref33;
  _let11 = { ["let"]: () => a }.let;
  const names = [_let11.name];
  _let11 = null;
  _let11 ||= { ["let"]: function() {} }.let;
  names.push(_let11.name);
  _let11 &&= { ["let"]: class {} }.let;
  names.push(_let11.name);
  _let11 = null;
  _let11 ??= { ["let"]: () => a }.let;
  names.push(_let11.name);
  [_let11 = { ["let"]: function() {} }.let] = [];
  names.push(_let11.name);
  ({x: _let11 = { ["let"]: class {} }.let} = {});
  names.push(_let11.name);
  ({let: _let11 = { ["let"]: () => a }.let} = {});
  names.push(_let11.name);
  ({let: _let11} = { let: 4 });
  return [names, _let11];
}
