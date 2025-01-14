class Cls {
  prop = (() => {
    var _superprop_getProp = () => super.prop, _this = this;
    return babelHelpers.asyncToGenerator(function* () {
      return _this, _superprop_getProp();
    });
  })();
  static prop = (() => {
    var _superprop_getProp2 = () => super.prop, _this2 = this;
    return babelHelpers.asyncToGenerator(function* () {
      return _this2, _superprop_getProp2();
    });
  })();

  nested = () => {
    var _superprop_getProp3 = () => super.prop, _this3 = this;
    /*#__PURE__*/babelHelpers.asyncToGenerator(function* () {
      return _this3, _superprop_getProp3();
    });
  };
  static nested = () => {
    var _superprop_getProp4 = () => super.prop, _this4 = this;
    /*#__PURE__*/babelHelpers.asyncToGenerator(function* () {
      return _this4, _superprop_getProp4();
    });
  };
}
