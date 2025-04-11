import { jsx as _jsx } from "react/jsx-runtime";

// Valid
/* @__PURE__ */ _jsx("div", { children: "\xA0 ¡ ¢ £ ¤ ¥ ¦ § ¨ ©" });
/* @__PURE__ */ _jsx("div", { children: "ª « ¬ ­ ® ¯ ° ± ² ³" });
/* @__PURE__ */ _jsx("div", { children: "´ µ ¶ · ¸ ¹ º » ¼ ½" });
/* @__PURE__ */ _jsx("div", { children: "¾ ¿ À Á Â Ã Ä Å Æ Ç" });
/* @__PURE__ */ _jsx("div", { children: "È É Ê Ë Ì Í Î Ï Ð Ñ" });
/* @__PURE__ */ _jsx("div", { children: "Ò Ó Ô Õ Ö × Ø Ù Ú Û" });
/* @__PURE__ */ _jsx("div", { children: "Ü Ý Þ ß à á â ã ä å" });
/* @__PURE__ */ _jsx("div", { children: "æ ç è é ê ë ì í î ï" });
/* @__PURE__ */ _jsx("div", { children: "ð ñ ò ó ô õ ö ÷ ø ù" });
/* @__PURE__ */ _jsx("div", { children: "ú û ü ý þ ÿ ƒ Α Β Γ" });
/* @__PURE__ */ _jsx("div", { children: "Δ Ε Ζ Η Θ Ι Κ Λ Μ Ν" });
/* @__PURE__ */ _jsx("div", { children: "Ξ Ο Π Ρ Σ Τ Υ Φ Χ Ψ" });
/* @__PURE__ */ _jsx("div", { children: "Ω α β γ δ ε ζ η θ ι" });
/* @__PURE__ */ _jsx("div", { children: "κ λ μ ν ξ ο π ρ ς σ" });
/* @__PURE__ */ _jsx("div", { children: "τ υ φ χ ψ ω ϑ ϒ ϖ •" });
/* @__PURE__ */ _jsx("div", { children: "… ′ ″ ‾ ⁄ ℘ ℑ ℜ ™ ℵ" });
/* @__PURE__ */ _jsx("div", { children: "← ↑ → ↓ ↔ ↵ ⇐ ⇑ ⇒ ⇓" });
/* @__PURE__ */ _jsx("div", { children: "⇔ ∀ ∂ ∃ ∅ ∇ ∈ ∉ ∋ ∏" });
/* @__PURE__ */ _jsx("div", { children: "∑ − ∗ √ ∝ ∞ ∠ ∧ ∨ ∩" });
/* @__PURE__ */ _jsx("div", { children: "∪ ∫ ∴ ∼ ≅ ≈ ≠ ≡ ≤ ≥" });
/* @__PURE__ */ _jsx("div", { children: "⊂ ⊃ ⊄ ⊆ ⊇ ⊕ ⊗ ⊥ ⋅ ⌈" });
/* @__PURE__ */ _jsx("div", { children: "⌉ ⌊ ⌋ 〈 〉 ◊ ♠ ♣ ♥ ♦" });
/* @__PURE__ */ _jsx("div", { children: "\" & < > Œ œ Š š Ÿ ˆ" });
/* @__PURE__ */ _jsx("div", { children: "˜       ‌ ‍ ‎ ‏ – —" });
/* @__PURE__ */ _jsx("div", { children: "‘ ’ ‚ “ ” „ † ‡ ‰ ‹" });
/* @__PURE__ */ _jsx("div", { children: "› €" });

// Invalid
/* @__PURE__ */ _jsx("div", { children: "&donkey;" });
