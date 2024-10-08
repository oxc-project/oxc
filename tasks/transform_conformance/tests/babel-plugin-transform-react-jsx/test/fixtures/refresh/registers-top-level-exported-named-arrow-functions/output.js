import { jsx as _jsx } from "react/jsx-runtime";
export const Hello = () => {
  function handleClick() {}
  return _jsx("h1", {
    onClick: handleClick,
    children: "Hi"
  });
};
_c = Hello;
export let Bar = (props) => _jsx(Hello, {});
_c2 = Bar;
export default () => {
  return _jsx(Hello, {});
};
var _c, _c2;
$RefreshReg$(_c, "Hello");
$RefreshReg$(_c2, "Bar");
