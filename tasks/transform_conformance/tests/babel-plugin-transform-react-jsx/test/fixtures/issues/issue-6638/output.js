import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
export function App() {
  return /* @__PURE__ */ _jsxs(Suspense, {
    fallback: "Loading...",
    children: [/* @__PURE__ */ _jsx(Init, {}), /* @__PURE__ */ _jsxs(PanelGroup, {
      direction: "horizontal",
      className: "app-main",
      children: [
        /* @__PURE__ */ _jsx(Panel, {
          defaultSize: 50,
          minSize: 33,
          maxSize: 66,
          children: /* @__PURE__ */ _jsx(Input, {})
        }),
        /* @__PURE__ */ _jsx(PanelResizeHandle, { className: "divider" }),
        /* @__PURE__ */ _jsx(Panel, { children: /* @__PURE__ */ _jsx(Output, {}) })
      ]
    })]
  });
}
