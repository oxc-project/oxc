function customIteratorMethod() {
  let previous = null;
  let next = null;
  return {
    previous: function () {
      var _ref = babelHelpers.asyncToGenerator(function* () {
        return previous || (previous = "previous value");
      });
      return function previous() {
        return _ref.apply(this, arguments);
      };
    }(),
    next: function () {
      var _ref2 = babelHelpers.asyncToGenerator(function* () {
        return next || (next = "next value");
      });
      return function next() {
        return _ref2.apply(this, arguments);
      };
    }()
  };
}
