import { jsx as _jsx } from "react/jsx-runtime";

// Valid hex
/* @__PURE__ */ _jsx("div", { children: "\f A ģ ሴ 𐀀 􏿿" });

// Invalid hex
/* @__PURE__ */ _jsx("div", { children: "&#x110000; &#xFFFFFF; &#xG;" });

// Valid decimal (same characters as valid hex above)
/* @__PURE__ */ _jsx("div", { children: "\f A ģ ሴ 𐀀 􏿿" });

// Invalid decimal
/* @__PURE__ */ _jsx("div", { children: "&#1114112; &#16777215; &#C;" });
