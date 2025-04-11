import { jsx as _jsx } from "react/jsx-runtime";
function Hello() {
  function handleClick() {}
  return /* @__PURE__ */ _jsx("h1", {
    onClick: handleClick,
    children: "Hi"
  });
}
_c = Hello;
function Bar() {
  return /* @__PURE__ */ _jsx(Hello, {});
}
_c2 = Bar;
var _c, _c2;
$RefreshReg$(_c, "Hello");
$RefreshReg$(_c2, "Bar");
