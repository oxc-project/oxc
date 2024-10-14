import { jsx as _jsx } from "react/jsx-runtime";
function Foo() {
  return _jsx("h1", { children: "Hi" });
}
_c = Foo;
export default _c2 = hoc(Foo);
export const A = hoc(Foo);
_c3 = A;
const B = hoc(Foo);
_c4 = B;
var _c, _c2, _c3, _c4;
$RefreshReg$(_c, "Foo");
$RefreshReg$(_c2, "%default%");
$RefreshReg$(_c3, "A");
$RefreshReg$(_c4, "B");
