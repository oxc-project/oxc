const _excluded = ["a"], _excluded2 = ["x"], _excluded3 = ["a1"], _excluded4 = ["a2"], _excluded5 = ["a"], _excluded6 = ["c"], _excluded7 = ["h"], _excluded8 = ["m"];
for (var _ref of [{
  a: 1,
  c: 2
}]) {
  var { a } = _ref, b = babelHelpers.objectWithoutProperties(_ref, _excluded);
}
console.log(a, b);
function f(items) {
  for (var _ref2 of items) {
    var { x } = _ref2, y = babelHelpers.objectWithoutProperties(_ref2, _excluded2);
  }
  return [x, y];
}
for (let _ref3 of []) {
  let { a1 } = _ref3, b1 = babelHelpers.objectWithoutProperties(_ref3, _excluded3);
}
try {} catch (_ref4) {
  let { a2 } = _ref4, b2 = babelHelpers.objectWithoutProperties(_ref4, _excluded4);
}
function g(o, list) {
  var a;
  for (var _ref5 in o) {
    var { a } = _ref5, b = babelHelpers.objectWithoutProperties(_ref5, _excluded5);
  }
  for (var [_ref6] of list) {
    var { c } = _ref6, d = babelHelpers.objectWithoutProperties(_ref6, _excluded6);
  }
  return [
    a,
    b,
    c,
    d
  ];
}
function h() {}
for (var _ref7 of []) {
  var { h } = _ref7, i = babelHelpers.objectWithoutProperties(_ref7, _excluded7);
}
async function k(items) {
  for await (var _ref8 of items) {
    var { m } = _ref8, n = babelHelpers.objectWithoutProperties(_ref8, _excluded8);
  }
  return [m, n];
}
