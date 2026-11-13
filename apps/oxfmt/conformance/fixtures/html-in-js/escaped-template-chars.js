// Template-literal characters (backtick, ${, backslash) inside html-in-js must be
// re-escaped when the formatted HTML IR is reinserted into the template literal
// (escape_template_chars_in_ir; the joined text is built from `.cooked` values).
const escaped = html`<p>tick \` dollar \${notAnExpression} backslash \\ end</p>`;

// Mixed with a real expression: escapes and placeholder substitution in the same text run.
const mixed = html`<div><code>\`${name}\` costs \${amount}</code><p>${description}</p></div>`;
