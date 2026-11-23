function fn() {
  var _Class, _priv;
  return new (_Class = (0, class {
    method(obj) {
      return babelHelpers.checkInRHS(obj) === _Class && !!_priv;
    }
  }), _priv = { _: void 0 }, _Class)();
}
