let first, second, rest;
var _ref;
var _ref2;
[..._ref] = [1, 2], _ref2 = _ref, {} = _ref2, rest = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref2), _ref2)), _ref2;
var _ref3;
var _ref4;
[first, ..._ref3] = [
  1,
  2,
  3
], _ref4 = _ref3, {0: second} = _ref4, rest = babelHelpers.objectWithoutProperties(_ref4, ["0"]), _ref4;
var _ref5;
var _ref6;
[...[_ref5]] = [{ value: 1 }], _ref6 = _ref5, {} = _ref6, rest = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref6), _ref6)), _ref6;
for (var _ref7 of [[1, 2]]) {
  var _ref8;
  var _ref9;
  [..._ref8] = _ref7, _ref9 = _ref8, {} = _ref9, rest = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref9), _ref9)), _ref9;
}
for (var _ref10 in { ab: 0 }) {
  var _ref11;
  var _ref12;
  [..._ref11] = _ref10, _ref12 = _ref11, {} = _ref12, rest = babelHelpers.extends({}, (babelHelpers.objectDestructuringEmpty(_ref12), _ref12)), _ref12;
}
