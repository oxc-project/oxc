var _C;
class C {}
_C = C;
babelHelpers.defineProperty(C, "fn", (() => {
  var _superprop_getStaticProp = () => babelHelpers.superPropGet(_C, "staticProp", _C), _this = _C;
  return babelHelpers.asyncToGenerator(function* () {
    return [_this, _superprop_getStaticProp()];
  });
})());
