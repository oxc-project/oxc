class Outer extends OuterSuper {
  constructor() {
    var _this;

    class Inner extends (super(), _this = this) {
      [(super(), _this = this)] = 1;
      static [(super(), _this = this)] = 2;
      [(super(), _this = this)]() {}
      static [(super(), _this = this)]() {}
    }

    let fn = /*#__PURE__*/function () {
      var _ref = babelHelpers.asyncToGenerator(function* () {
        return _this;
      });
      return function fn() {
        return _ref.apply(this, arguments);
      };
    }();

  }
}