const func = /*#__PURE__*/function () {
  var _ref = babelHelpers.asyncToGenerator(function* (a, b) {
    console.log(a, yield Promise.resolve());
  });
  return function func(_x, _x2) {
    return _ref.apply(this, arguments);
  };
}();
setTimeout(/*#__PURE__*/babelHelpers.asyncToGenerator(function* (p = 0) {
  yield Promise.resolve();
}));