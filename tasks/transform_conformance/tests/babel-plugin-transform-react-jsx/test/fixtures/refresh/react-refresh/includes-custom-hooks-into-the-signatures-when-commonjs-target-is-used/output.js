"use strict";
import { jsx as _jsx } from "react/jsx-runtime";
Object.defineProperty(exports, "__esModule", { value: true });
(exports.default = App);
var _hooks = require("./hooks");
var _s = $RefreshSig$();
function App() {
  _s();
  const bar = (0, _hooks.useFancyState)();
  return _jsx("h1", { children: bar });
}
_s(App, "useFancyState{bar}", false, function() {
  return [_hooks.useFancyState];
});
_c = App;
var _c;
$RefreshReg$(_c, "App");
