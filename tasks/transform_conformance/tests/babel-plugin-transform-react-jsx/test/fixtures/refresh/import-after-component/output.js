import { observer } from "mobx-react-lite";
import { useFoo } from "./useFoo";
import { Fragment as _Fragment, jsxs as _jsxs } from "react/jsx-runtime";
var _s = $RefreshSig$();

export const BazComponent = _s(observer(_c = _s(function BazComponent_() {
  _s();
  const foo = useFoo();
  return /* @__PURE__ */ _jsxs(_Fragment, { children: [foo, bar] });
}, "useFoo{foo}", false, function() {
  return [useFoo];
})), "useFoo{foo}", false, function() {
  return [useFoo];
});
_c2 = BazComponent;

import { bar } from "./bar";

var _c, _c2;
$RefreshReg$(_c, "BazComponent$observer");
$RefreshReg$(_c2, "BazComponent");
