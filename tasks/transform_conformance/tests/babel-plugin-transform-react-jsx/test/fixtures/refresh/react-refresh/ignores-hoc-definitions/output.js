import { jsx as _jsx } from "react/jsx-runtime";
let connect = () => {
  function Comp() {
    const handleClick = () => {};
    return /* @__PURE__ */ _jsx("h1", {
      onClick: handleClick,
      children: "Hi"
    });
  }
  return Comp;
};
function withRouter() {
  return function Child() {
    const handleClick = () => {};
    return /* @__PURE__ */ _jsx("h1", {
      onClick: handleClick,
      children: "Hi"
    });
  };
}
;
