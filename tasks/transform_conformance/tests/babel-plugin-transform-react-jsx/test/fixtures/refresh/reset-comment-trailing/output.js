import { useCustom } from "hooks";
var _s = $RefreshSig$(), _s2 = $RefreshSig$(), _s3 = $RefreshSig$();
export function First() {
  _s();
  useState(0);
  return null;
}
_s(First, "useState{(0)}", true);
_c = First;
export const Second = () => {
  _s2();
  useCustom();
  return null;
};
_s2(Second, "useCustom{}", true, function() {
  return [useCustom];
});
_c2 = Second;
export const Third = _s3(memo(_c3 = _s3(function Third() {
  _s3();
  useEffect(() => {});
  return null;
}, "useEffect{}", true)), "useEffect{}", true);
_c4 = Third;
var _c, _c2, _c3, _c4;
$RefreshReg$(_c, "First");
$RefreshReg$(_c2, "Second");
$RefreshReg$(_c3, "Third$memo");
$RefreshReg$(_c4, "Third");
