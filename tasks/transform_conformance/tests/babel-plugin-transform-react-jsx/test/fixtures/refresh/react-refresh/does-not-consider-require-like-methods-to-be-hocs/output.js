import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
const A = require("A");
const B = foo ? require("X") : require("Y");
const C = requireCond(gk, "C");
const D = import("D");
export default function App() {
  return _jsxs("div", { children: [_jsx(A, {}), _jsx(B, {}), _jsx(C, {}), _jsx(D, {})] });
}
_c = App;
var _c;
$RefreshReg$(_c, "App");
