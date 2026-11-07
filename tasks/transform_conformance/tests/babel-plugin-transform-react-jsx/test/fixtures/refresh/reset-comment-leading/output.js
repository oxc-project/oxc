import { useState } from "react";
import { jsx as _jsx } from "react/jsx-runtime";
var _s = $RefreshSig$();
export default function App() {
  _s();
  const [count, setCount] = useState(0);
  return /* @__PURE__ */ _jsx("button", {
    onClick: () => setCount(count + 1),
    children: count
  });
}
_s(App, "oDgYfYHkD9Wkv4hrAPCkI/ev3YU=", true);
_c = App;
var _c;
$RefreshReg$(_c, "App");
