import { jsx as _jsx } from "react/jsx-runtime";
let A = foo ? () => {
  return /* @__PURE__ */ _jsx("h1", { children: "Hi" });
} : null;
const B = (function Foo() {
  return /* @__PURE__ */ _jsx("h1", { children: "Hi" });
})();
let C = () => () => {
  return /* @__PURE__ */ _jsx("h1", { children: "Hi" });
};
let D = bar && (() => {
  return /* @__PURE__ */ _jsx("h1", { children: "Hi" });
});
