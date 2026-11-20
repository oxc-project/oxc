var _toPropertyKey = require("@oxc-project/runtime/helpers/toPropertyKey");
var _runInitializers = require("@oxc-project/runtime/helpers/runInitializers");
var _defineProperty = require("@oxc-project/runtime/helpers/defineProperty");
var _decorateElement = require("@oxc-project/runtime/helpers/decorateElement");
var _decoratorStart = require("@oxc-project/runtime/helpers/decoratorStart");
var _asyncToGenerator = require("@oxc-project/runtime/helpers/asyncToGenerator");
class Outer extends OuterSuper {
  constructor() {
    var _this;
    var _super, _super2, _super3, _super4, _super5, _Inner_decorators, _superclass, _member_dec, _member_dec2, _member_dec3, _member_dec4;
    let _init, _toPropertyKey2, _toPropertyKey3, _toPropertyKey4, _toPropertyKey5, _toPropertyKey6, _Inner;
    _Inner_decorators = [[_super = (super(), _this = this), _super.decorate]];
    class Inner extends (_superclass = (super(), _this = this), _member_dec = [[_super2 = (super(), _this = this), _super2.decorate]], _toPropertyKey2 = _toPropertyKey((super(), _this = this)), _member_dec2 = [[_super3 = (super(), _this = this), _super3.decorate]], _toPropertyKey3 = _toPropertyKey((super(), _this = this)), _toPropertyKey4 = _toPropertyKey3, _member_dec3 = [[_super4 = (super(), _this = this), _super4.decorate]], _toPropertyKey5 = _toPropertyKey((super(), _this = this)), _member_dec4 = [[_super5 = (super(), _this = this), _super5.decorate]], _toPropertyKey6 = _toPropertyKey((super(), _this = this)), _superclass) {
      constructor(..._args) {
        super(..._args);
        _runInitializers(_init, 5, this);
        _defineProperty(this, _toPropertyKey2, _runInitializers(_init, 12, this, 1));
        _runInitializers(_init, 15, this);
      }
      [_toPropertyKey5]() {}
      static [_toPropertyKey6]() {}
    }
    _Inner = Inner;
    _init = _decoratorStart(_superclass);
    _decorateElement(_init, 9, _toPropertyKey6, _member_dec4, Inner);
    _decorateElement(_init, 1, _toPropertyKey5, _member_dec3, Inner);
    _decorateElement(_init, 13, _toPropertyKey3, _member_dec2, Inner);
    _decorateElement(_init, 5, _toPropertyKey2, _member_dec, Inner);
    _Inner = Inner = _decorateElement(_init, 0, "Inner", _Inner_decorators, Inner);
    _runInitializers(_init, 3, Inner);
    _defineProperty(Inner, _toPropertyKey4, _runInitializers(_init, 8, _Inner, 2));
    _runInitializers(_init, 11, _Inner);
    _runInitializers(_init, 1, Inner);
    let fn = function() {
      var _ref = _asyncToGenerator(function* () {
        return _this;
      });
      return function fn() {
        return _ref.apply(this, arguments);
      };
    }();
  }
}
