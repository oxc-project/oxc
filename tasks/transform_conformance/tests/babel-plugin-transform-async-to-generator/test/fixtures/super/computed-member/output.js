class Foo extends class {} {
  method() {
    var _superprop_get = _prop => super[_prop],
      _this = this;
    return babelHelpers.asyncToGenerator(function* () {
      _superprop_get('name');
      {
        _superprop_get('name').call(_this);
        _superprop_get('object')['name']();
      }
    })();
  }
}