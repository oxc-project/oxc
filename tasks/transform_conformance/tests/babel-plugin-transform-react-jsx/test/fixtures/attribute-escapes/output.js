import { jsx as _jsx } from "react/jsx-runtime";

// Valid escapes
/* @__PURE__ */ _jsx(Foo, { bar: "È € \"" });
/* @__PURE__ */ _jsx(Foo, { bar: "\f A" });
/* @__PURE__ */ _jsx(Foo, { bar: "\f A" });

// Invalid escapes
/* @__PURE__ */ _jsx(Foo, { bar: "&donkey; &#x110000; &#xFFFFFF; &#xG; &#1114112; &#16777215; &#C;" });

// Unterminated escapes
/* @__PURE__ */ _jsx(Foo, { bar: "&euro xxx" });
/* @__PURE__ */ _jsx(Foo, { bar: "&#123 xxx" });
/* @__PURE__ */ _jsx(Foo, { bar: "&#x123 xxx" });
