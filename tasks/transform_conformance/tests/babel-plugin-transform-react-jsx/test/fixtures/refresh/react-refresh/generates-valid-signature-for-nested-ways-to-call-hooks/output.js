import FancyHook from "fancy";
import { jsx as _jsx } from "react/jsx-runtime";
var _s = $RefreshSig$();
export default function App() {
  _s();
  const foo = FancyHook.property.useNestedThing();
  return /* @__PURE__ */ _jsx("h1", { children: foo });
}
_s(App, "useNestedThing{foo}", false, function () {
  return [FancyHook.property.useNestedThing];
});
_c = App;
var _c;
$RefreshReg$(_c, "App");
