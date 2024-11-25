const Obj = {
  value: 0,
  method() {
    var _superprop_setValue = (_value) => (super.value = _value),
      _superprop_set = (_prop, _value2) => (super[_prop] = _value2),
      _superprop_getObject = () => super.object;
    return babelHelpers.asyncToGenerator(function* () {
      _superprop_setValue(true);
      () => {
        _superprop_set("value", true);
        _superprop_getObject().value = true;
      };
    })();
  }
};
