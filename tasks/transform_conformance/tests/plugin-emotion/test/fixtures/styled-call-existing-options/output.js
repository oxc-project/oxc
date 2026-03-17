import styled from "@emotion/styled";
const Box = /* @__PURE__ */ styled("div", {
  target: "custom-target",
  label: "CustomLabel",
  shouldForwardProp: (prop) => prop !== "color",
})({ padding: 10 });
