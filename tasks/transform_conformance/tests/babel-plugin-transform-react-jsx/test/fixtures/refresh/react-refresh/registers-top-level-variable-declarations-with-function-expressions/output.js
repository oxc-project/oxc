import { jsx as _jsx } from "react/jsx-runtime";
let Hello = function() {
  function handleClick() {}
  return _jsx("h1", {
    onClick: handleClick,
    children: "Hi"
  });
};
_c = Hello;
const Bar = function Baz() {
  return _jsx(Hello, {});
};
_c2 = Bar;
function sum() {}
let Baz = 10;
var Qux;
var _c, _c2;
$RefreshReg$(_c, "Hello");
$RefreshReg$(_c2, "Bar");
