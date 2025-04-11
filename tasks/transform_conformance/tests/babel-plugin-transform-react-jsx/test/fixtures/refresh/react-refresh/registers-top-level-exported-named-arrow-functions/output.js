import { jsx as _jsx } from "react/jsx-runtime";
export const Hello = () => {
  function handleClick() {}
  return /* @__PURE__ */ _jsx("h1", {
    onClick: handleClick,
    children: "Hi"
  });
};
_c = Hello;
export let Bar = (props) => /* @__PURE__ */ _jsx(Hello, {});
_c2 = Bar;
export default () => {
  return /* @__PURE__ */ _jsx(Hello, {});
};
var _c, _c2;
$RefreshReg$(_c, "Hello");
$RefreshReg$(_c2, "Bar");
