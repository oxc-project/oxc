class C extends S {
  static {
    var _superprop_getFoo = () => super.foo;
    this.fn = babelHelpers.asyncToGenerator(function* () {
      return _superprop_getFoo();
    });
  }
}
