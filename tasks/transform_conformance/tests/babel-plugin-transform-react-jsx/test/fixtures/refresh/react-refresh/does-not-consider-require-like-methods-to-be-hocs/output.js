import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
const A = require("A");
const B = foo ? require("X") : require("Y");
const C = requireCond(gk, "C");
const D = import("D");
export default function App() {
  return /* @__PURE__ */ _jsxs("div", { children: [
    /* @__PURE__ */ _jsx(A, {}),
    /* @__PURE__ */ _jsx(B, {}),
    /* @__PURE__ */ _jsx(C, {}),
    /* @__PURE__ */ _jsx(D, {})
  ] });
}
_c = App;
var _c;
$RefreshReg$(_c, "App");
