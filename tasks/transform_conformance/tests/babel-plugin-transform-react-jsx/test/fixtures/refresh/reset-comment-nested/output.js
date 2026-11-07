var _s = $RefreshSig$(), _s2 = $RefreshSig$();
export function First() {
  _s();
  useState(0);
  return null;
}
_s(First, "useState{(0)}", true);
_c = First;
export function Second() {
  _s2();
  useEffect(() => {});
  return null;
}
_s2(Second, "useEffect{}", true);
_c2 = Second;
var _c, _c2;
$RefreshReg$(_c, "First");
$RefreshReg$(_c2, "Second");
