var _Methods_brand = /* @__PURE__ */ new WeakSet();
class Methods {
  constructor() {
    babelHelpers.classPrivateMethodInitSpec(this, _Methods_brand);
  }
  run(object, rhs) {
    var _object$method, _object$method2, _object$method3;
    babelHelpers.assertClassBrand(_Methods_brand, object, _method) || (object, rhs(), babelHelpers.readOnlyError("#method"));
    babelHelpers.assertClassBrand(_Methods_brand, object, _method) ?? (object, rhs(), babelHelpers.readOnlyError("#method"));
    babelHelpers.assertClassBrand(_Methods_brand, object, _method) && (object, rhs(), babelHelpers.readOnlyError("#method"));
    object, babelHelpers.assertClassBrand(_Methods_brand, object, _method) + rhs(), babelHelpers.readOnlyError("#method");
    object, _object$method = babelHelpers.assertClassBrand(_Methods_brand, object, _method), ++_object$method, babelHelpers.readOnlyError("#method");
    object, _object$method2 = babelHelpers.assertClassBrand(_Methods_brand, object, _method), _object$method3 = _object$method2--, _object$method2, babelHelpers.readOnlyError("#method"), _object$method3;
  }
  static run(object, rhs) {
    var _object$staticMethod, _object$staticMethod2, _object$staticMethod3;
    babelHelpers.assertClassBrand(Methods, object, _staticMethod) || (object, rhs(), babelHelpers.readOnlyError("#staticMethod"));
    babelHelpers.assertClassBrand(Methods, object, _staticMethod) ?? (object, rhs(), babelHelpers.readOnlyError("#staticMethod"));
    babelHelpers.assertClassBrand(Methods, object, _staticMethod) && (object, rhs(), babelHelpers.readOnlyError("#staticMethod"));
    object, babelHelpers.assertClassBrand(Methods, object, _staticMethod) + rhs(), babelHelpers.readOnlyError("#staticMethod");
    object, _object$staticMethod = babelHelpers.assertClassBrand(Methods, object, _staticMethod), ++_object$staticMethod, babelHelpers.readOnlyError("#staticMethod");
    object, _object$staticMethod2 = babelHelpers.assertClassBrand(Methods, object, _staticMethod), _object$staticMethod3 = _object$staticMethod2--, _object$staticMethod2, babelHelpers.readOnlyError("#staticMethod"), _object$staticMethod3;
  }
}
function _method() {}
function _staticMethod() {}
