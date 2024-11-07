let g = function() {
  var _ref = babelHelpers.asyncToGenerator(function* () {
          console.log("Good");
  });
  return function g() {
          return _ref.apply(this, arguments);
  };
}();
