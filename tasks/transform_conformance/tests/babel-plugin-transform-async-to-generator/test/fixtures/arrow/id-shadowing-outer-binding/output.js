let count = 0;
const typeNext = function () {
  var _ref = babelHelpers.asyncToGenerator(function* () {
    count++;
    if (count < 3) typeNext();
  });
  return function typeNext() {
    return _ref.apply(this, arguments);
  };
}();
typeNext();
