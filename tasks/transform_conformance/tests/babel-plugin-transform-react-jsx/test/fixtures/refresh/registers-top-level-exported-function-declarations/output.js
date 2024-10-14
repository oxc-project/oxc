import { jsx as _jsx } from "react/jsx-runtime";
export function Hello() {
  function handleClick() {}
  return _jsx("h1", {
    onClick: handleClick,
    children: "Hi"
  });
}
_c = Hello;
export default function Bar() {
  return _jsx(Hello, {});
}
_c2 = Bar;
function Baz() {
  return _jsx("h1", { children: "OK" });
}
_c3 = Baz;
const NotAComp = 'hi';
export { Baz, NotAComp };
export function sum() {}
export const Bad = 42;

var _c, _c2, _c3;

$RefreshReg$(_c, "Hello");
$RefreshReg$(_c2, "Bar");
$RefreshReg$(_c3, "Baz");
