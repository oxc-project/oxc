class Outer extends OuterSuper {
  constructor() {
    var _this;
    var _super, _super2, _super3, _super4, _super5, _Inner_decorators, _superclass, _member_dec, _member_dec2, _member_dec3, _member_dec4;
    let _init, _Inner, _babelHelpers$toPrope, _babelHelpers$toPrope2, _babelHelpers$toPrope3, _babelHelpers$toPrope4, _babelHelpers$toPrope5;
    _Inner_decorators = [[_super = (super(), _this = this), _super.decorate]];
    class Inner extends (_superclass = (super(), _this = this), _member_dec = [[_super2 = (super(), _this = this), _super2.decorate]], _babelHelpers$toPrope = babelHelpers.toPropertyKey((super(), _this = this)), _member_dec2 = [[_super3 = (super(), _this = this), _super3.decorate]], _babelHelpers$toPrope2 = babelHelpers.toPropertyKey((super(), _this = this)), _babelHelpers$toPrope3 = _babelHelpers$toPrope2, _member_dec3 = [[_super4 = (super(), _this = this), _super4.decorate]], _babelHelpers$toPrope4 = babelHelpers.toPropertyKey((super(), _this = this)), _member_dec4 = [[_super5 = (super(), _this = this), _super5.decorate]], _babelHelpers$toPrope5 = babelHelpers.toPropertyKey((super(), _this = this)), _superclass) {
      constructor(..._args) {
        super(..._args);
        babelHelpers.runInitializers(_init, 5, this);
        babelHelpers.defineProperty(this, _babelHelpers$toPrope, babelHelpers.runInitializers(_init, 12, this, 1));
        babelHelpers.runInitializers(_init, 15, this);
      }
      [_babelHelpers$toPrope4]() {}
      static [_babelHelpers$toPrope5]() {}
    }
    _Inner = Inner;
    _init = babelHelpers.decoratorStart(_superclass);
    babelHelpers.decorateElement(_init, 9, _babelHelpers$toPrope5, _member_dec4, Inner);
    babelHelpers.decorateElement(_init, 1, _babelHelpers$toPrope4, _member_dec3, Inner);
    babelHelpers.decorateElement(_init, 13, _babelHelpers$toPrope2, _member_dec2, Inner);
    babelHelpers.decorateElement(_init, 5, _babelHelpers$toPrope, _member_dec, Inner);
    _Inner = Inner = babelHelpers.decorateElement(_init, 0, "Inner", _Inner_decorators, Inner);
    babelHelpers.runInitializers(_init, 3, Inner);
    babelHelpers.defineProperty(Inner, _babelHelpers$toPrope3, babelHelpers.runInitializers(_init, 8, _Inner, 2));
    babelHelpers.runInitializers(_init, 11, _Inner);
    babelHelpers.runInitializers(_init, 1, Inner);
    let fn = function() {
      var _ref = babelHelpers.asyncToGenerator(function* () {
        return _this;
      });
      return function fn() {
        return _ref.apply(this, arguments);
      };
    }();
  }
}
