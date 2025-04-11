import { jsx as _jsx } from "react/jsx-runtime";
let Hello = () => {
  const handleClick = () => {};
  return /* @__PURE__ */ _jsx("h1", {
    onClick: handleClick,
    children: "Hi"
  });
};
_c = Hello;
const Bar = () => {
  return /* @__PURE__ */ _jsx(Hello, {});
};
_c2 = Bar;
var Baz = () => /* @__PURE__ */ _jsx("div", {});
_c3 = Baz;
var sum = () => {};
var _c, _c2, _c3;
$RefreshReg$(_c, "Hello");
$RefreshReg$(_c2, "Bar");
$RefreshReg$(_c3, "Baz");
