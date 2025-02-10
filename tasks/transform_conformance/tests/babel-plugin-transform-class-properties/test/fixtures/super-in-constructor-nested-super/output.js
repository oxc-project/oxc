let c;

class S {}

class C extends S {
  constructor() {
    var _super = (..._args) => (super(..._args), babelHelpers.defineProperty(this, "prop", 123), this);
    _super(c = _super());
  }
}
