import { jsx as _jsx } from "react/jsx-runtime";
let A = foo ? () => {
  return _jsx("h1", { children: "Hi" });
} : null;
const B = function Foo() {
  return _jsx("h1", { children: "Hi" });
}();
let C = () => () => {
  return _jsx("h1", { children: "Hi" });
};
let D = bar && (() => {
  return _jsx("h1", { children: "Hi" });
});
