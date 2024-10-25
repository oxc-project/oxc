const A = /*#__PURE__*/function () {
  var _ref = babelHelpers.asyncToGenerator(function* (a) {
    yield Promise.resolve();
  });
  return function A(_x) {
    return _ref.apply(this, arguments);
  };
}();
setTimeout(/*#__PURE__*/babelHelpers.asyncToGenerator(function* (p = 0) {
  yield Promise.resolve();
  console.log(p);
}));