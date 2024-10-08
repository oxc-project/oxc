var _s = $RefreshSig$(), _s2 = $RefreshSig$(), _s3 = $RefreshSig$(), _s4 = $RefreshSig$(), _s5 = $RefreshSig$(), _s6 = $RefreshSig$();
export default _s(() => {
  _s();
  return useContext(X);
}, "useContext{}");
export const Foo = () => {
  _s2();
  return useContext(X);
};
_s2(Foo, "useContext{}");
_c = Foo;
module.exports = _s3(() => {
  _s3();
  return useContext(X);
}, "useContext{}");
const Bar = () => {
  _s4();
  return useContext(X);
};
_s4(Bar, "useContext{}");
_c2 = Bar;
const Baz = _s5(memo(_c3 = _s5(() => {
  _s5();
  return useContext(X);
}, "useContext{}")), "useContext{}");
_c4 = Baz;
const Qux = () => {
  _s6();
  return 0, useContext(X);
};
_s6(Qux, "useContext{}");
_c5 = Qux;
var _c, _c2, _c3, _c4, _c5;
$RefreshReg$(_c, "Foo");
$RefreshReg$(_c2, "Bar");
$RefreshReg$(_c3, "Baz$memo");
$RefreshReg$(_c4, "Baz");
$RefreshReg$(_c5, "Qux");
