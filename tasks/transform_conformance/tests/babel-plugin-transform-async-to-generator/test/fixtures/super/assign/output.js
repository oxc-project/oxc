const Obj = {
  value: 0,
  method() {
    var _superprop_getObject = () => super.object,
    _superprop_set = (_prop, _value) => super[_prop] = _value,
    _superprop_setValue = _value2 => super.value = _value2;
    return babelHelpers.asyncToGenerator(function* () {
      _superprop_setValue(true);
      () => {
        _superprop_set('value', true);
        _superprop_getObject().value = true;
      };
    })();
  }
};