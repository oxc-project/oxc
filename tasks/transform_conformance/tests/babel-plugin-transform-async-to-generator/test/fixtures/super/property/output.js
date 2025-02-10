class Cls {
  prop = (() => {
    var _superprop_getProp = () => super.prop;
    return babelHelpers.asyncToGenerator(function* () {
      _superprop_getProp();
    });
  })();
  static prop = (() => {
    var _superprop_getProp2 = () => super.prop;
    return babelHelpers.asyncToGenerator(function* () {
      _superprop_getProp2();
    });
  })();
}
